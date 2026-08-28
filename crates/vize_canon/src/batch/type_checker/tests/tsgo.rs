use std::path::{Path, PathBuf};

pub(super) fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    if let Some(path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) {
        assert!(
            path.is_file() || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "VIZE_TEST_REQUIRE_TSGO is set, but CORSA_PATH is not a file: {}",
            path.display()
        );
        return path.is_file().then_some(path);
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent);
    resolve_test_tsgo_binary_from(
        workspace_root,
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
    )
}

fn resolve_test_tsgo_binary_from(
    workspace_root: Option<&Path>,
    require_tsgo: bool,
) -> Option<PathBuf> {
    if let Some(sibling_cache) = workspace_root
        .and_then(Path::parent)
        .map(|parent| parent.join("corsa-bind/.cache/tsgo"))
    {
        if sibling_cache.exists() {
            return Some(sibling_cache);
        }
    }

    let resolved =
        workspace_root.and_then(vize_carton::corsa_resolver::discover_corsa_in_ancestors);
    assert!(
        resolved.is_some() || !require_tsgo,
        "VIZE_TEST_REQUIRE_TSGO is set, but no TypeScript 7/Corsa executable was found"
    );
    resolved
}

#[test]
#[should_panic(expected = "VIZE_TEST_REQUIRE_TSGO is set")]
fn required_tsgo_rejects_missing_workspace_root() {
    let _ = resolve_test_tsgo_binary_from(None, true);
}
