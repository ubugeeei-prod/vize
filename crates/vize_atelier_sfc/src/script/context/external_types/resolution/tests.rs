use super::{resolve_at_src_alias, resolve_import_path};
use std::path::{Path, PathBuf};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

#[test]
fn resolves_at_alias_from_nearest_src_directory() {
    let project = temp_project_dir("at-alias");
    let components = project.join("packages/frontend/src/components");
    std::fs::create_dir_all(&components).unwrap();
    let target = components.join("Base.vue");
    std::fs::write(&target, "").unwrap();

    let current = components.join("Child.vue");
    let resolved = resolve_at_src_alias(&current, "@/components/Base.vue");
    let target = target.canonicalize().unwrap();

    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn ignores_at_alias_without_src_ancestor() {
    let current = Path::new("/repo/packages/frontend/components/Child.vue");

    assert!(resolve_at_src_alias(current, "@/components/Base.vue").is_none());
}

#[test]
fn leaves_non_at_alias_specifiers_to_existing_resolution() {
    let current = Path::new("/repo/src/components/Child.vue");

    assert!(resolve_import_path(current, "vue").is_none());
}

#[test]
fn resolves_bare_specifier_through_node_modules_types_field() {
    let project = temp_project_dir("bare-types-field");
    let package = project.join("node_modules/some-ui");
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{ "name": "some-ui", "types": "./dist/index.d.ts" }"#,
    )
    .unwrap();
    std::fs::write(
        package.join("dist/index.d.ts"),
        "export interface RootProps { autocomplete?: string }",
    )
    .unwrap();
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();

    let current = components.join("Select.vue");
    let resolved = resolve_import_path(&current, "some-ui");
    let target = package.join("dist/index.d.ts").canonicalize().unwrap();
    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn resolves_scoped_bare_specifier_through_exports_types() {
    let project = temp_project_dir("bare-exports-types");
    let package = project.join("node_modules/@scope/pkg");
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{ "name": "@scope/pkg", "exports": { ".": { "import": { "types": "./dist/main.d.mts", "default": "./dist/main.mjs" } } } }"#,
    )
    .unwrap();
    std::fs::write(package.join("dist/main.d.mts"), "export type T = string").unwrap();
    let src = project.join("src");
    std::fs::create_dir_all(&src).unwrap();

    let current = src.join("App.vue");
    let resolved = resolve_import_path(&current, "@scope/pkg");
    let target = package.join("dist/main.d.mts").canonicalize().unwrap();
    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn does_not_follow_bare_specifiers_from_inside_node_modules() {
    let project = temp_project_dir("bare-from-node-modules");
    let nested = project.join("node_modules/vue");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(
        nested.join("package.json"),
        r#"{ "types": "./index.d.ts" }"#,
    )
    .unwrap();
    std::fs::write(nested.join("index.d.ts"), "export type X = 1").unwrap();

    let current = project.join("node_modules/some-ui/dist/index.d.ts");
    assert!(resolve_import_path(&current, "vue").is_none());

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn cached_path_revalidates_only_once_per_batch() {
    use super::{CachedPath, NO_EPOCH, cached_path_is_fresh};
    use std::sync::atomic::{AtomicU64, Ordering};

    let project = temp_project_dir("cached-path-freshness");
    std::fs::create_dir_all(&project).unwrap();
    let file = project.join("present.ts");
    std::fs::write(&file, "export type T = string").unwrap();

    // Pretend this entry was validated in batch epoch 7.
    let entry = CachedPath {
        path: file.clone(),
        validated_epoch: AtomicU64::new(7),
    };

    // A later hit in the *same* batch trusts the entry without touching the
    // filesystem — proven by deleting the file first: a re-stat would fail,
    // but the same-epoch fast path returns `true` anyway.
    std::fs::remove_file(&file).unwrap();
    assert!(
        cached_path_is_fresh(&entry, 7),
        "same-epoch hit must skip the is_file stat"
    );

    // A new batch (epoch 8) forces revalidation, which now fails because the
    // file is gone — so a fresh resolution would run.
    assert!(
        !cached_path_is_fresh(&entry, 8),
        "a new batch must re-stat and observe the deletion"
    );

    // Outside any batch (NO_EPOCH) every call re-stats; recreate the file
    // and confirm it revalidates and stamps the epoch forward.
    std::fs::write(&file, "export type T = number").unwrap();
    assert!(cached_path_is_fresh(&entry, NO_EPOCH));
    assert_eq!(entry.validated_epoch.load(Ordering::Relaxed), NO_EPOCH);

    let _ = std::fs::remove_dir_all(project);
}
