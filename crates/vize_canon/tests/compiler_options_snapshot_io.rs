#![cfg(feature = "native")]

use vize_canon::{project_virtual_lock_paths, snapshot_tsconfig_compiler_options};

/// The public options reader must not acquire the generated project's locks.
#[test]
fn compiler_options_snapshot_does_not_create_cache_locks() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
    let locks = project_virtual_lock_paths(&root);
    assert!(locks.iter().all(|path| !path.exists()));
    let options = snapshot_tsconfig_compiler_options(&root, &config).unwrap();
    assert_eq!(
        options,
        serde_json::json!({"strict":true})
            .as_object()
            .unwrap()
            .clone()
    );
    assert!(
        locks.iter().all(|path| !path.exists()),
        "reading compiler options created generated-project cache locks: {locks:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        r#"{"compilerOptions":{"strict":true}}"#
    );
}
