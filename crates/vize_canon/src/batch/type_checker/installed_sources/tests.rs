#![expect(
    clippy::disallowed_macros,
    reason = "fixture assertions use std formatting"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "fixture assertions use std strings"
)]
#![expect(
    clippy::disallowed_types,
    reason = "fixture assertions use std strings"
)]
//! Installed-source storage must remain alive with its owner, and only build
//! info may outlive it.

use super::{BatchTypeChecker, BatchTypeCheckerOptions, contains_node_modules};
use std::path::{Path, PathBuf};

fn checker(root: &Path) -> BatchTypeChecker {
    let binary = root.join("unused-tsgo");
    std::fs::write(&binary, "unused fixture executable").unwrap();
    BatchTypeChecker::with_options_and_corsa_path(
        root,
        BatchTypeCheckerOptions::default(),
        Some(&binary),
    )
    .unwrap()
}

/// Only plain build info outlives an owner; none exists before a check.
fn assert_mirror_removed(storage: &Path) {
    let left = std::fs::read_dir(storage)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(left, Vec::<PathBuf>::new());
}

fn source(root: &Path, relative: &str) -> PathBuf {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "export const selected: number = 1\n").unwrap();
    path
}

#[test]
fn only_installed_path_components_select_private_storage() {
    assert!(contains_node_modules(Path::new(
        "node_modules/selected/index.ts"
    )));
    assert!(!contains_node_modules(Path::new(
        "src/my_node_modules/index.ts"
    )));
    assert!(!contains_node_modules(Path::new("src/node_modules.ts")));
    #[cfg(windows)]
    assert!(contains_node_modules(Path::new(
        "NODE_MODULES/selected/index.ts"
    )));
    #[cfg(not(windows))]
    assert!(!contains_node_modules(Path::new(
        "NODE_MODULES/selected/index.ts"
    )));
}

#[test]
fn ordinary_roots_keep_the_existing_persistent_namespace() {
    let root = tempfile::tempdir().unwrap();
    let selected = source(root.path(), "src/index.ts");
    let mut owner = checker(root.path());
    let expected = crate::batch::project_virtual_root(root.path());
    owner.scan_paths(&[selected]).unwrap();
    assert!(owner.owned_storage.is_none());
    assert_eq!(owner.project.virtual_root(), expected);
}

#[test]
fn installed_owner_reuses_private_paths_and_cleans_successful_materialization() {
    let root = tempfile::tempdir().unwrap();
    let selected = source(root.path(), "node_modules/selected/index.ts");
    let raw = std::fs::read(&selected).unwrap();
    let mut owner = checker(root.path());
    owner.scan_paths(std::slice::from_ref(&selected)).unwrap();
    let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
    let namespace = owner.project.virtual_root().to_path_buf();
    assert_eq!(
        namespace
            .ancestors()
            .find(|path| *path == storage.as_path())
            .map(Path::to_path_buf),
        Some(storage.clone())
    );
    assert_ne!(namespace, crate::batch::project_virtual_root(root.path()));
    assert_eq!(namespace.strip_prefix(root.path()).ok(), None);
    assert!(!contains_node_modules(&namespace));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&storage).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    owner.project.materialize().unwrap();
    let virtual_paths = owner
        .project
        .virtual_files_sorted()
        .into_iter()
        .map(|file| file.virtual_path.clone())
        .collect::<Vec<_>>();
    owner.scan_paths(std::slice::from_ref(&selected)).unwrap();
    assert_eq!(owner.project.virtual_root(), namespace);
    assert_eq!(
        owner
            .project
            .virtual_files_sorted()
            .into_iter()
            .map(|file| file.virtual_path.clone())
            .collect::<Vec<_>>(),
        virtual_paths
    );
    assert!(namespace.is_dir());
    drop(owner);
    assert_mirror_removed(&storage);
    assert_eq!(std::fs::read(&selected).unwrap(), raw);
}

#[test]
fn installed_owner_cleans_storage_after_registration_error() {
    let root = tempfile::tempdir().unwrap();
    let invalid = source(root.path(), "node_modules/selected/invalid.ts");
    std::fs::write(&invalid, [0xff]).unwrap();
    let mut owner = checker(root.path());
    assert!(matches!(owner.scan_paths(&[invalid]),
    Err(crate::batch::error::CorsaError::Io(error))
        if error.kind() == std::io::ErrorKind::InvalidData));
    let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
    assert!(storage.is_dir());
    drop(owner);
    assert_mirror_removed(&storage);
}

#[test]
fn installed_owner_cleans_storage_during_unwind() {
    let root = tempfile::tempdir().unwrap();
    let selected = source(root.path(), "node_modules/selected/index.ts");
    let mut storage = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut owner = checker(root.path());
        owner.scan_paths(&[selected]).unwrap();
        owner.project.materialize().unwrap();
        storage = Some(owner.owned_storage.as_ref().unwrap().path().to_path_buf());
        panic!("exercise batch owner unwind");
    }));
    assert!(result.is_err());
    assert_mirror_removed(&storage.unwrap());
}

#[cfg(unix)]
#[test]
fn logical_installed_symlink_roots_still_select_private_storage() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    source(external.path(), "index.ts");
    std::fs::create_dir(root.path().join("node_modules")).unwrap();
    std::os::unix::fs::symlink(external.path(), root.path().join("node_modules/selected")).unwrap();
    let mut owner = checker(root.path());
    owner
        .scan_paths(&[root.path().join("node_modules/selected/index.ts")])
        .unwrap();
    let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
    drop(owner);
    assert_mirror_removed(&storage);
    assert_eq!(
        std::fs::read_link(root.path().join("node_modules/selected")).unwrap(),
        external.path()
    );
}

