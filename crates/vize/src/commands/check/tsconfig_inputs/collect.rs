//! Filesystem walking and hidden-root expansion.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use vize_s0::FxHashSet;

use super::glob::{normalize_input_path, normalize_walked_path};
use super::matching::{
    SupportedFileOptions, is_generated_codegen_declaration_path, is_generated_path,
    is_hidden_path_segment, is_nuxt_import_manifest_path, is_supported_check_file_with_options,
    matches_tsconfig_patterns, should_skip_generated_for_root,
};
use super::spec::{FileCollectionOptions, GlobSpec};

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
                    && is_supported_check_file_with_options(
                        path,
                        SupportedFileOptions {
                            include_js: options.include_js,
                            include_jsx: options.include_jsx,
                        },
                    )
                    && (!skip_generated || !is_generated_path(path))
                    && !is_nuxt_import_manifest_path(path)
                    && !is_generated_codegen_declaration_path(path)
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

pub(super) fn collect_supported_files_for_include_roots(
    project_root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
    options: FileCollectionOptions,
) -> Vec<PathBuf> {
    let normalized_project_root = normalize_input_path(project_root);
    let mut roots = vec![normalized_project_root.clone()];
    let mut seen_roots = FxHashSet::default();
    seen_roots.insert(normalized_project_root.clone());

    for include in includes {
        let root = normalize_input_path(&include.base_dir);
        if root.is_dir()
            && !root.starts_with(&normalized_project_root)
            && seen_roots.insert(root.clone())
        {
            roots.push(root);
        }
    }

    let mut files = Vec::new();
    let mut seen_files = FxHashSet::default();
    for root in roots {
        for path in collect_supported_files_with_options(&root, includes, excludes, options) {
            if seen_files.insert(path.clone()) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
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

/// Include roots whose hidden directory is spelled out by the include pattern
/// itself, such as `packages/docs/.vitepress/theme/components`.
///
/// `tsc` drops dot-directories only while expanding wildcards; a literal path
/// segment is matched literally, so these files are part of the program and
/// must be checked. This is narrower than [`explicit_hidden_include_roots`],
/// which also treats a tsconfig that merely *lives* in a hidden directory
/// (`.nuxt/tsconfig.json`) as a hidden root — appropriate when collecting
/// ambient declarations, but not for deciding what to typecheck.
pub(super) fn explicit_hidden_pattern_roots(includes: &[GlobSpec]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let mut seen = FxHashSet::default();

    for include in includes {
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
