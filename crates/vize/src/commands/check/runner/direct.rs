//! Direct check orchestration over one or more effective TypeScript programs.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use super::{
    CanonicalPathCache, CheckArgs, CheckerSettings, TsconfigInputCache, build_virtual_ts_options,
    collect_roots, dialect_from_features, exit_after_execution_error, explicit_input_root,
    finish_executions, prepare_and_execute, report_no_inputs, resolve_from_config_dir,
    resolve_invocation_program, resolve_nuxt_project_root, split_program_candidates,
    template_syntax_mode, validate_corsa_server_count, warn_for_disabled_legacy,
};

/// Run type checking directly with materialized Corsa projects.
pub(crate) fn run_direct(args: &CheckArgs) {
    let start = Instant::now();
    args.profile_export.begin(args.profile);
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_config = if args.no_config {
        crate::config::LoadedProjectConfig::default()
    } else {
        crate::config::try_load_project_config_with_source(args.config.as_deref()).unwrap_or_else(
            |error| {
                eprintln!("\x1b[31mError:\x1b[0m {error}");
                std::process::exit(2);
            },
        )
    };
    let experimental_vue = project_config.document.experimental_vue_flags();
    let compiler_template_syntax = project_config.document.compiler_template_syntax();
    let ignores = project_config.document.entry_ignores();
    let (config, features) = project_config.document.into_config_and_features();
    let loaded_config = crate::config::LoadedConfigWithFeatures {
        config,
        features,
        source_path: project_config.source_path,
    };
    crate::config::write_schema_for_config(loaded_config.source_path.as_deref());
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
    let project = crate::config::ProjectModel::new(
        Some(&cwd),
        loaded_config.source_path.as_deref(),
        &config.type_checker,
    )
    .with_explicit_tsconfig(args.tsconfig.as_deref());
    let effective_tsconfig = project.tsconfig().map(Path::to_path_buf);
    super::input_scope::validate_selected_tsconfig(args, effective_tsconfig.as_deref());
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

    let default_root = if args.patterns.is_empty() && args.tsconfig.is_none() {
        project_config.project_root.as_deref().unwrap_or(&cwd)
    } else {
        &cwd
    };
    let (invocation_project_root, invocation_tsconfig_path) =
        resolve_invocation_program(effective_tsconfig.as_deref(), default_root);
    let nuxt_project_root = resolve_nuxt_project_root(
        effective_tsconfig.as_deref(),
        &cwd,
        &invocation_project_root,
    );
    let explicit_input_root = explicit_input_root(&invocation_project_root, &cwd);
    let mut tsconfig_input_cache = TsconfigInputCache::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let mut package_route_resolver = vize_canon::PackageRouteResolver::default();
    let check_ignore_set = if project_config.project_root.is_some() {
        super::ignores::CheckIgnoreSet::for_project(&ignores, config_dir)
    } else {
        super::ignores::CheckIgnoreSet::new(&ignores, config_dir)
    };
    let collect_start = Instant::now();
    let collected = collect_roots(
        args,
        &invocation_project_root,
        default_root,
        invocation_tsconfig_path.as_deref(),
        jsx_typecheck,
        &mut tsconfig_input_cache,
        &mut canonical_paths,
        check_ignore_set.as_ref(),
        &mut package_route_resolver,
    );
    let collect_time = collect_start.elapsed();
    if collected.files.is_empty() {
        report_no_inputs(args, invocation_tsconfig_path.as_deref());
        return;
    }

    let candidates = split_program_candidates(
        collected,
        invocation_tsconfig_path.as_deref(),
        !args.patterns.is_empty() && effective_tsconfig.is_none(),
        jsx_typecheck,
        &mut tsconfig_input_cache,
        &mut canonical_paths,
    );
    let settings = CheckerSettings {
        virtual_ts_options: build_virtual_ts_options(&config, config_dir),
        corsa_path: effective_corsa_path,
        servers: corsa_servers,
        checkers: args.checkers.map(std::num::NonZero::get),
        options_api,
        legacy_vue2,
        jsx_typecheck,
        template_syntax: template_syntax_mode(compiler_template_syntax),
        experimental_in_tag_comments: loaded_config.features.experimental_in_tag_comments,
        experimental_patterned_template: loaded_config.features.experimental_patterned_template,
        experimental_strict_slot_children: experimental_vue.strict_slot_children,
        dialect,
        check_props: config.type_checker.check_props && !args.no_check_props,
        check_template_bindings: config.type_checker.check_template_bindings
            && !args.no_check_template_bindings,
        check_emits: config.type_checker.check_emits && !args.no_check_emits,
        quiet: args.quiet,
    };
    let validate_inputs = !args.patterns.is_empty() && invocation_tsconfig_path.is_some();
    let mut executions = Vec::new();
    let mut only_excluded_explicit_inputs = !candidates.is_empty();
    for candidate in candidates {
        let prepared = match prepare_and_execute(
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
            Ok(prepared) => prepared,
            Err(error) => exit_after_execution_error(executions, error),
        };
        only_excluded_explicit_inputs &= prepared.excluded_explicit_inputs;
        if let Some(execution) = prepared.execution {
            executions.push(execution);
        }
    }
    if executions.is_empty() {
        // Finding inputs that their owning program intentionally excludes is
        // distinct from selecting a project with no supported workload.
        report_no_inputs(
            args,
            invocation_tsconfig_path
                .as_deref()
                .filter(|_| !only_excluded_explicit_inputs),
        );
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
