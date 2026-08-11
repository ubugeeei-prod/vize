use std::path::{Path, PathBuf};
pub(crate) fn resolve_test_corsa_path() -> Option<PathBuf> {
    std::env::var_os("CORSA_PATH")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            let path = workspace_node_modules().join(".bin/tsgo");
            path.exists().then_some(path)
        })
}

pub(crate) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

pub(crate) fn workspace_node_modules() -> PathBuf {
    std::env::var_os("VIZE_TEST_NODE_MODULES")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("node_modules"))
}
