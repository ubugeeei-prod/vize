//! Filesystem walking, hidden-root expansion, and tsconfig ownership resolution.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use vize_carton::FxHashSet;

use super::glob::{default_exclude_specs, normalize_input_path, normalize_walked_path};
use super::loader::{collect_tsconfig_project_paths, load_tsconfig_inputs};
use super::matching::{
    is_generated_path, is_hidden_path_segment, is_supported_check_file, matches_tsconfig_patterns,
    should_skip_generated_for_root,
};
use super::spec::{FileCollectionOptions, GlobSpec};

pub(super) fn collect_supported_files(
    root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
) -> Vec<PathBuf> {
    collect_supported_files_with_options(root, includes, excludes, FileCollectionOptions::default())
}

pub(super) fn collect_supported_files_with_options(
    root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
    options: FileCollectionOptions,
) -> Vec<PathBuf> {
    // Keep the tsconfig scan ignore-aware and canonicalize only the root. The
    // matched files are sorted after collection, so the parallel walk can avoid
    // expensive per-entry canonicalization without making CLI output unstable.
    let skip_generated = should_skip_generated_for_root(root);
    let normalized_root = normalize_input_path(root);
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(!options.include_hidden)
        .build_parallel();

    let collected = std::sync::Mutex::new(Vec::<PathBuf>::new());
    walker.run(|| {
        let collected = &collected;
        let normalized_root = normalized_root.clone();
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && is_supported_check_file(path)
                    && (!skip_generated || !is_generated_path(path))
                    && matches_tsconfig_patterns(path, includes, excludes)
                    && let Ok(mut collected) = collected.lock()
                {
                    collected.push(normalize_walked_path(root, &normalized_root, path));
                }
            }
            ignore::WalkState::Continue
        })
    });

    let Ok(mut collected) = collected.into_inner() else {
        return Vec::new();
    };
    collected.sort();
    collected.dedup();
    collected
}

pub(super) fn explicit_hidden_include_roots(
    project_root: &Path,
    includes: &[GlobSpec],
) -> Vec<PathBuf> {
    let normalized_project_root = normalize_input_path(project_root);
    let mut roots = Vec::new();
    let mut seen = FxHashSet::default();

    for include in includes {
        if path_has_hidden_component_under_root(&include.base_dir, &normalized_project_root) {
            push_hidden_include_root(&mut roots, &mut seen, &include.base_dir);
        }
        if let Some(root) = hidden_pattern_root(&include.base_dir, &include.normalized) {
            push_hidden_include_root(&mut roots, &mut seen, &root);
        }
    }

    roots
}

fn push_hidden_include_root(roots: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>, root: &Path) {
    let root = normalize_input_path(root);
    if root.is_dir() && seen.insert(root.clone()) {
        roots.push(root);
    }
}

fn path_has_hidden_component_under_root(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(is_hidden_path_segment)
    })
}

fn hidden_pattern_root(base_dir: &Path, pattern: &str) -> Option<PathBuf> {
    let mut root = base_dir.to_path_buf();
    for segment in pattern.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment.contains(['*', '?', '[']) {
            break;
        }
        root.push(segment);
        if is_hidden_path_segment(segment) {
            return Some(root);
        }
    }
    None
}

pub(crate) fn resolve_tsconfig_for_files(
    tsconfig_path: Option<&Path>,
    files: &[PathBuf],
) -> Option<PathBuf> {
    let tsconfig_path = tsconfig_path?;
    let projects = collect_tsconfig_project_paths(tsconfig_path);
    let root_project = projects
        .first()
        .cloned()
        .unwrap_or_else(|| normalize_input_path(tsconfig_path));
    let files = files
        .iter()
        .filter(|path| is_supported_check_file(path))
        .map(|path| normalize_input_path(path))
        .collect::<Vec<_>>();
    if files.is_empty() {
        return Some(root_project);
    }

    if let Some(owner) = projects
        .iter()
        .find(|project| files.iter().all(|file| tsconfig_owns_file(project, file)))
    {
        return Some(owner.clone());
    }

    let mut shared_owner = None::<PathBuf>;
    for file in &files {
        let Some(owner) = projects
            .iter()
            .find(|project| tsconfig_owns_file(project, file))
        else {
            return Some(root_project);
        };
        match &shared_owner {
            Some(shared) if shared != owner => return Some(root_project),
            Some(_) => {}
            None => shared_owner = Some(owner.clone()),
        }
    }

    shared_owner.or(Some(root_project))
}

fn tsconfig_owns_file(tsconfig_path: &Path, file: &Path) -> bool {
    let Some(spec) = load_tsconfig_inputs(tsconfig_path) else {
        return false;
    };
    let file = normalize_input_path(file);
    if spec
        .files
        .iter()
        .any(|entry| normalize_input_path(&entry.resolve()) == file)
    {
        return true;
    }

    let default_base_dir = tsconfig_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let includes = if !spec.has_includes && !spec.has_files {
        GlobSpec::new(&default_base_dir, "**/*")
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        spec.includes
    };
    if includes.is_empty() || !is_supported_check_file(&file) {
        return false;
    }
    let excludes = if !spec.has_excludes {
        default_exclude_specs(&default_base_dir)
    } else {
        spec.excludes
    };

    matches_tsconfig_patterns(&file, &includes, &excludes)
}
