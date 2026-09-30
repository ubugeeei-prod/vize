//! Shared TypeScript project-reference ownership.
//!
//! Editor project selection and CLI program partitioning must use the same
//! effective `files` / `include` / `exclude` interpretation. Keeping that
//! authority here also lets the editor use a referenced config's inherited
//! compiler options without teaching either consumer its own tsconfig dialect.

use std::path::{Path, PathBuf};

mod graph;
mod implicit_exclude;
mod ownership;
mod spec;

pub use ownership::{TsconfigOwnershipCache, TsconfigOwnershipOptions, TsconfigSourceKind};

use super::super::tsconfig_paths::{
    normalize_path_lexically, parse_jsonc_value, resolve_extended_tsconfig_path,
};

/// The transitive project configs referenced by `tsconfig_path`, in stable
/// declaration order. The solution shell itself is omitted.
pub(in super::super) fn referenced_project_configs(tsconfig_path: &Path) -> Vec<PathBuf> {
    let mut cache = TsconfigOwnershipCache::default();
    cache
        .project_paths(tsconfig_path)
        .into_iter()
        .skip(1)
        .collect()
}

/// Authored sources under `project_root` that this tsconfig's `files` /
/// `include` accepts. The walk skips `node_modules`, VCS, build output, and
/// dot directories. TypeScript's `**` does not descend into hidden
/// directories, and a gitignore-aware walk would drop included sources the
/// diagnostic pass still has to see.
pub(in super::super) fn included_sources(
    tsconfig_path: &Path,
    project_root: &Path,
) -> Vec<PathBuf> {
    let Some(spec) = spec::SpecCache::default().load(tsconfig_path) else {
        return Vec::new();
    };
    let case_sensitive = !cfg!(windows);
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(project_root)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            name != "node_modules" && name != ".git" && name != "target" && !name.starts_with('.')
        })
    {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        // Generated `.d.ts` keeps its authored path. Mirroring it loads the
        // module twice (#2047).
        if crate::batch::declaration_path::is_declaration_file(path) {
            continue;
        }
        let Some(kind) = included_source_kind(path) else {
            continue;
        };
        if spec.includes(path, case_sensitive, kind) {
            found.push(graph::normalize_path(path));
        }
    }
    found.sort();
    found.dedup();
    found
}

fn included_source_kind(path: &Path) -> Option<TsconfigSourceKind> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("js" | "jsx" | "mjs" | "cjs") => Some(TsconfigSourceKind::JavaScript),
        Some("vue" | "ts" | "tsx" | "mts" | "cts") => Some(TsconfigSourceKind::Typed),
        _ => None,
    }
}

/// Select the unique effective project that owns an authored source. Missing
/// or ambiguous ownership fails closed to the solution shell.
pub(in super::super) fn effective_config_for_source(
    tsconfig_path: &Path,
    source_path: &Path,
) -> PathBuf {
    TsconfigOwnershipCache::default().effective_config_for_source(
        tsconfig_path,
        source_path,
        TsconfigSourceKind::Typed,
    )
}

#[cfg(test)]
#[path = "references/tests.rs"]
mod tests;