#[test]
fn private_scoping_preserves_preloaded_authored_package_bindings() {
    let root = tempfile::tempdir().unwrap();
    let selected = source(root.path(), "node_modules/resolved-control/index.ts");
    let package = selected.parent().unwrap().to_path_buf();
    let manifest = package.join("package.json");
    std::fs::write(
        &manifest,
        r#"{"name":"resolved-control","types":"index.ts"}"#,
    )
    .unwrap();
    let mut owner = checker(root.path());
    owner.set_package_routes([crate::PackageRouteBinding {
        importer_path: selected.clone(),
        specifier: "resolved-control".into(),
        occurrence_mode: crate::PackageResolutionMode::Import,
        context: crate::PackageResolutionContext::default(),
        route: Some(crate::PackageRoute {
            source_paths: vec![selected.clone()],
            dependency_paths: vec![],
            source_targets: vec![],
            package_root: package.clone(),
            package_link_root: package,
            manifest_path: manifest.clone(),
            package_name: Some("resolved-control".into()),
            workspace_source: false,
            nested_routes: vec![],
        }),
        invalidation_paths: vec![manifest],
        shared_invalidation_paths: Vec::new(),
    }]);
    let before = owner.project.package_routes_snapshot();
    assert_eq!(before.len(), 1);
    owner.scope_initial_installed_sources(&[selected]).unwrap();
    assert_eq!(owner.project.package_routes_snapshot(), before);
}

#[test]
fn later_ordinary_owner_expansion_retains_the_documented_persistent_boundary() {
    let root = tempfile::tempdir().unwrap();
    let ordinary = source(root.path(), "src/index.ts");
    let installed = source(root.path(), "node_modules/selected/index.ts");
    let mut owner = checker(root.path());
    owner.scan_paths(&[ordinary]).unwrap();
    let namespace = owner.project.virtual_root().to_path_buf();
    owner.scan_paths(&[installed]).unwrap();
    assert!(owner.owned_storage.is_none());
    assert_eq!(owner.project.virtual_root(), namespace);
}

/// A pnpm store and a hoisted workspace root keep installed files outside
/// the project root. They mirror into `__vize_external__`, never below the
/// virtual root's `node_modules`, so they select no private storage.
#[test]
fn installed_files_outside_the_project_root_keep_the_persistent_namespace() {
    let root = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let ordinary = source(root.path(), "src/index.ts");
    let declaration = store.path().join("node_modules/vue/jsx.d.ts");
    std::fs::create_dir_all(declaration.parent().unwrap()).unwrap();
    std::fs::write(&declaration, "export {};\n").unwrap();
    let installed = source(store.path(), "node_modules/selected/index.ts");
    let mut owner = checker(root.path());
    let expected = crate::batch::project_virtual_root(root.path());
    owner
        .scan_paths(&[ordinary, declaration, installed])
        .unwrap();
    assert!(owner.owned_storage.is_none());
    assert_eq!(owner.project.virtual_root(), expected);
}

#[test]
fn only_installed_segments_below_the_project_root_select_private_storage() {
    let root = Path::new("/workspace/apps/web");
    for installed in [
        "/workspace/apps/web/node_modules/selected/index.ts",
        "/workspace/apps/web/packages/ui/node_modules/selected/index.d.ts",
    ] {
        assert!(
            super::is_installed_source(root, Path::new(installed)),
            "{installed}"
        );
    }
    for external in [
        "/workspace/node_modules/.pnpm/vue@3.5.0/node_modules/vue/jsx.d.ts",
        "/workspace/packages/tsconfig/node_modules/vue/jsx.d.ts",
        "/workspace/apps/web/src/node_modules.ts",
        "/workspace/apps/web-node_modules/src/index.ts",
    ] {
        assert!(
            !super::is_installed_source(root, Path::new(external)),
            "{external}"
        );
    }
    // A project below a `node_modules` directory still owns its own one.
    let nested_root = Path::new("/workspace/node_modules/my-app");
    assert!(super::is_installed_source(
        nested_root,
        Path::new("/workspace/node_modules/my-app/node_modules/vue/index.d.ts"),
    ));
    for external in [
        "/workspace/node_modules/other/index.ts",
        "/workspace/node_modules/.pnpm/vue@3.5.0/node_modules/vue/index.d.ts",
    ] {
        assert!(
            !super::is_installed_source(nested_root, Path::new(external)),
            "{external}"
        );
    }
}

#[cfg(unix)]
#[test]
fn canonical_installed_target_selects_private_storage() {
    let root = tempfile::tempdir().unwrap();
    let installed = source(root.path(), "node_modules/selected/index.ts");
    std::fs::create_dir(root.path().join("src")).unwrap();
    let logical = root.path().join("src/installed.ts");
    std::os::unix::fs::symlink(&installed, &logical).unwrap();
    let mut owner = checker(root.path());
    owner.scan_paths(std::slice::from_ref(&logical)).unwrap();
    let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
    drop(owner);
    assert_mirror_removed(&storage);
    assert_eq!(std::fs::read_link(logical).unwrap(), installed);
}
