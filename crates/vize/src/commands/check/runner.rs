//! Check command execution logic.
//!
//! The direct runner delegates to `vize_canon`'s project-backed Corsa type
//! checker so Vue SFCs, TypeScript sources, ambient declarations, and emitted
//! `.d.ts` output all share the same virtual project.

#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, DeclarationEmitOptions,
    batch::TypeChecker as BatchTypeCheckerTrait,
};
use vize_carton::{
    FxHashSet, String, cstr, profile,
    profiler::{allocation_snapshot, global_profiler},
};

use vize_curator::profile::{ProfilePhase, ProfilePhaseKind, ProfileReport, print_profile_report};

use super::{
    CheckArgs,
    reporting::{JsonFileResult, JsonOutput},
    tsconfig_inputs::{
        TsconfigDeclarationOptions, collect_ambient_declaration_files, collect_default_check_files,
        load_tsconfig_declaration_options, resolve_tsconfig_for_files,
    },
};

mod collect;
#[cfg(unix)]
mod socket;

use collect::collect_check_files;
#[cfg(unix)]
pub(crate) use socket::run_with_socket;

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
    let legacy_vue2 = loaded_config.features.type_checker_legacy_vue2;
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
    if let Err(error) = validate_corsa_server_count(args.servers.or(config.type_checker.servers)) {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }
    let project_root = resolve_project_root(effective_tsconfig.as_deref(), &cwd, &[]);
    let tsconfig_path =
        resolve_tsconfig_path(effective_tsconfig.as_deref(), &cwd, &project_root, &[]);
    let collect_start = Instant::now();
    let mut files = if args.patterns.is_empty() {
        collect_default_check_files(&project_root, tsconfig_path.as_deref())
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
                .map(|path| path.canonicalize().unwrap_or_else(|_| path.clone()))
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
        for path in super::imports::collect_transitive_local_imports(&files, &cwd) {
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
        resolve_tsconfig_for_files(tsconfig_path.as_deref(), &explicit_files)
    };

    // An explicit file subset (`vize check src/App.vue`) omits ambient
    // declaration files, since nothing imports them; `declare global` types
    // would then be missing and surface as false `TS2304` errors. Pull the
    // tsconfig program's `.d.ts` files back in so global types stay in scope.
    if !args.patterns.is_empty() && program_tsconfig_path.is_some() {
        for path in
            collect_ambient_declaration_files(&project_root, program_tsconfig_path.as_deref())
        {
            if !files.contains(&path) {
                files.push(path);
            }
        }
        files.sort();
        files.dedup();
    }

    let mut virtual_ts_options = build_virtual_ts_options(&config, config_dir);
    nuxt::detect_nuxt_auto_imports(&mut virtual_ts_options, &project_root);

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
            tsconfig_path: program_tsconfig_path.clone(),
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
    if legacy_vue2 {
        checker.enable_legacy_vue2();
    }
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
        for file in &virtual_files {
            eprintln!("\n=== {} ===", file.original_path.display());
            eprintln!("{}", file.content);
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
            if !is_reported(&reported_files, &diagnostic.file) {
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
        let allocation_summary = allocation_snapshot();
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
            allocations: Some(allocation_summary),
            recommendations: &recommendations,
        };
        print_profile_report(&report);
    }

    if args.format == "json" {
        let mut files_json: Vec<JsonFileResult> = virtual_files
            .iter()
            .filter(|file| is_reported(&reported_files, &file.original_path))
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
        let reported_file_count = files_json.len();

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
        for file in checker.virtual_files() {
            let key = file.original_path.to_string_lossy();
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

fn emit_json_output(json_output: JsonOutput) {
    match serde_json::to_string_pretty(&json_output) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("Failed to serialize check output: {error}");
            std::process::exit(1);
        }
    }
}

/// Whether a registered file's diagnostics should be reported. For an explicit
/// subset (`reported` is `Some`), only the requested files are reported; ambient
/// and transitively-registered files exist only to resolve cross-file types.
fn is_reported(reported: &Option<FxHashSet<PathBuf>>, path: &Path) -> bool {
    match reported {
        None => true,
        Some(set) => {
            let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            set.contains(&canonical)
        }
    }
}

fn is_suppressed_false_positive(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2320)
        && diagnostic
            .message
            .contains("Interface 'ImportMeta' cannot simultaneously extend types")
        && diagnostic.message.contains("NitroStaticBuildFlags")
        && diagnostic.message.contains("NitroImportMeta")
}

