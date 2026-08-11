use std::path::{Path, PathBuf};

pub(super) fn required_iterations() -> usize {
    let Some(raw) = std::env::var_os("VIZE_NUXT_CONFIG_ITERATIONS") else {
        return 1;
    };
    let raw = raw.to_string_lossy();
    let iterations = raw
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("VIZE_NUXT_CONFIG_ITERATIONS must be an integer: {raw}"));
    assert!(
        iterations > 0,
        "Nuxt config isolation iterations must be positive"
    );
    if std::env::var_os("CI").is_some() {
        assert!(
            iterations >= 100,
            "CI must run at least 100 Nuxt config isolation iterations"
        );
    }
    iterations
}

pub(super) fn resolve_test_corsa_path() -> Option<PathBuf> {
    std::env::var_os("CORSA_PATH")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .or_else(|| {
            let path = workspace_node_modules().join(".bin/tsgo");
            path.exists().then_some(path)
        })
}

pub(super) fn workspace_node_modules() -> PathBuf {
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
