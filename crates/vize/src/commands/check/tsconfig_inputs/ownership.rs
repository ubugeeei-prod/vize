//! Maps authored source roots to the shared Canon tsconfig ownership authority.

use std::path::{Path, PathBuf};

use vize_canon::batch::TsconfigSourceKind;

use super::{
    glob::normalize_input_path,
    loader::TsconfigInputCache,
    matching::{
        SupportedFileOptions, is_nuxt_import_manifest_path, is_supported_check_file_with_options,
    },
};

#[derive(Debug)]
pub(crate) struct TsconfigProgramInputs {
    pub(crate) tsconfig_path: PathBuf,
    pub(crate) files: Vec<PathBuf>,
}

pub(crate) fn resolve_tsconfig_program_inputs(
    tsconfig_path: Option<&Path>,
    files: &[PathBuf],
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
) -> Vec<TsconfigProgramInputs> {
    let Some(tsconfig_path) = tsconfig_path else {
        return Vec::new();
    };
    let projects = cache.project_paths(tsconfig_path);
    let root_project = projects
        .first()
        .cloned()
        .unwrap_or_else(|| normalize_input_path(tsconfig_path));
    let projects = if projects.is_empty() {
        vec![root_project]
    } else {
        projects
    };
    let mut groups = projects
        .iter()
        .cloned()
        .map(|tsconfig_path| TsconfigProgramInputs {
            tsconfig_path,
            files: Vec::new(),
        })
        .collect::<Vec<_>>();

    for file in files.iter().filter(|path| {
        is_supported_check_file_with_options(
            path,
            SupportedFileOptions {
                // JavaScript reaches Canon so the exact owning config's
                // inherited allowJs value decides membership.
                include_js: true,
                include_jsx,
            },
        )
    }) {
        let file = normalize_input_path(file);
        let owner = if is_nuxt_import_manifest_path(&file) {
            0
        } else {
            let effective = cache.effective_config_for_source(
                tsconfig_path,
                &file,
                source_kind(&file, include_jsx),
            );
            projects
                .iter()
                .position(|project| project == &effective)
                .unwrap_or(0)
        };
        groups[owner].files.push(file);
    }

    groups.retain(|group| !group.files.is_empty());
    groups
}

pub(crate) fn resolve_tsconfig_for_files(
    tsconfig_path: Option<&Path>,
    files: &[PathBuf],
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
) -> Option<PathBuf> {
    let tsconfig_path = tsconfig_path?;
    let root_project = cache
        .project_paths(tsconfig_path)
        .first()
        .cloned()
        .unwrap_or_else(|| normalize_input_path(tsconfig_path));
    let groups = resolve_tsconfig_program_inputs(Some(tsconfig_path), files, include_jsx, cache);
    match groups.as_slice() {
        [] => Some(root_project),
        [group] => Some(group.tsconfig_path.clone()),
        _ => Some(root_project),
    }
}

fn source_kind(path: &Path, include_jsx: bool) -> TsconfigSourceKind {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("js" | "mjs" | "cjs") => TsconfigSourceKind::JavaScript,
        Some("jsx") if !include_jsx => TsconfigSourceKind::JavaScript,
        _ => TsconfigSourceKind::Typed,
    }
}