#[allow(clippy::disallowed_types)]
fn render_diagnostics(
    diagnostics: &[vize_canon::BatchDiagnostic],
) -> std::collections::BTreeMap<std::string::String, Vec<std::string::String>> {
    let mut grouped = std::collections::BTreeMap::<
        std::string::String,
        Vec<(u32, u32, std::string::String)>,
    >::new();

    for diagnostic in diagnostics {
        let severity = match diagnostic.severity {
            1 => "error",
            2 => "warning",
            3 => "info",
            _ => "hint",
        };
        let code = diagnostic
            .code
            .map(|code| cstr!(" [TS{}]", code))
            .unwrap_or_default();
        let rendered = cstr!(
            "{}:{}:{}{} {}",
            severity,
            diagnostic.line + 1,
            diagnostic.column + 1,
            code,
            diagnostic.message
        )
        .into();
        grouped
            .entry(diagnostic.file.to_string_lossy().into_owned())
            .or_default()
            .push((diagnostic.line, diagnostic.column, rendered));
    }

    grouped
        .into_iter()
        .map(|(file, mut diagnostics)| {
            diagnostics.sort_by(|left, right| {
                left.0
                    .cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
                    .then_with(|| left.2.cmp(&right.2))
            });
            let rendered = diagnostics
                .into_iter()
                .map(|(_, _, rendered)| rendered)
                .collect();
            (file, rendered)
        })
        .collect()
}

fn write_profile_virtual_ts(files: &[&vize_canon::VirtualFile]) {
    let profile_dir = PathBuf::from("node_modules/.vize/check-profile");
    if profile_dir.exists() {
        match profile!(
            "cli.check.profile_artifact.remove_dir_all",
            fs::remove_dir_all(&profile_dir)
        ) {
            Ok(()) => global_profiler().record_fs_remove_dir_all(),
            Err(error) => {
                global_profiler().record_fs_remove_dir_all_failure();
                eprintln!(
                    "Failed to clean profile directory {}: {}",
                    profile_dir.display(),
                    error
                );
                return;
            }
        }
    }

    match profile!(
        "cli.check.profile_artifact.create_dir_all",
        fs::create_dir_all(&profile_dir)
    ) {
        Ok(()) => global_profiler().record_fs_create_dir_all(),
        Err(error) => {
            global_profiler().record_fs_create_dir_all_failure();
            eprintln!("Failed to create profile directory: {}", error);
            return;
        }
    }

    for file in files {
        let file_name = file
            .original_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| cstr!("{name}.ts"))
            .unwrap_or_else(|| "unknown.ts".into());
        let target = profile_dir.join(file_name.as_str());
        let bytes = file.content.len();
        match profile!(
            "cli.check.profile_artifact.write",
            fs::write(&target, &file.content)
        ) {
            Ok(()) => global_profiler().record_fs_write(bytes),
            Err(error) => {
                global_profiler().record_fs_write_failure(bytes);
                eprintln!("Failed to write {}: {}", target.display(), error);
            }
        }
    }

    eprintln!(
        "\x1b[33mProfile:\x1b[0m Virtual TS files written to {}",
        profile_dir.display()
    );
}

fn build_virtual_ts_options(
    config: &crate::config::VizeConfig,
    config_dir: &Path,
) -> vize_canon::virtual_ts::VirtualTsOptions {
    let mut template_globals = config
        .global_types
        .iter()
        .map(
            |(name, declaration)| vize_canon::virtual_ts::TemplateGlobal {
                name: name.clone(),
                type_annotation: declaration.type_annotation.clone(),
                default_value: declaration.template_default_value(),
            },
        )
        .collect::<Vec<_>>();

    let globals_path = config
        .type_checker
        .globals_file
        .as_deref()
        .map(|candidate| resolve_from_config_dir(config_dir, candidate));

    if template_globals.is_empty()
        && let Some(ref globals_path) = globals_path
    {
        match parse_dts_globals(globals_path) {
            Ok(globals) => template_globals = globals,
            Err(error) => {
                eprintln!(
                    "\x1b[33mWarning:\x1b[0m Failed to parse globals from {}: {}",
                    globals_path.display(),
                    error
                );
            }
        }
    }

    vize_canon::virtual_ts::VirtualTsOptions {
        template_globals,
        ..Default::default()
    }
}

