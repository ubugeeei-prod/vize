use std::path::Path;

use vize_carton::FxHashSet;

use super::{persistent_batch_storage_root, persistent_batch_virtual_root};
use crate::batch::VirtualProject;

#[test]
fn private_storage_separates_owners_resolution_roots_and_configs() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let other = root.path().join("other");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    let config = app.join("tsconfig.json");
    std::fs::write(&config, "{}").unwrap();
    let first = persistent_batch_virtual_root(root.path(), &app, Some(&config));
    assert_eq!(
        first,
        persistent_batch_virtual_root(root.path(), &app, Some(&config))
    );
    assert_ne!(
        first,
        persistent_batch_virtual_root(root.path(), &other, Some(&config))
    );
    assert_ne!(
        first,
        persistent_batch_virtual_root(&app, &app, Some(&config))
    );
    assert_ne!(
        first,
        persistent_batch_virtual_root(root.path(), &app, None)
    );
    assert!(!first.starts_with(crate::batch::project_virtual_root(&app)));
    assert!(first.starts_with(persistent_batch_storage_root(&app)));
    assert!(!first.starts_with(root.path()));
    assert_eq!(
        first.parent().unwrap().parent().unwrap().file_name(),
        Some(std::ffi::OsStr::new("canon"))
    );
}

#[cfg(unix)]
#[test]
fn owned_write_ancestors_detach_stale_install_links_without_touching_targets() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    let installed = temp.path().join("installed");
    let namespace = temp.path().join("private");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&installed).unwrap();
    std::fs::create_dir_all(namespace.join("node_modules")).unwrap();
    let original = installed.join("index.ts");
    std::fs::write(&original, b"export const installed = 'immutable';\n").unwrap();
    let modified = std::fs::metadata(&original).unwrap().modified().unwrap();
    std::os::unix::fs::symlink(&installed, namespace.join("node_modules/package")).unwrap();
    let owned = namespace.join("node_modules/package/index.ts");
    let mut project = VirtualProject::new(&root).unwrap();
    project.scope_persistent_batch_namespace(namespace.clone());
    project
        .prepare_owned_batch_writes(&FxHashSet::from_iter([owned.clone()]))
        .unwrap();
    crate::batch::materialize_fs::write_if_changed(&owned, b"export const owned = 1;\n").unwrap();
    assert_eq!(
        std::fs::read(&original).unwrap(),
        b"export const installed = 'immutable';\n"
    );
    assert_eq!(
        std::fs::metadata(&original).unwrap().modified().unwrap(),
        modified
    );
    assert_eq!(std::fs::read(&owned).unwrap(), b"export const owned = 1;\n");
    assert!(
        !std::fs::symlink_metadata(namespace.join("node_modules/package"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn private_write_authority_rejects_parent_components_and_outside_files() {
    let temp = tempfile::tempdir().unwrap();
    let namespace = temp.path().join("private");
    let mut project = VirtualProject::new(temp.path()).unwrap();
    project.scope_persistent_batch_namespace(namespace.clone());
    for file in [
        namespace.join("../outside.ts"),
        temp.path().join("outside.ts"),
        namespace,
    ] {
        assert_eq!(
            project
                .prepare_owned_batch_writes(&FxHashSet::from_iter([file]))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
    assert!(!Path::new(&temp.path().join("outside.ts")).exists());
}

#[test]
fn native_build_info_outputs_do_not_write_through_existing_hardlinks() {
    let temp = tempfile::tempdir().unwrap();
    let namespace = temp.path().join("private");
    let outputs = namespace.join(super::super::CLI_BUILD_INFO_DIR);
    std::fs::create_dir_all(&outputs).unwrap();
    let original = temp.path().join("installed.json");
    std::fs::write(&original, b"{\"immutable\":true}\n").unwrap();
    let modified = std::fs::metadata(&original).unwrap().modified().unwrap();
    let output = outputs.join("project.tsbuildinfo");
    std::fs::hard_link(&original, &output).unwrap();
    let mut project = VirtualProject::new(temp.path()).unwrap();
    project.scope_persistent_batch_namespace(namespace);
    project
        .prepare_owned_batch_writes(&FxHashSet::default())
        .unwrap();
    std::fs::write(&output, b"{\"native\":\"new state\"}\n").unwrap();
    assert_eq!(std::fs::read(&original).unwrap(), b"{\"immutable\":true}\n");
    assert_eq!(
        std::fs::metadata(&original).unwrap().modified().unwrap(),
        modified
    );
    assert_eq!(
        std::fs::read(&output).unwrap(),
        b"{\"native\":\"new state\"}\n"
    );
}
