//! Check command execution logic.
//!
//! The direct runner delegates to `vize_canon`'s project-backed Corsa type
//! checker so Vue SFCs, TypeScript sources, ambient declarations, and emitted
//! `.d.ts` output all share the same virtual project.

#![allow(clippy::disallowed_macros)]

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde_json::{Map, Value};
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, batch::TypeChecker as BatchTypeCheckerTrait,
};
use vize_carton::{FxHashSet, String, cstr, profiler::global_profiler};

use crate::profile_support;
use vize_curator::profile::{ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report};

use super::{
    CheckArgs,
    path_cache::CanonicalPathCache,
    reporting::{JsonFileResult, JsonOutput},
    tsconfig_inputs::{
        TsconfigInputCache, collect_ambient_declaration_files, collect_default_check_files,
        resolve_tsconfig_for_files,
    },
};

mod collect;
mod diagnostics;
mod global_components;
mod nuxt_tsconfig;
mod resolve;
#[cfg(unix)]
mod socket;
#[cfg(test)]
mod tests;

use collect::collect_check_files;
use diagnostics::{
    emit_json_output, is_reported, is_suppressed_false_positive, render_diagnostics,
    save_virtual_ts_for_path, write_profile_virtual_ts,
};
use global_components::{
    build_virtual_ts_options, collect_project_global_component_stubs, dialect_from_features,
    template_syntax_mode,
};
use nuxt_tsconfig::resolve_checker_tsconfig_path;
#[cfg(test)]
use nuxt_tsconfig::write_nuxt_fallback_tsconfig;
use resolve::{
    display_path, resolve_declaration_emit_options, resolve_from_config_dir, resolve_project_root,
    resolve_tsconfig_path, validate_corsa_server_count,
};
#[cfg(test)]
use resolve::{find_nearest_tsconfig_dir, resolve_declaration_dir};
#[cfg(unix)]
pub(crate) use socket::run_with_socket;

#[allow(clippy::disallowed_types)]
type JsonObject = Map<std::string::String, Value>;