fn resolve_declaration_emit_options(
    declaration_dir: Option<&Path>,
    tsconfig_path: Option<&Path>,
    project_root: &Path,
) -> DeclarationEmitOptions {
    let tsconfig_options = tsconfig_path
        .map(load_tsconfig_declaration_options)
        .unwrap_or_default();
    let out_dir = resolve_declaration_dir(declaration_dir, &tsconfig_options, project_root);

    DeclarationEmitOptions::new(out_dir)
        .with_declaration_map(tsconfig_options.declaration_map.unwrap_or(false))
}

fn resolve_declaration_dir(
    declaration_dir: Option<&Path>,
    tsconfig_options: &TsconfigDeclarationOptions,
    project_root: &Path,
) -> PathBuf {
    declaration_dir
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                project_root.join(path)
            }
        })
        .or_else(|| tsconfig_options.output_dir().map(Path::to_path_buf))
        .unwrap_or_else(|| project_root.join("dist").join("types"))
}

fn resolve_project_root(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    files: &[PathBuf],
) -> PathBuf {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        let tsconfig_dir = tsconfig_path
            .canonicalize()
            .unwrap_or(tsconfig_path)
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| cwd.to_path_buf());
        if files.is_empty() {
            return tsconfig_dir;
        }

        return common_project_root(tsconfig_dir, files);
    }

    if let Some(root) = resolve_project_root_from_files(files) {
        return root;
    }

    if let Some(root) = find_nearest_tsconfig_dir(cwd) {
        return root;
    }

    cwd.to_path_buf()
}

