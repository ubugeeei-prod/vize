use std::path::{Path, PathBuf};

/// Walk upward until a `node_modules/vue` anchor is found.
pub(super) fn find_node_modules_with_vue(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        let node_modules = dir.join("node_modules");
        if node_modules.join("vue").is_dir() {
            return Some(node_modules);
        }
        dir = dir.parent()?;
    }
}

/// Resolve the base directory used for per-client scratch state.
pub(super) fn resolve_temp_dir_base(project_root: Option<&Path>) -> PathBuf {
    let fallback_root = project_root
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    find_node_modules_with_vue(&fallback_root)
        .unwrap_or_else(|| fallback_root.join("node_modules"))
        .join(".vize")
        .join("corsa")
}
