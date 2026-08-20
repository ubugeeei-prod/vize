//! Direct check orchestration over one or more effective TypeScript programs.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use vize_carton::profiler::global_profiler;

use super::{
    CanonicalPathCache, CheckArgs, CheckerSettings, TsconfigInputCache, build_virtual_ts_options,
    collect_roots, dialect_from_features, exit_after_execution_error, explicit_input_root,
    finish_executions, load_check_ignore_set, prepare_and_execute, report_no_inputs,
    resolve_from_config_dir, resolve_invocation_program, resolve_nuxt_project_root,
    split_program_candidates, template_syntax_mode, validate_config_arg,
    validate_corsa_server_count, warn_for_disabled_legacy,
};

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
        let style = super::text_style::TextStyle::stderr();
        eprintln!("{} {}", style.red("Error:"), error);
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
    let mut package_route_resolver = vize_canon::PackageRouteResolver::default();
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
        &mut package_route_resolver,
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
            &mut package_route_resolver,
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
