//! Source reads and path metadata for canonical dependency traversal.

use oxc_span::SourceType;
use std::path::{Path, PathBuf};
use vize_carton::{FxHashMap, String};

pub(in crate::corsa_bridge) fn dependency_content(
    path: &Path,
    overlays: &FxHashMap<PathBuf, &str>,
) -> Option<String> {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    overlays
        .get(&key)
        .map(|content| String::from(*content))
        .or_else(|| std::fs::read_to_string(path).ok().map(Into::into))
}

pub(in crate::corsa_bridge) fn source_type_for_path(path: &Path) -> SourceType {
    SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts())
}

pub(in crate::corsa_bridge) fn parent_dir(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf())
}
