//! Maps authored source roots to the tsconfig program that owns them.

use std::path::{Path, PathBuf};

use vize_carton::FxHashSet;

use super::{
    glob::normalize_input_path,
    loader::TsconfigInputCache,
    matching::{
        SupportedFileOptions, is_nuxt_import_manifest_path, is_supported_check_file_with_options,
        matches_tsconfig_patterns,
    },
    spec::GlobSpec,
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
    let mut matchers = projects
        .iter()
        .map(|project| TsconfigOwnershipMatcher::load(project, cache, include_jsx))
        .collect::<Vec<_>>();
    if matchers.is_empty() {
        matchers.push(TsconfigOwnershipMatcher::unloaded(include_jsx));
    }
    let projects = if projects.is_empty() {
        vec![root_project]
    } else {
        projects
    };
    let mut groups = projects
        .into_iter()
        .map(|tsconfig_path| TsconfigProgramInputs {
            tsconfig_path,
            files: Vec::new(),
        })
        .collect::<Vec<_>>();

    for file in files.iter().filter(|path| {
        is_supported_check_file_with_options(
            path,
            SupportedFileOptions {
                // JavaScript must reach the ownership matcher so allowJs is
                // applied by its exact project, never by a sibling.
                include_js: true,
                include_jsx,
            },
        )
    }) {
        let file = normalize_input_path(file);
        let owner = matchers
            .iter()
            .position(|matcher| matcher.owns(&file))
            .unwrap_or(0);
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

/// Precompiled ownership matcher for one tsconfig project: canonicalized
/// `files` plus effective include/exclude globs with tsc defaults applied.
struct TsconfigOwnershipMatcher {
    loaded: bool,
    files: FxHashSet<PathBuf>,
    includes: Vec<GlobSpec>,
    excludes: Vec<GlobSpec>,
    include_js: bool,
    include_jsx: bool,
}

impl TsconfigOwnershipMatcher {
    fn unloaded(include_jsx: bool) -> Self {
        Self {
            loaded: false,
            files: FxHashSet::default(),
            includes: Vec::new(),
            excludes: Vec::new(),
            include_js: false,
            include_jsx,
        }
    }

    fn load(tsconfig_path: &Path, cache: &mut TsconfigInputCache, include_jsx: bool) -> Self {
        let Some(spec) = cache.load(tsconfig_path) else {
            return Self::unloaded(include_jsx);
        };
        let files = spec
            .files
            .iter()
            .map(|entry| normalize_input_path(&entry.resolve()))
            .collect();
        let default_base_dir = tsconfig_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let includes = if !spec.has_includes && !spec.has_files {
            GlobSpec::new(&default_base_dir, "**/*")
                .into_iter()
                .collect()
        } else {
            spec.includes.clone()
        };

        Self {
            loaded: true,
            files,
            includes,
            excludes: spec.effective_excludes(),
            include_js: spec.allow_js.unwrap_or(false),
            include_jsx,
        }
    }

    fn owns(&self, file: &Path) -> bool {
        if !self.loaded || is_nuxt_import_manifest_path(file) {
            return false;
        }
        if self.files.contains(file) {
            return true;
        }
        if self.includes.is_empty()
            || !is_supported_check_file_with_options(
                file,
                SupportedFileOptions {
                    include_js: self.include_js,
                    include_jsx: self.include_jsx,
                },
            )
        {
            return false;
        }
        matches_tsconfig_patterns(file, &self.includes, &self.excludes)
    }
}
