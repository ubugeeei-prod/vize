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
        !vize_carton::path::is_git_metadata_path(path)
            && is_supported_check_file_with_options(
                path,
                SupportedFileOptions {
                    // JavaScript reaches Canon so the exact owning config's
                    // inherited allowJs value decides membership.
                    include_js: true,
                    include_jsx,
                },
            )
    }) {
        let file = cache.normalize(file);
        let owner = if is_nuxt_import_manifest_path(&file) {
            0
        } else {
            let effective = cache.effective_config_for_normalized_source(
                tsconfig_path,
                &file,
                source_kind(&file, include_jsx),
            );
            projects
                .iter()
                .position(|project| project == &effective)
                .unwrap_or(0)
        };
        if let Some(group) = groups.get_mut(owner) {
            group.files.push(file);
        }
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

/// An authored import a referenced project owns, with that project's config.
pub(crate) struct ForeignSource {
    pub(crate) path: PathBuf,
    pub(crate) owner: PathBuf,
}

/// Split authored imports into those this program reports and those a
/// referenced project owns. `tsc -b` checks the latter in their owning
/// project alone, so an importing program neither reports them nor checks
/// them without the owner's ambient declarations (#11642); it can mirror the
/// owner's emitted declarations instead (#12102). A program without
/// references owns everything.
pub(crate) fn partition_program_owned_sources(
    tsconfig_path: Option<&Path>,
    files: Vec<PathBuf>,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
) -> (Vec<PathBuf>, Vec<ForeignSource>) {
    let Some(tsconfig_path) = tsconfig_path else {
        return (files, Vec::new());
    };
    let projects = cache.project_paths(tsconfig_path);
    let (Some(root_project), true) = (projects.first(), projects.len() > 1) else {
        return (files, Vec::new());
    };
    let root_project = root_project.clone();
    let mut owned = Vec::new();
    let mut foreign = Vec::new();
    for file in files {
        let normalized = cache.normalize(&file);
        if is_nuxt_import_manifest_path(&normalized) {
            owned.push(file);
            continue;
        }
        let effective = cache.effective_config_for_normalized_source(
            tsconfig_path,
            &normalized,
            source_kind(&normalized, include_jsx),
        );
        if effective == root_project {
            owned.push(file);
        } else {
            foreign.push(ForeignSource {
                path: file,
                owner: effective,
            });
        }
    }
    (owned, foreign)
}

/// Keep only the authored imports this program reports; see
/// [`partition_program_owned_sources`].
pub(crate) fn retain_program_owned_sources(
    tsconfig_path: Option<&Path>,
    files: &mut Vec<PathBuf>,
    include_jsx: bool,
    cache: &mut TsconfigInputCache,
) {
    let (owned, _) =
        partition_program_owned_sources(tsconfig_path, std::mem::take(files), include_jsx, cache);
    *files = owned;
}

/// The projects `tsconfig_path` references, transitively, without itself.
pub(crate) fn referenced_projects(
    tsconfig_path: &Path,
    cache: &mut TsconfigInputCache,
) -> Vec<PathBuf> {
    let mut projects = cache.project_paths(tsconfig_path);
    if !projects.is_empty() {
        projects.remove(0);
    }
    projects
}

fn source_kind(path: &Path, include_jsx: bool) -> TsconfigSourceKind {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("js" | "mjs" | "cjs") => TsconfigSourceKind::JavaScript,
        Some("jsx") if !include_jsx => TsconfigSourceKind::JavaScript,
        _ => TsconfigSourceKind::Typed,
    }
}