/// Run type checking directly with a materialized Corsa project.
pub(crate) fn run_direct(args: &CheckArgs) {
    use super::nuxt;

    let start = Instant::now();
    if args.profile {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
    }

    crate::config::write_schema(None);

    if let Some(path) = args.config.as_deref()
        && !args.no_config
        && let Err(error) = crate::config::validate_explicit_config_path(path)
    {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let loaded_config = if args.no_config {
        crate::config::LoadedConfigWithFeatures {
            config: crate::config::VizeConfig::default(),
            source_path: None,
            features: crate::config::ConfigFeatureFlags::default(),
        }
    } else {
        crate::config::load_config_with_features_and_source(args.config.as_deref())
    };
    let compiler_template_syntax = loaded_config
        .source_path
        .as_deref()
        .and_then(|path| crate::config::load_compiler_template_syntax(Some(path)));
    // Configured Vue dialect (`vue.version`). Defaults to Vue 3 when unset.
    // Plumbing only today: threaded into canon's virtual-TS generation so it can
    // later emit dialect-aware instance types; it does not change output yet.
    let dialect = dialect_from_features(loaded_config.features.vue_version);
    // Vue 3 Options API binding resolution is officially supported and is a
    // standard-build opt-in (not the `legacy` feature).
    let options_api = loaded_config.features.type_checker_options_api;
    // Legacy Vue 2.7 / Nuxt 2 Options-API type checking is opt-in and compiled out
    // of the default Vue 3 binary. Without the `legacy` feature, honor the config
    // flag by warning instead of silently ignoring it.
    #[cfg(feature = "legacy")]
    let legacy_vue2 = loaded_config.features.type_checker_legacy_vue2;
    #[cfg(not(feature = "legacy"))]
    if loaded_config.features.type_checker_legacy_vue2 {
        eprintln!(
            "\x1b[33mwarning:\x1b[0m `type_checker_legacy_vue2` is set but this `vize` build \
             has no legacy Vue support; rebuild with `--features legacy` to enable Vue 2 \
             Options API type checking."
        );
    }
    let config = loaded_config.config;
    let config_dir = loaded_config
        .source_path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(cwd.as_path());
    if !config.type_checker.enabled {
        eprintln!("[vize] Skipping check because typeChecker.enabled is false in vize.config.");
        return;
    }
    let effective_tsconfig = args
        .tsconfig
        .clone()
        .or_else(|| config.type_checker.tsconfig.as_ref().map(PathBuf::from));
    let effective_corsa_path = args.corsa_path.as_ref().map(PathBuf::from).or_else(|| {
        config
            .type_checker
            .runtime_path()
            .map(|candidate| resolve_from_config_dir(config_dir, candidate))
    });
    let corsa_servers = args.servers.or(config.type_checker.servers);
    if let Err(error) = validate_corsa_server_count(corsa_servers) {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }
    let project_root = resolve_project_root(effective_tsconfig.as_deref(), &cwd, &[]);
    let tsconfig_path =
        resolve_tsconfig_path(effective_tsconfig.as_deref(), &cwd, &project_root, &[]);
    // Run-scoped caches: tsconfig chains are parsed (and their include/exclude
    // globs compiled) once per run, and each unique path is canonicalized at
    // most once across reported-file bookkeeping, diagnostic filtering, and
    // transitive import resolution.
    let mut tsconfig_input_cache = TsconfigInputCache::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let collect_start = Instant::now();
    let mut files = if args.patterns.is_empty() {
        collect_default_check_files(
            &project_root,
            tsconfig_path.as_deref(),
            &mut tsconfig_input_cache,
        )
    } else {
        collect_check_files(&args.patterns)
    };
    let explicit_files = if args.patterns.is_empty() {
        Vec::new()
    } else {
        files.clone()
    };
    let collect_time = collect_start.elapsed();

    // For an explicit subset, only the requested files' diagnostics are
    // reported: ambient `.d.ts` and transitively-registered relative imports are
    // pulled into the program solely so cross-file types resolve, not to surface
    // diagnostics for files the user did not ask about. `None` reports every
    // registered file (the default full-project run).
    let reported_files: Option<FxHashSet<PathBuf>> = if args.patterns.is_empty() {
        None
    } else {
        Some(
            files
                .iter()
                .map(|path| canonical_paths.canonicalize(path))
                .collect(),
        )
    };

    if files.is_empty() {
        if args.format == "json" {
            emit_json_output(JsonOutput {
                files: Vec::new(),
                error_count: 0,
                warning_count: 0,
                file_count: 0,
                declarations: None,
            });
            return;
        }
        eprintln!(
            "No Vue or TypeScript files found matching inputs: {:?}",
            args.patterns
        );
        return;
    }

    // An explicit subset only registers the requested files, so a relative
    // import (`import { Foo } from './types'`) cannot see its sibling's real
    // types and degrades to `any`. Register the transitive closure of relative
    // source imports — analogous to the ambient pull-in below — so cross-file
    // types resolve precisely, the way tsc/vue-tsc load the reachable program.
    // Do this before root resolution so cwd-external files without tsconfig can
    // still choose a materialization root covering all registered source files.
    if !args.patterns.is_empty() {
        for path in
            super::imports::collect_transitive_local_imports(&files, &cwd, &mut canonical_paths)
        {
            if !files.contains(&path) {
                files.push(path);
            }
        }
        files.sort();
        files.dedup();
    }

    let project_root = resolve_project_root(effective_tsconfig.as_deref(), &cwd, &files);
    let tsconfig_path =
        resolve_tsconfig_path(effective_tsconfig.as_deref(), &cwd, &project_root, &files);
    let program_tsconfig_path = if args.patterns.is_empty() {
        tsconfig_path.clone()
    } else {
        resolve_tsconfig_for_files(
            tsconfig_path.as_deref(),
            &explicit_files,
            &mut tsconfig_input_cache,
        )
    };

    // An explicit file subset (`vize check src/App.vue`) omits ambient
    // declaration files, since nothing imports them; `declare global` types
    // would then be missing and surface as false `TS2304` errors. Pull the
    // tsconfig program's `.d.ts` files back in so global types stay in scope.
    if !args.patterns.is_empty() && program_tsconfig_path.is_some() {
        for path in collect_ambient_declaration_files(
            &project_root,
            program_tsconfig_path.as_deref(),
            &mut tsconfig_input_cache,
        ) {
            if !files.contains(&path) {
                files.push(path);
            }
        }
        files.sort();
        files.dedup();
    }

    let mut virtual_ts_options = build_virtual_ts_options(&config, config_dir);
    let nuxt_path_aliases = nuxt::detect_nuxt_auto_imports(&mut virtual_ts_options, &project_root);
    collect_project_global_component_stubs(
        &mut virtual_ts_options,
        &files,
        &project_root,
        program_tsconfig_path.as_deref(),
    );
    let checker_tsconfig_path = match resolve_checker_tsconfig_path(
        program_tsconfig_path.as_deref(),
        &project_root,
        &nuxt_path_aliases,
    ) {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "\x1b[31mError:\x1b[0m Failed to prepare type checker tsconfig: {}",
                error
            );
            std::process::exit(1);
        }
    };

    if !args.quiet {
        eprintln!(
            "Building Corsa virtual project for {} files under {}...",
            files.len(),
            project_root.display()
        );
    }

    let gen_start = Instant::now();
    let mut checker = match BatchTypeChecker::with_options_and_corsa_path(
        &project_root,
        BatchTypeCheckerOptions {
            tsconfig_path: checker_tsconfig_path.clone(),
            virtual_ts_options,
        },
        effective_corsa_path.as_deref(),
    ) {
        Ok(checker) => checker,
        Err(error) => {
            eprintln!("\x1b[31mError:\x1b[0m {}", error);
            std::process::exit(1);
        }
    };
    checker.set_server_count(corsa_servers);
    if options_api {
        checker.enable_options_api();
    }
    #[cfg(feature = "legacy")]
    if legacy_vue2 {
        checker.enable_legacy_vue2();
    }
    checker.set_template_syntax(template_syntax_mode(compiler_template_syntax));
    checker.set_dialect(dialect);
    checker.set_virtual_ts_checks(
        config.type_checker.check_props && !args.no_check_props,
        config.type_checker.check_template_bindings && !args.no_check_template_bindings,
        config.type_checker.check_emits && !args.no_check_emits,
    );

    if let Err(error) = checker.scan_paths(&files) {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(1);
    }
    let gen_time = gen_start.elapsed();

    let virtual_files = checker.virtual_files();
    if virtual_files.is_empty() {
        if args.format == "json" {
            emit_json_output(JsonOutput {
                files: Vec::new(),
                error_count: 0,
                warning_count: 0,
                file_count: 0,
                declarations: None,
            });
            return;
        }
        eprintln!("No files were registered for type checking");
        return;
    }

    if args.show_virtual_ts {
        eprintln!(
            "\n=== {} ===",
            vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME
        );
        eprintln!("{}", vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
        for file in &virtual_files {
            eprintln!("\n=== {} ===", file.original_path.display());
            eprintln!("{}", file.content);
        }
    }

    if let Some(path) = args.save_virtual_ts_for.as_deref() {
        match save_virtual_ts_for_path(
            path,
            &cwd,
            virtual_files
                .iter()
                .map(|file| (file.original_path.as_path(), file.content.as_str())),
        ) {
            Ok(target) => {
                if !args.quiet {
                    eprintln!("Saved Virtual TS to {}", target.display());
                }
            }
            Err(error) => {
                eprintln!("\x1b[31mError:\x1b[0m {}", error);
                std::process::exit(1);
            }
        }
    }

    let profile_artifact_start = Instant::now();
    if args.profile {
        write_profile_virtual_ts(&virtual_files);
    }
    let profile_artifact_time = profile_artifact_start.elapsed();

    if !args.quiet {
        eprintln!(
            "Running Corsa diagnostics for {} files...",
            virtual_files.len()
        );
    }

    let check_start = Instant::now();
    let result = match checker.check_project() {
        Ok(result) => result,
        Err(error) => {
            eprintln!("\x1b[31mError:\x1b[0m {}", error);
            std::process::exit(1);
        }
    };
    let check_time = check_start.elapsed();

    let emit_start = Instant::now();
    let emitted_declarations = if args.declaration {
        let declaration_options = resolve_declaration_emit_options(
            args.declaration_dir.as_deref(),
            program_tsconfig_path.as_deref(),
            &project_root,
        );
        let declaration_dir = declaration_options.out_dir.clone();
        match checker.emit_declarations(&declaration_options) {
            Ok(result) => Some((declaration_dir, result)),
            Err(error) => {
                eprintln!("\x1b[31mError:\x1b[0m {}", error);
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    let emit_time = emit_start.elapsed();
    let diagnostics_render_start = Instant::now();
    // Restrict diagnostics to the requested files for an explicit subset; the
    // ambient/transitive files were registered only to resolve cross-file types.
    let reported_raw = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            if !is_reported(&reported_files, &diagnostic.file, &mut canonical_paths) {
                return false;
            }
            !is_suppressed_false_positive(diagnostic)
        })
        .cloned()
        .collect::<Vec<_>>();
    let diagnostics = render_diagnostics(&reported_raw);
    let diagnostics_render_time = diagnostics_render_start.elapsed();
    let total_time = start.elapsed();
    let total_errors = reported_raw
        .iter()
        .filter(|diagnostic| diagnostic.severity == 1)
        .count();
    let total_warnings = reported_raw
        .iter()
        .filter(|diagnostic| diagnostic.severity == 2)
        .count();

    if args.profile {
        let profiler = global_profiler();
        let allocation_summary = profile_support::allocation_snapshot();
        let counter_summary = profiler.counter_summary();
        let operation_summary = profiler.summary();
        profiler.disable();
        let mut phases = vec![
            ProfilePhase {
                name: "collect inputs",
                duration: collect_time,
                kind: ProfilePhaseKind::Wall,
                note: "tsconfig or explicit patterns",
            },
            ProfilePhase {
                name: "virtual project",
                duration: gen_time,
                kind: ProfilePhaseKind::Wall,
                note: "scan paths and generate Virtual TS",
            },
            ProfilePhase {
                name: "profile artifacts",
                duration: profile_artifact_time,
                kind: ProfilePhaseKind::Wall,
                note: "write node_modules/.vize/check-profile",
            },
            ProfilePhase {
                name: "corsa diagnostics",
                duration: check_time,
                kind: ProfilePhaseKind::Wall,
                note: "project-session diagnostics",
            },
            ProfilePhase {
                name: "render diagnostics",
                duration: diagnostics_render_time,
                kind: ProfilePhaseKind::Wall,
                note: "group diagnostics by file",
            },
        ];
        if args.declaration {
            phases.push(ProfilePhase {
                name: "declaration emit",
                duration: emit_time,
                kind: ProfilePhaseKind::Wall,
                note: "materialized Corsa project",
            });
        }

        let virtual_bytes = virtual_files
            .iter()
            .fold(0usize, |acc, file| acc + file.content.len());
        let mut recommendations: Vec<String> = Vec::new();
        if check_time > gen_time * 2 {
            recommendations.push(
                "Corsa diagnostics dominate; keep the generated Virtual TS directory and inspect the largest generated files."
                    .into(),
            );
        } else if gen_time > check_time {
            recommendations.push(
                "Virtual TS generation dominates; inspect SFCs with large templates, macros, or cross-file imports."
                    .into(),
            );
        }
        if let Some(largest) = virtual_files.iter().max_by_key(|file| file.content.len()) {
            recommendations.push(cstr!(
                "Largest Virtual TS: {} ({} bytes).",
                largest.original_path.display(),
                largest.content.len()
            ));
        }

        let summary = cstr!(
            "{} virtual file(s), {} error(s), project {}",
            virtual_files.len(),
            total_errors,
            project_root.display()
        );
        let report = ProfileReport {
            title: "check",
            summary: summary.as_str(),
            total: total_time,
            phases: &phases,
            files: &[],
            slow_threshold: Duration::from_millis(0),
            throughput_bytes: Some(virtual_bytes),
            operations: Some(&operation_summary),
            counters: Some(&counter_summary),
            allocations: allocation_summary,
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }

    if args.format == "json" {
        let mut files_json: Vec<JsonFileResult> = virtual_files
            .iter()
            .filter(|file| is_reported(&reported_files, &file.original_path, &mut canonical_paths))
            .map(|file| {
                let key = file.original_path.to_string_lossy().into_owned();
                JsonFileResult {
                    file: display_path(&cwd, &file.original_path).into(),
                    virtual_ts: file.content.clone().into(),
                    diagnostics: diagnostics.get(key.as_str()).cloned().unwrap_or_default(),
                }
            })
            .collect();
        files_json.sort_by(|left, right| left.file.cmp(&right.file));
        // `fileCount` counts checked source files; project-level diagnostic
        // groups (anchored to a tsconfig) are appended to `files` afterwards
        // so every counted error appears in the output, but they are not
        // checked files themselves.
        let reported_file_count = files_json.len();

        let virtual_keys: FxHashSet<String> = virtual_files
            .iter()
            .map(|file| String::from(file.original_path.to_string_lossy()))
            .collect();
        let mut project_level: Vec<JsonFileResult> = diagnostics
            .iter()
            .filter(|(key, file_diagnostics)| {
                !file_diagnostics.is_empty() && !virtual_keys.contains(key.as_str())
            })
            .map(|(key, file_diagnostics)| JsonFileResult {
                file: display_path(&cwd, Path::new(key)).into(),
                virtual_ts: "".into(),
                diagnostics: file_diagnostics.clone(),
            })
            .collect();
        files_json.append(&mut project_level);

        let declarations = emitted_declarations.as_ref().map(|(_, result)| {
            result
                .files
                .iter()
                .map(|file| display_path(&cwd, &file.path).into())
                .collect()
        });

        let json_output = JsonOutput {
            files: files_json,
            error_count: total_errors,
            warning_count: total_warnings,
            file_count: reported_file_count,
            declarations,
        };
        emit_json_output(json_output);
        if total_errors > 0 {
            std::process::exit(1);
        }
        return;
    }

    if !args.quiet {
        let mut printed_keys: FxHashSet<String> = FxHashSet::default();
        for file in checker.virtual_files() {
            let key = file.original_path.to_string_lossy();
            printed_keys.insert(String::from(key.as_ref()));
            let Some(file_diagnostics) = diagnostics.get(key.as_ref()) else {
                continue;
            };
            if file_diagnostics.is_empty() {
                continue;
            }

            println!("\n\x1b[4m{}\x1b[0m", key);
            for diagnostic in file_diagnostics {
                let color = if diagnostic.starts_with("error") {
                    "\x1b[31m"
                } else {
                    "\x1b[33m"
                };
                println!("  {}{}\x1b[0m", color, diagnostic);
            }
        }

        // Project-level diagnostics (anchored to a tsconfig, not a checked
        // source file) — print after the per-file groups.
        for (key, file_diagnostics) in &diagnostics {
            if file_diagnostics.is_empty() || printed_keys.contains(key.as_str()) {
                continue;
            }
            println!("\n\x1b[4m{}\x1b[0m", key);
            for diagnostic in file_diagnostics {
                let color = if diagnostic.starts_with("error") {
                    "\x1b[31m"
                } else {
                    "\x1b[33m"
                };
                println!("  {}{}\x1b[0m", color, diagnostic);
            }
        }
    }

    let status = if total_errors > 0 {
        "\x1b[31m\u{2717}\x1b[0m"
    } else {
        "\x1b[32m\u{2713}\x1b[0m"
    };
    if emitted_declarations.is_some() {
        println!(
            "\n{} Type checked {} files in {:.2?} (collect: {:.2?}, gen: {:.2?}, corsa: {:.2?}, dts: {:.2?})",
            status,
            virtual_files.len(),
            total_time,
            collect_time,
            gen_time,
            check_time,
            emit_time
        );
    } else {
        println!(
            "\n{} Type checked {} files in {:.2?} (collect: {:.2?}, gen: {:.2?}, corsa: {:.2?})",
            status,
            virtual_files.len(),
            total_time,
            collect_time,
            gen_time,
            check_time
        );
    }

    if total_errors > 0 {
        println!("  \x1b[31m{} error(s)\x1b[0m", total_errors);
    } else {
        println!("  \x1b[32mNo type errors found!\x1b[0m");
    }
    if total_warnings > 0 {
        println!("  \x1b[33m{} warning(s)\x1b[0m", total_warnings);
    }

    if let Some((declaration_dir, emit_result)) = emitted_declarations {
        println!(
            "  \x1b[32mEmitted {} declaration file(s)\x1b[0m to {}",
            emit_result.files.len(),
            declaration_dir.display()
        );
    }

    if total_errors > 0 {
        std::process::exit(1);
    }
    if let Some(max_warnings) = args.max_warnings
        && total_warnings > max_warnings
    {
        eprintln!("\nToo many warnings ({total_warnings} > max {max_warnings})");
        std::process::exit(1);
    }
}
