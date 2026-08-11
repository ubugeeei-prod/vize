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
