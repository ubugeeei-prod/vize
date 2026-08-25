use std::path::{Path, PathBuf};

use vize_s0::FxHashSet;

use super::default_imports::canonical_file_set;
use crate::commands::check::{
    path_cache::CanonicalPathCache,
    patterns::{CheckFileOptions, is_supported_check_file},
    tsconfig_inputs::{
        TsconfigInputCache, resolve_tsconfig_for_files, tsconfig_allows_js,
        tsconfig_project_graph_allows_js,
    },
};

pub(super) fn project_graph_allows_js(
    tsconfig_path: Option<&Path>,
    cache: &mut TsconfigInputCache,
) -> bool {
    tsconfig_path.is_some_and(|path| tsconfig_project_graph_allows_js(path, cache))
}

pub(super) struct ProgramInputContext<'a> {
    pub(super) tsconfig_path: Option<&'a Path>,
    pub(super) explicit: bool,
    pub(super) include_jsx: bool,
    pub(super) files: &'a mut Vec<PathBuf>,
    pub(super) inputs: &'a mut Vec<PathBuf>,
    pub(super) reported: &'a mut FxHashSet<PathBuf>,
    pub(super) cache: &'a mut TsconfigInputCache,
    pub(super) canonical_paths: &'a mut CanonicalPathCache,
}

pub(super) fn filter_for_program(context: ProgramInputContext<'_>) -> Option<PathBuf> {
    let program_tsconfig_path = resolve_tsconfig_for_files(
        context.tsconfig_path,
        context.inputs,
        context.include_jsx,
        context.cache,
    );
    if !context.explicit {
        return program_tsconfig_path;
    }

    // Candidate collection admits JavaScript when any referenced project
    // enables allowJs. Once ownership selects the program, apply that exact
    // config so an allowJs child cannot make a deny sibling eligible.
    let file_options = CheckFileOptions {
        include_js: program_tsconfig_path
            .as_deref()
            .is_some_and(|path| tsconfig_allows_js(path, context.cache)),
        include_jsx: context.include_jsx,
    };
    context
        .files
        .retain(|path| is_supported_check_file(path, file_options));
    context
        .inputs
        .retain(|path| is_supported_check_file(path, file_options));
    *context.reported = canonical_file_set(context.inputs, context.canonical_paths);

    program_tsconfig_path
}
