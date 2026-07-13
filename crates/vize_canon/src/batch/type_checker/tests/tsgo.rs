use std::path::{Path, PathBuf};

pub(super) fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }

    let resolved = vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace_root);
    assert!(
        resolved.is_some() || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
        "VIZE_TEST_REQUIRE_TSGO is set, but no tsgo executable was found"
    );
    resolved
}
