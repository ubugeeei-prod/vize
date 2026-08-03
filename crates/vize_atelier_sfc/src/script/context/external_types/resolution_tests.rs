use std::sync::atomic::{AtomicU64, Ordering};

use super::super::super::batch_epoch::NO_EPOCH;
use super::{CachedPath, cached_path_is_fresh, resolve_package_types};

#[test]
fn cached_path_revalidates_only_once_per_batch() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let project = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-cached-path-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&project).unwrap();
    let file = project.join("present.ts");
    std::fs::write(&file, "export type T = string").unwrap();

    let entry = CachedPath {
        path: file.clone(),
        validated_epoch: AtomicU64::new(7),
    };

    // The same batch trusts the cached path without touching the filesystem.
    std::fs::remove_file(&file).unwrap();
    assert!(cached_path_is_fresh(&entry, 7));

    // A new batch re-stats and observes the deletion.
    assert!(!cached_path_is_fresh(&entry, 8));

    // Outside a batch every call re-stats and stamps NO_EPOCH.
    std::fs::write(&file, "export type T = number").unwrap();
    assert!(cached_path_is_fresh(&entry, NO_EPOCH));
    assert_eq!(entry.validated_epoch.load(Ordering::Relaxed), NO_EPOCH);

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn package_root_prefers_exports_types_over_top_level_types() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-exports-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"types":"./legacy.d.ts","exports":{".":{"types":"./dist/index.d.ts","import":"./dist/index.mjs"}}}"#,
    )
    .unwrap();
    std::fs::write(package.join("legacy.d.ts"), "export type Legacy = string").unwrap();
    std::fs::write(
        package.join("dist/index.d.ts"),
        "export type Modern = string",
    )
    .unwrap();

    let resolved = resolve_package_types(&package, "").unwrap();
    assert!(
        resolved.ends_with("dist/index.d.ts"),
        "expected exports types entry, got {resolved:?}"
    );

    let _ = std::fs::remove_dir_all(package);
}
