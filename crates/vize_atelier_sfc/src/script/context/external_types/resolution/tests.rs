#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
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

#[test]
fn package_root_falls_back_to_typescript_source_when_declarations_are_unbuilt() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-workspace-source-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"types":"./index.d.ts","module":"./index.ts","main":"./index.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.ts"),
        "export interface WorkspaceProps { emptyValues?: unknown[] }",
    )
    .unwrap();

    let resolved = resolve_package_types(&package, "").unwrap();
    let expected = package.join("index.ts").canonicalize().unwrap();
    assert_eq!(resolved, expected);

    let _ = std::fs::remove_dir_all(package);
}

#[test]
fn package_root_reads_condition_only_exports_map() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-root-conditions-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(package.join("dist")).unwrap();
    // `exports` may omit the `"."` subpath and hold conditions directly.
    std::fs::write(
        package.join("package.json"),
        r#"{"types":"./legacy.d.ts","exports":{"types":"./dist/index.d.ts","default":"./dist/index.js"}}"#,
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
        "expected root condition types entry, got {resolved:?}"
    );

    let _ = std::fs::remove_dir_all(package);
}

#[test]
fn package_root_ignores_subpath_only_exports_map() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-subpath-only-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(package.join("dist")).unwrap();
    // Only a subpath is exported, so its `types` must not answer for the root.
    std::fs::write(
        package.join("package.json"),
        r#"{"types":"./legacy.d.ts","exports":{"./sub":{"types":"./dist/sub.d.ts"}}}"#,
    )
    .unwrap();
    std::fs::write(package.join("legacy.d.ts"), "export type Legacy = string").unwrap();
    std::fs::write(package.join("dist/sub.d.ts"), "export type Sub = string").unwrap();

    let resolved = resolve_package_types(&package, "").unwrap();
    assert!(
        resolved.ends_with("legacy.d.ts"),
        "expected legacy types fallback, got {resolved:?}"
    );

    let _ = std::fs::remove_dir_all(package);
}

#[test]
fn subpath_falls_back_to_wildcard_exports_types() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-wildcard-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(package.join("types")).unwrap();
    std::fs::create_dir_all(package.join("dist")).unwrap();
    // Only a `*` pattern subpath is exported, as bundlers commonly emit.
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{"./*":{"types":"./types/*.d.ts","import":"./dist/*.mjs"}}}"#,
    )
    .unwrap();
    std::fs::write(package.join("dist/button.mjs"), "export default {}").unwrap();
    std::fs::write(
        package.join("types/button.d.ts"),
        "export type ButtonProps = { label: string }",
    )
    .unwrap();

    let resolved = resolve_package_types(&package, "button").unwrap();
    assert!(
        resolved.ends_with("types/button.d.ts"),
        "expected wildcard exports types entry, got {resolved:?}"
    );

    let _ = std::fs::remove_dir_all(package);
}

#[test]
fn subpath_excluded_by_a_null_export_is_not_resolved() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-null-export-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(package.join("types")).unwrap();
    std::fs::create_dir_all(package.join("private")).unwrap();
    // The broad pattern exposes declarations, but `./private/*` is blocked.
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{"./*":{"types":"./types/*.d.ts"},"./private/*":null}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("types/button.d.ts"),
        "export type ButtonProps = { label: string }",
    )
    .unwrap();
    // A physical declaration under the blocked subpath must stay unreachable.
    std::fs::write(
        package.join("private/internal.d.ts"),
        "export type Internal = { secret: string }",
    )
    .unwrap();

    assert!(resolve_package_types(&package, "button").is_some());
    let blocked = resolve_package_types(&package, "private/internal");
    assert!(blocked.is_none(), "expected no resolution, got {blocked:?}");

    let _ = std::fs::remove_dir_all(package);
}

/// Writes `files` into a fresh package directory and resolves `subpath`.
fn resolve_in_package(files: &[(&str, &str)], subpath: &str) -> Option<std::path::PathBuf> {
    let package = tempfile::tempdir().unwrap();
    for (path, contents) in files {
        let path = package.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    let root = package.path().canonicalize().unwrap();
    resolve_package_types(package.path(), subpath)
        .map(|path| path.strip_prefix(&root).unwrap().to_path_buf())
}

#[test]
fn source_first_exports_resolve_without_declarations() {
    const SOURCE: &str = "export const props = {}";
    // A root `index.ts` beside the export does not answer for the package.
    for manifest in [
        r#"{"exports":"./src/entry.ts"}"#,
        r#"{"exports":{".":"./src/entry.ts"}}"#,
        r#"{"exports":{".":{"import":"./src/entry.ts","require":"./cjs/entry.cjs"}}}"#,
        r#"{"exports":{"require":"./cjs/entry.cjs","default":"./src/entry.ts"}}"#,
        r#"{"exports":{".":[{"import":"./missing.ts"},"./src/entry.ts"]}}"#,
    ] {
        let resolved = resolve_in_package(
            &[
                ("package.json", manifest),
                ("src/entry.ts", SOURCE),
                ("index.ts", SOURCE),
            ],
            "",
        );
        assert_eq!(
            resolved.as_deref(),
            Some(std::path::Path::new("src/entry.ts")),
            "{manifest}"
        );
    }

    let manifest =
        r#"{"exports":{"./props":"./src/props.ts","./parts/*":{"import":"./src/parts/*.ts"}}}"#;
    for (subpath, expected) in [
        ("props", "src/props.ts"),
        ("parts/button", "src/parts/button.ts"),
    ] {
        let resolved = resolve_in_package(
            &[
                ("package.json", manifest),
                ("src/props.ts", SOURCE),
                ("src/parts/button.ts", SOURCE),
            ],
            subpath,
        );
        assert_eq!(
            resolved.as_deref(),
            Some(std::path::Path::new(expected)),
            "{subpath}"
        );
    }
}
