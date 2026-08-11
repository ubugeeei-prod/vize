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

pub(crate) fn workspace_node_modules() -> PathBuf {
    std::env::var_os("VIZE_TEST_NODE_MODULES")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .unwrap()
                .join("node_modules")
        })
}
