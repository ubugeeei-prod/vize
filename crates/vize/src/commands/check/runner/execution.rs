//! One compiler-options-homogeneous Corsa program within a check invocation.

use std::{path::PathBuf, time::Duration, time::Instant};

use vize_canon::{
    BatchTypeCheckResult, BatchTypeChecker, BatchTypeCheckerOptions,
    batch::TypeChecker as BatchTypeCheckerTrait,
};
use vize_carton::{FxHashSet, config::VueVersion};

use super::{collect_project_global_component_stubs, resolve_checker_tsconfig_path};
use crate::commands::check::nuxt;

#[derive(Clone)]
pub(super) struct CheckerSettings {
    pub(super) virtual_ts_options: vize_canon::virtual_ts::VirtualTsOptions,
    pub(super) corsa_path: Option<PathBuf>,
    pub(super) servers: Option<usize>,
    pub(super) options_api: bool,
    pub(super) legacy_vue2: bool,
    pub(super) jsx_typecheck: bool,
    pub(super) template_syntax: vize_atelier_core::TemplateSyntaxMode,
    pub(super) experimental_in_tag_comments: bool,
    pub(super) dialect: VueVersion,
    pub(super) check_props: bool,
    pub(super) check_template_bindings: bool,
    pub(super) check_emits: bool,
    pub(super) quiet: bool,
}

pub(super) struct ProgramExecution {
    pub(super) checker: BatchTypeChecker,
    pub(super) result: BatchTypeCheckResult,
    pub(super) reported_files: FxHashSet<PathBuf>,
    pub(super) tsconfig_path: Option<PathBuf>,
    pub(super) program_root: PathBuf,
    pub(super) gen_time: Duration,
    pub(super) check_time: Duration,
}

pub(super) struct ProgramExecutionInput<'a> {
    pub(super) files: &'a [PathBuf],
    pub(super) reported_files: FxHashSet<PathBuf>,
    pub(super) virtual_module_aliases: &'a [(vize_carton::String, PathBuf)],
    pub(super) project_root: &'a std::path::Path,
    pub(super) program_root: PathBuf,
    pub(super) tsconfig_path: Option<PathBuf>,
    pub(super) nuxt_project_root: &'a std::path::Path,
}

pub(super) fn execute_program(
    input: ProgramExecutionInput<'_>,
    settings: &CheckerSettings,
) -> ProgramExecution {
    let mut virtual_ts_options = settings.virtual_ts_options.clone();
    let nuxt_path_aliases = nuxt::detect(
        &mut virtual_ts_options,
        input.nuxt_project_root,
        input.tsconfig_path.as_deref(),
        settings.legacy_vue2,
        settings.dialect,
    );
    collect_project_global_component_stubs(
        &mut virtual_ts_options,
        input.files,
        input.project_root,
        input.tsconfig_path.as_deref(),
    );
    let checker_tsconfig_path = resolve_checker_tsconfig_path(
        input.tsconfig_path.as_deref(),
        input.project_root,
        input.nuxt_project_root,
        &nuxt_path_aliases,
    )
    .unwrap_or_else(|error| {
        eprintln!(
            "\x1b[31mError:\x1b[0m Failed to prepare type checker tsconfig: {}",
            error
        );
        std::process::exit(1);
    });

    if !settings.quiet {
        eprintln!(
            "Building Corsa virtual project for {} files under {}...",
            input.files.len(),
            input.program_root.display()
        );
    }

    let gen_start = Instant::now();
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        input.project_root,
        BatchTypeCheckerOptions {
            tsconfig_path: checker_tsconfig_path,
            virtual_ts_options,
        },
        settings.corsa_path.as_deref(),
    )
    .unwrap_or_else(|error| {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(1);
    });
    checker.set_server_count(settings.servers);
    if settings.options_api {
        checker.enable_options_api();
    }
    #[cfg(feature = "legacy")]
    if settings.legacy_vue2 {
        checker.enable_legacy_vue2();
    }
    if settings.jsx_typecheck {
        checker.enable_jsx_typecheck();
    }
    checker.set_template_syntax(settings.template_syntax);
    checker.set_experimental_in_tag_comments(settings.experimental_in_tag_comments);
    checker.set_dialect(settings.dialect);
    checker.set_virtual_ts_checks(
        settings.check_props,
        settings.check_template_bindings,
        settings.check_emits,
    );
    checker.set_virtual_module_aliases(input.virtual_module_aliases.iter().cloned());
    checker.scan_paths(input.files).unwrap_or_else(|error| {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(1);
    });
    checker.set_diagnostic_paths(input.reported_files.iter().map(PathBuf::as_path));
    let gen_time = gen_start.elapsed();

    if !settings.quiet {
        eprintln!(
            "Running Corsa diagnostics for {} files...",
            checker.virtual_files().len()
        );
    }
    let check_start = Instant::now();
    let result = checker.check_project().unwrap_or_else(|error| {
        eprintln!("\x1b[31mError:\x1b[0m {}", error);
        std::process::exit(1);
    });

    ProgramExecution {
        checker,
        result,
        reported_files: input.reported_files,
        tsconfig_path: input.tsconfig_path,
        program_root: input.program_root,
        gen_time,
        check_time: check_start.elapsed(),
    }
}
