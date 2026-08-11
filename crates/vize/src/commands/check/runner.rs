//! Check command execution logic.
//! The direct runner materializes one Corsa program per owning tsconfig so
//! project references retain their own compiler options.

#![allow(clippy::disallowed_macros)]

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use vize_carton::{FxHashSet, profiler::global_profiler};

use super::{
    CheckArgs,
    imports::ImportFileOptions,
    path_cache::CanonicalPathCache,
    patterns::CheckFileOptions,
    reporting::{JsonFileResult, JsonOutput},
    tsconfig_inputs::{TsconfigInputCache, resolve_tsconfig_program_inputs, tsconfig_allows_js},
};

mod collect;
mod default_imports;
mod diagnostics;
mod execution;
mod global_components;
mod ignores;
mod input_scope;
mod invocation;
mod nuxt_tsconfig;
mod output;
mod program_inputs;
mod resolve;
#[cfg(unix)]
mod socket;
#[cfg(test)]
mod tests;

use collect::collect_check_files_with_ignores;
use default_imports::{
    ExplicitAmbientImportContext, canonical_file_set, collect_default_run_files,
    register_ambient_declaration_files, register_explicit_ambient_imports,
    register_transitive_local_imports,
};
#[cfg(test)]
use diagnostics::is_suppressed_false_positive;
use execution::{CheckerSettings, ProgramExecution, ProgramExecutionInput, execute_program};
use global_components::{
    build_virtual_ts_options, collect_project_global_component_stubs, dialect_from_features,
    template_syntax_mode,
};
use ignores::load_check_ignore_set;
use input_scope::{exit_if_default_run_leaves_cwd, report_no_inputs};
use invocation::{resolve_invocation_program, resolve_nuxt_project_root};
use nuxt_tsconfig::resolve_checker_tsconfig_path;
use output::{exit_after_execution_error, finish_executions};
use program_inputs::{ProgramInputContext, filter_for_program, project_graph_allows_js};
use resolve::{
    display_path, explicit_input_root, resolve_declaration_emit_options, resolve_from_config_dir,
    resolve_project_root, resolve_tsconfig_path, validate_corsa_server_count,
    validate_inputs_in_root,
};
#[cfg(test)]
use resolve::{find_nearest_tsconfig_dir, resolve_declaration_dir};
#[cfg(unix)]
pub(crate) use socket::run_with_socket;

struct ProgramCandidate {
    files: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    reported: FxHashSet<PathBuf>,
    virtual_module_aliases: Vec<(vize_carton::String, PathBuf)>,
    tsconfig_path: Option<PathBuf>,
    rebuild_supporting_files: bool,
}

struct CollectedRoots {
    files: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    reported: FxHashSet<PathBuf>,
    virtual_module_aliases: Vec<(vize_carton::String, PathBuf)>,
}