fn resolve_tsconfig_path(
    explicit_tsconfig: Option<&Path>,
    cwd: &Path,
    project_root: &Path,
    files: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(tsconfig) = explicit_tsconfig {
        let tsconfig_path = if tsconfig.is_absolute() {
            tsconfig.to_path_buf()
        } else {
            cwd.join(tsconfig)
        };
        return Some(tsconfig_path.canonicalize().unwrap_or(tsconfig_path));
    }

    let candidate = project_root.join("tsconfig.json");
    if candidate.exists() {
        return Some(candidate);
    }

    for file in files {
        let Some(root) = find_nearest_tsconfig_dir(file) else {
            continue;
        };
        let candidate = root.join("tsconfig.json");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn find_nearest_tsconfig_dir(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        if dir.join("tsconfig.json").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

fn resolve_project_root_from_files(files: &[PathBuf]) -> Option<PathBuf> {
    let common = common_file_parent(files)?;
    Some(find_nearest_tsconfig_dir(&common).unwrap_or(common))
}

fn common_file_parent(files: &[PathBuf]) -> Option<PathBuf> {
    let mut common = files
        .first()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)?;

    for file in &files[1..] {
        let parent = file.parent().unwrap_or(file.as_path());
        while !parent.starts_with(&common) {
            if !common.pop() {
                return None;
            }
        }
    }

    Some(common)
}

fn common_project_root(mut common: PathBuf, files: &[PathBuf]) -> PathBuf {
    for file in files {
        let parent = file.parent().unwrap_or(file.as_path());
        while !parent.starts_with(&common) {
            if !common.pop() {
                return common;
            }
        }
    }

    common
}

fn display_path(base: &Path, path: &Path) -> vize_carton::String {
    path.strip_prefix(base)
        .map(|relative| cstr!("{}", relative.display()))
        .unwrap_or_else(|_| cstr!("{}", path.display()))
}

fn resolve_from_config_dir(config_dir: &Path, candidate: &str) -> PathBuf {
    let path = Path::new(candidate);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    config_dir.join(path)
}

fn validate_corsa_server_count(servers: Option<usize>) -> Result<(), String> {
    let Some(servers) = servers else {
        return Ok(());
    };
    if servers == 0 {
        return Err("typeChecker.servers must be at least 1.".into());
    }
    if servers > 1 {
        return Err(format!(
            "typeChecker.servers={servers} is not supported by the direct Corsa project-session runner yet; use 1 or omit the option."
        )
        .into());
    }
    Ok(())
}

/// Parse a `.d.ts` file containing `ComponentCustomProperties` augmentation.
fn parse_dts_globals(
    path: &Path,
) -> Result<Vec<vize_canon::virtual_ts::TemplateGlobal>, std::io::Error> {
    use super::dts::parse_interface_members;
    use vize_canon::virtual_ts::TemplateGlobal;

    Ok(
        parse_interface_members(path, "interface ComponentCustomProperties")?
            .into_iter()
            .map(|(name, type_annotation)| TemplateGlobal {
                name,
                type_annotation,
                default_value: "{} as any".into(),
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        find_nearest_tsconfig_dir, is_suppressed_false_positive, resolve_declaration_dir,
        resolve_declaration_emit_options, resolve_project_root, resolve_tsconfig_path,
        validate_corsa_server_count,
    };
    use crate::commands::check::tsconfig_inputs::TsconfigDeclarationOptions;
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

        let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(format!(
                "check-runner-{name}-{}-{case_id}",
                std::process::id()
            ))
    }

    #[test]
    fn suppresses_nuxt_nitro_import_meta_conflict_false_positive() {
        let diagnostic = vize_canon::BatchDiagnostic {
            file: PathBuf::from("app/app.vue"),
            line: 0,
            column: 0,
            message: "Interface 'ImportMeta' cannot simultaneously extend types 'NitroStaticBuildFlags' and 'NitroImportMeta'.\nNamed property 'preset' of types 'NitroStaticBuildFlags' and 'NitroImportMeta' are not identical.".into(),
            code: Some(2320),
            severity: 1,
            block_type: None,
        };

        assert!(is_suppressed_false_positive(&diagnostic));

        let mut unrelated = diagnostic.clone();
        unrelated.message = "Interface 'Other' cannot simultaneously extend types".into();
        assert!(!is_suppressed_false_positive(&unrelated));
    }

    #[test]
    fn resolves_monorepo_root_for_files_spanning_package_tsconfigs() {
        let project_root = unique_case_dir("monorepo-root");
        let _ = std::fs::remove_dir_all(&project_root);
        let app_dir = project_root.join("packages/app");
        let ui_dir = project_root.join("packages/ui");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::create_dir_all(ui_dir.join("src")).unwrap();
        std::fs::write(project_root.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(app_dir.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(ui_dir.join("tsconfig.json"), "{}").unwrap();
        let files = vec![app_dir.join("src/App.vue"), ui_dir.join("src/UiButton.vue")];
        for file in &files {
            std::fs::write(file, "<template />").unwrap();
        }

        let resolved_root = resolve_project_root(None, &project_root, &files);
        let resolved_tsconfig = resolve_tsconfig_path(None, &project_root, &resolved_root, &files);

        assert_eq!(resolved_root, project_root);
        assert_eq!(resolved_tsconfig, Some(resolved_root.join("tsconfig.json")));

        let _ = std::fs::remove_dir_all(&resolved_root);
    }

    #[test]
    fn resolves_package_root_for_files_inside_one_package() {
        let project_root = unique_case_dir("package-root");
        let _ = std::fs::remove_dir_all(&project_root);
        let app_dir = project_root.join("packages/app");
        std::fs::create_dir_all(app_dir.join("src")).unwrap();
        std::fs::write(project_root.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(app_dir.join("tsconfig.json"), "{}").unwrap();
        let files = vec![app_dir.join("src/App.vue"), app_dir.join("src/main.ts")];
        for file in &files {
            std::fs::write(file, "").unwrap();
        }

        let resolved_root = resolve_project_root(None, &project_root, &files);
        let resolved_tsconfig = resolve_tsconfig_path(None, &project_root, &resolved_root, &files);

        assert_eq!(resolved_root, app_dir);
        assert_eq!(resolved_tsconfig, Some(resolved_root.join("tsconfig.json")));

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn resolves_common_root_when_explicit_tsconfig_is_below_inputs() {
        let project_root = unique_case_dir("explicit-tsconfig-below-inputs");
        let _ = std::fs::remove_dir_all(&project_root);
        let config_dir = project_root.join("config");
        let src_dir = project_root.join("src");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&src_dir).unwrap();
        let tsconfig = config_dir.join("tsconfig.json");
        let app = src_dir.join("App.vue");
        std::fs::write(&tsconfig, "{}").unwrap();
        std::fs::write(&app, "<template />").unwrap();
        let files = vec![app];

        let resolved_root = resolve_project_root(Some(&tsconfig), &project_root, &files);
        let resolved_tsconfig =
            resolve_tsconfig_path(Some(&tsconfig), &project_root, &resolved_root, &files);

        assert_eq!(resolved_root, project_root);
        assert_eq!(resolved_tsconfig, Some(tsconfig));

        let _ = std::fs::remove_dir_all(&resolved_root);
    }

    #[test]
    fn falls_back_to_cwd_resolution_when_files_have_no_tsconfig() {
        let project_root = unique_case_dir("no-tsconfig");
        let _ = std::fs::remove_dir_all(&project_root);
        let src_dir = project_root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let files = vec![src_dir.join("App.vue")];
        std::fs::write(&files[0], "<template />").unwrap();

        let resolved_root = resolve_project_root(None, &project_root, &files);
        let resolved_tsconfig = resolve_tsconfig_path(None, &project_root, &resolved_root, &files);
        let expected_root =
            find_nearest_tsconfig_dir(&project_root).unwrap_or_else(|| project_root.clone());

        assert_eq!(resolved_root, expected_root);
        assert_eq!(
            resolved_tsconfig,
            resolved_root
                .join("tsconfig.json")
                .exists()
                .then_some(resolved_root.join("tsconfig.json"))
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn falls_back_to_common_file_parent_for_external_files_without_tsconfig() {
        let case_root = std::env::temp_dir().join(format!(
            "vize-check-runner-external-root-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&case_root);
        let cwd = case_root.join("cwd");
        let source_dir = case_root.join("external");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(&source_dir).unwrap();
        let files = vec![source_dir.join("Repro.vue")];
        std::fs::write(&files[0], "<template />").unwrap();

        let resolved_root = resolve_project_root(None, &cwd, &files);
        let resolved_tsconfig = resolve_tsconfig_path(None, &cwd, &resolved_root, &files);

        assert_eq!(resolved_root, source_dir);
        assert_eq!(resolved_tsconfig, None);

        let _ = std::fs::remove_dir_all(&case_root);
    }

    #[test]
    fn resolve_declaration_dir_defaults_to_dist_types() {
        let project_root = PathBuf::from("/workspace/project");
        let tsconfig_options = TsconfigDeclarationOptions::default();
        assert_eq!(
            resolve_declaration_dir(None, &tsconfig_options, &project_root),
            project_root.join("dist").join("types")
        );
        assert_eq!(
            resolve_declaration_dir(Some(Path::new("types")), &tsconfig_options, &project_root),
            project_root.join("types")
        );
    }

    #[test]
    fn resolve_declaration_dir_uses_tsconfig_when_cli_dir_is_absent() {
        let project_root = PathBuf::from("/workspace/project");
        let tsconfig_options = TsconfigDeclarationOptions {
            declaration_dir: Some(project_root.join("types")),
            out_dir: Some(project_root.join("dist")),
            declaration_map: Some(true),
        };

        assert_eq!(
            resolve_declaration_dir(None, &tsconfig_options, &project_root),
            project_root.join("types")
        );
        assert_eq!(
            resolve_declaration_dir(Some(Path::new("custom")), &tsconfig_options, &project_root),
            project_root.join("custom")
        );

        let out_dir_only = TsconfigDeclarationOptions {
            declaration_dir: None,
            out_dir: Some(project_root.join("dist")),
            declaration_map: None,
        };
        assert_eq!(
            resolve_declaration_dir(None, &out_dir_only, &project_root),
            project_root.join("dist")
        );
    }

    #[test]
    fn resolve_declaration_emit_options_uses_tsconfig_declaration_map() {
        let project_root = unique_case_dir("declaration-options");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(&project_root).unwrap();
        std::fs::write(
            project_root.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "declarationDir": "types",
    "declarationMap": true
  }
}"#,
        )
        .unwrap();

        let options = resolve_declaration_emit_options(
            None,
            Some(&project_root.join("tsconfig.json")),
            &project_root,
        );

        assert_eq!(options.out_dir, project_root.join("types"));
        assert!(options.declaration_map);

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn validates_unsupported_corsa_server_counts() {
        assert!(validate_corsa_server_count(None).is_ok());
        assert!(validate_corsa_server_count(Some(1)).is_ok());
        assert!(validate_corsa_server_count(Some(0)).is_err());
        assert!(validate_corsa_server_count(Some(2)).is_err());
    }
}
