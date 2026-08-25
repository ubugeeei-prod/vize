use std::path::{Path, PathBuf};

use vize_s0::FxHashSet;

use crate::commands::check::path_cache::CanonicalPathCache;

/// Whether a registered file's diagnostics should be reported. Configured or
/// explicit source roots and their authored transitive imports are reported;
/// ambient-only support and dependency files exist only to resolve cross-file
/// types. Project-level diagnostics (anchored to a tsconfig or the project root,
/// not a source file) describe the whole check and are always reported.
pub(in crate::commands::check::runner) fn is_reported(
    reported: &FxHashSet<PathBuf>,
    path: &Path,
    canonical_paths: &mut CanonicalPathCache,
) -> bool {
    if !is_source_path(path) {
        return true;
    }
    reported.contains(&canonical_paths.canonicalize(path))
}

fn is_source_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "vue" | "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs"
            )
        })
}