/// Run type checking directly with materialized Corsa projects.
pub(crate) fn run_direct(args: &CheckArgs) {
    let start = Instant::now();
    if args.profile {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
    }
    crate::config::write_schema(None);
    validate_config_arg(args);

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
    let dialect = dialect_from_features(loaded_config.features.vue_version);
    let options_api = loaded_config.features.type_checker_options_api;
    let jsx_typecheck = loaded_config.features.type_checker_jsx_typecheck;
    let legacy_vue2 = cfg!(feature = "legacy") && loaded_config.features.type_checker_legacy_vue2;
    warn_for_disabled_legacy(loaded_config.features.type_checker_legacy_vue2);

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
    let effective_tsconfig = args.tsconfig.clone().or_else(|| {
        config
            .type_checker
            .tsconfig
            .as_deref()
            .map(|path| resolve_from_config_dir(config_dir, path))
    });
    let effective_corsa_path = args.corsa_path.as_ref().map(PathBuf::from).or_else(|| {
        config
            .type_checker
            .runtime_path()
            .map(|path| resolve_from_config_dir(config_dir, path))
    });
    let corsa_servers = args.servers.or(config.type_checker.servers);
    if let Err(error) = validate_corsa_server_count(corsa_servers) {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }

    let (invocation_project_root, invocation_tsconfig_path) =
        resolve_invocation_program(effective_tsconfig.as_deref(), &cwd);
    let nuxt_project_root = resolve_nuxt_project_root(
        effective_tsconfig.as_deref(),
        &cwd,
        &invocation_project_root,
    );
    let explicit_input_root = explicit_input_root(&invocation_project_root, &cwd);
    let mut tsconfig_input_cache = TsconfigInputCache::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let check_ignore_set = load_check_ignore_set(args, config_dir);
    let collect_start = Instant::now();
    let collected = collect_roots(
        args,
        &invocation_project_root,
        &cwd,
        invocation_tsconfig_path.as_deref(),
        jsx_typecheck,
        &mut tsconfig_input_cache,
        &mut canonical_paths,
        check_ignore_set.as_ref(),
    );
    let collect_time = collect_start.elapsed();
    if collected.files.is_empty() {
        report_no_inputs(args);
        return;
    }

    let candidates = split_program_candidates(
        collected,
        invocation_tsconfig_path.as_deref(),
        jsx_typecheck,
        &mut tsconfig_input_cache,
        &mut canonical_paths,
    );
    let settings = CheckerSettings {
        virtual_ts_options: build_virtual_ts_options(&config, config_dir),
        corsa_path: effective_corsa_path,
        servers: corsa_servers,
        options_api,
        legacy_vue2,
        jsx_typecheck,
        template_syntax: template_syntax_mode(compiler_template_syntax),
        experimental_in_tag_comments: loaded_config.features.experimental_in_tag_comments,
        dialect,
        check_props: config.type_checker.check_props && !args.no_check_props,
        check_template_bindings: config.type_checker.check_template_bindings
            && !args.no_check_template_bindings,
        check_emits: config.type_checker.check_emits && !args.no_check_emits,
        quiet: args.quiet,
    };
    let validate_inputs = !args.patterns.is_empty() && invocation_tsconfig_path.is_some();
    let mut executions = Vec::new();
    for candidate in candidates {
        let execution = match prepare_and_execute(
            args,
            candidate,
            &cwd,
            &invocation_project_root,
            &nuxt_project_root,
            &explicit_input_root,
            validate_inputs,
            jsx_typecheck,
            &settings,
            &mut tsconfig_input_cache,
            &mut canonical_paths,
        ) {
            Ok(execution) => execution,
            Err(error) => exit_after_execution_error(executions, error),
        };
        if let Some(execution) = execution {
            executions.push(execution);
        }
    }
    if executions.is_empty() {
        report_no_inputs(args);
        return;
    }
    finish_executions(
        args,
        &cwd,
        start,
        collect_time,
        executions,
        &mut canonical_paths,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_roots(
    args: &CheckArgs,
    invocation_project_root: &Path,
    cwd: &Path,
    invocation_tsconfig_path: Option<&Path>,
    jsx_typecheck: bool,
    cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
    check_ignore_set: Option<&ignores::CheckIgnoreSet>,
) -> CollectedRoots {
    let include_js = project_graph_allows_js(invocation_tsconfig_path, cache);
    let import_options = ImportFileOptions {
        include_js,
        include_jsx: jsx_typecheck,
    };
    if args.patterns.is_empty() {
        let collected = collect_default_run_files(
            invocation_project_root,
            cwd,
            invocation_tsconfig_path,
            import_options,
            cache,
            canonical_paths,
            check_ignore_set,
        );
        exit_if_default_run_leaves_cwd(
            &collected.files,
            cwd,
            invocation_project_root,
            invocation_tsconfig_path,
            args.quiet,
        );
        return collected;
    }

    let files = collect_check_files_with_ignores(
        &args.patterns,
        CheckFileOptions {
            include_js,
            include_jsx: jsx_typecheck,
        },
        check_ignore_set,
    );
    let reported = canonical_file_set(&files, canonical_paths);
    CollectedRoots {
        files: files.clone(),
        inputs: files,
        reported,
        virtual_module_aliases: Vec::new(),
    }
}

fn split_program_candidates(
    collected: CollectedRoots,
    tsconfig_path: Option<&Path>,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
) -> Vec<ProgramCandidate> {
    let CollectedRoots {
        files,
        inputs,
        reported,
        virtual_module_aliases,
    } = collected;
    let groups = resolve_tsconfig_program_inputs(tsconfig_path, &inputs, include_jsx, cache);
    let single_group_uses_invocation_config = groups.first().is_none_or(|group| {
        tsconfig_path.is_none_or(|path| {
            canonical_paths.canonicalize(path) == canonical_paths.canonicalize(&group.tsconfig_path)
        })
    });
    if groups.len() <= 1 && single_group_uses_invocation_config {
        return vec![ProgramCandidate {
            files,
            inputs,
            reported,
            virtual_module_aliases,
            tsconfig_path: groups
                .first()
                .map(|group| group.tsconfig_path.clone())
                .or_else(|| tsconfig_path.map(Path::to_path_buf)),
            rebuild_supporting_files: false,
        }];
    }

    // Each referenced program rebuilds its own reachable graph below; routes
    // collected from the solution shell must not leak across program scopes.
    drop(virtual_module_aliases);
    groups
        .into_iter()
        .map(|group| {
            let reported = canonical_file_set(&group.files, canonical_paths);
            ProgramCandidate {
                inputs: group.files.clone(),
                files: group.files,
                reported,
                virtual_module_aliases: Vec::new(),
                tsconfig_path: Some(group.tsconfig_path),
                rebuild_supporting_files: true,
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn prepare_and_execute(
    args: &CheckArgs,
    mut candidate: ProgramCandidate,
    cwd: &Path,
    invocation_project_root: &Path,
    nuxt_project_root: &Path,
    explicit_input_root: &Path,
    validate_inputs: bool,
    jsx_typecheck: bool,
    settings: &CheckerSettings,
    cache: &mut TsconfigInputCache,
    canonical_paths: &mut CanonicalPathCache,
) -> Result<Option<ProgramExecution>, vize_carton::String> {
    let initial_root = candidate
        .tsconfig_path
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(invocation_project_root);
    let logical_program_root = if candidate.rebuild_supporting_files {
        Some(initial_root.to_path_buf())
    } else if args.patterns.is_empty() {
        Some(invocation_project_root.to_path_buf())
    } else {
        None
    };
    let import_options = ImportFileOptions {
        include_js: candidate
            .tsconfig_path
            .as_deref()
            .is_some_and(|path| tsconfig_allows_js(path, cache)),
        include_jsx: jsx_typecheck,
    };
    let mut authored_imports = Vec::new();
    let mut virtual_module_aliases = std::mem::take(&mut candidate.virtual_module_aliases);
    if !args.patterns.is_empty() || candidate.rebuild_supporting_files {
        let discovered = register_transitive_local_imports(
            &mut candidate.files,
            cwd,
            candidate.tsconfig_path.as_deref(),
            import_options,
            canonical_paths,
            Some(explicit_input_root),
            validate_inputs,
        );
        authored_imports = discovered.authored;
        virtual_module_aliases.extend(discovered.virtual_module_aliases);
    }
    if args.patterns.is_empty() && candidate.rebuild_supporting_files {
        register_ambient_declaration_files(
            &mut candidate.files,
            initial_root,
            candidate.tsconfig_path.as_deref(),
            cache,
        );
        let discovered = register_transitive_local_imports(
            &mut candidate.files,
            cwd,
            candidate.tsconfig_path.as_deref(),
            import_options,
            canonical_paths,
            Some(explicit_input_root),
            validate_inputs,
        );
        virtual_module_aliases.extend(discovered.virtual_module_aliases);
    }
    validate_inputs_in_root(explicit_input_root, &candidate.files, validate_inputs)?;

    let project_root =
        resolve_project_root(candidate.tsconfig_path.as_deref(), cwd, &candidate.files);
    let discovered_tsconfig_path = candidate
        .tsconfig_path
        .clone()
        .or_else(|| resolve_tsconfig_path(None, cwd, &project_root, &candidate.files));
    let program_tsconfig_path = filter_for_program(ProgramInputContext {
        tsconfig_path: discovered_tsconfig_path.as_deref(),
        explicit: !args.patterns.is_empty(),
        include_jsx: jsx_typecheck,
        files: &mut candidate.files,
        inputs: &mut candidate.inputs,
        reported: &mut candidate.reported,
        cache,
        canonical_paths,
    });
    if !args.patterns.is_empty() && candidate.inputs.is_empty() {
        return Ok(None);
    }
    candidate.reported.extend(
        authored_imports
            .into_iter()
            .map(|path| canonical_paths.canonicalize(&path)),
    );
    if !args.patterns.is_empty()
        && let Some(program_tsconfig_path) = program_tsconfig_path.as_deref()
    {
        register_explicit_ambient_imports(
            &mut candidate.files,
            ExplicitAmbientImportContext::new(
                &project_root,
                cwd,
                program_tsconfig_path,
                explicit_input_root,
                import_options,
            ),
            cache,
            canonical_paths,
        );
    }
    let project_root =
        resolve_project_root(program_tsconfig_path.as_deref(), cwd, &candidate.files);
    let logical_program_root = logical_program_root.unwrap_or_else(|| project_root.clone());
    resolve::retain_project_files(&mut candidate.files, &project_root);
    if candidate.files.is_empty() {
        return Ok(None);
    }

    virtual_module_aliases.sort();
    virtual_module_aliases.dedup();
    execute_program(
        ProgramExecutionInput {
            files: &candidate.files,
            reported_files: candidate.reported,
            virtual_module_aliases: &virtual_module_aliases,
            project_root: &project_root,
            program_root: logical_program_root,
            tsconfig_path: program_tsconfig_path,
            nuxt_project_root,
        },
        settings,
    )
    .map(Some)
}

fn validate_config_arg(args: &CheckArgs) {
    if let Some(path) = args.config.as_deref()
        && !args.no_config
        && let Err(error) = crate::config::validate_explicit_config_path(path)
    {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(2);
    }
}

#[cfg(not(feature = "legacy"))]
fn warn_for_disabled_legacy(requested: bool) {
    if requested {
        eprintln!(
            "\x1b[33mwarning:\x1b[0m a Vue 2 dialect is configured (`typeChecker.legacyVue2` \
             or `vue.version` 2/2.7) but this `vize` build has no legacy Vue support; rebuild \
             with `--features legacy` to enable Vue 2 Options API type checking."
        );
    }
}

#[cfg(feature = "legacy")]
fn warn_for_disabled_legacy(_requested: bool) {}
