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
    let storage = persistent_batch_storage_root(&app);
    let ordinary = crate::batch::project_virtual_root(&app);
    assert_eq!(first.parent().unwrap(), storage.join("canon/projects"));
    assert_eq!(
        storage.parent().unwrap().file_name(),
        Some(std::ffi::OsStr::new("private-batch-v1"))
    );
    assert_eq!(
        storage.parent().unwrap().parent().unwrap(),
        ordinary.parent().unwrap().parent().unwrap()
    );
    assert_eq!(first.file_name().unwrap().as_encoded_bytes().len(), 64);
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
    let _context_lease = project
        .scope_persistent_batch_namespace(namespace.clone())
        .unwrap();
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
    let _context_lease = project
        .scope_persistent_batch_namespace(namespace.clone())
        .unwrap();
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
    let _context_lease = project.scope_persistent_batch_namespace(namespace).unwrap();
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

#[cfg(unix)]
#[test]
fn owner_lease_precedes_owned_directory_writes_and_blocks_gc() {
    use std::fs::{File, FileTimes, OpenOptions, TryLockError};
    use std::os::unix::fs::PermissionsExt;

    let temporary = tempfile::tempdir().unwrap();
    let storage = temporary
        .path()
        .join("canon/private-batch-v1")
        .join(format!("{:064x}", u64::MAX));
    let lease = super::prepare_persistent_storage(storage.clone(), |storage| {
        assert!(storage.is_dir());
        assert!(!storage.join("canon").exists());
        let active = storage.with_file_name(format!(
            "{}.active.lock",
            storage.file_name().unwrap().to_str().unwrap()
        ));
        {
            let _gc_guard =
                super::super::super::cache_lock::try_acquire_exclusive(storage).unwrap();
            let active = OpenOptions::new()
                .read(true)
                .write(true)
                .open(active)
                .expect("the owner lease must precede the first owned directory");
            assert!(matches!(active.try_lock(), Err(TryLockError::WouldBlock)));
        }
        // Make this owner independently eligible for eviction, then run the
        // real bounded GC while initialization has not created its contexts.
        File::open(storage)
            .unwrap()
            .set_times(FileTimes::new().set_modified(std::time::SystemTime::UNIX_EPOCH))
            .unwrap();
        let owners = storage.parent().unwrap();
        for index in 0..27 {
            std::fs::create_dir_all(owners.join(format!("{index:064x}"))).unwrap();
        }
        super::super::cache_gc::maintain_project_cache(&owners.join(format!("{:064x}", 26)));
        assert!(storage.is_dir(), "GC must retain the initializing owner");
    })
    .unwrap();
    assert!(storage.join("canon/projects").is_dir());
    assert_eq!(
        std::fs::metadata(&storage).unwrap().permissions().mode() & 0o777,
        0o700
    );
    drop(lease);
    let active = storage.with_file_name(format!(
        "{}.active.lock",
        storage.file_name().unwrap().to_str().unwrap()
    ));
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(active)
        .unwrap()
        .try_lock()
        .unwrap();
}
