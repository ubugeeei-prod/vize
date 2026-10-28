#![cfg(feature = "native")]

use std::path::{Path, PathBuf};
use vize_canon::{VirtualProject, project_virtual_root, snapshot_tsconfig_compiler_options};

/// Inspect only this unique project root and its adjacent hash.* entries.
/// Other namespaces are never statted, descended into, read, or deleted.
fn namespace_entries(root: &Path) -> Vec<(PathBuf, &'static str, u64)> {
    fn collect(parent: &Path, path: &Path, entries: &mut Vec<(PathBuf, &'static str, u64)>) {
        let metadata = std::fs::symlink_metadata(path).unwrap();
        let kind = if metadata.is_dir() {
            "directory"
        } else if metadata.file_type().is_symlink() {
            "symlink"
        } else {
            "file"
        };
        entries.push((
            path.strip_prefix(parent).unwrap().to_path_buf(),
            kind,
            metadata.len(),
        ));
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                collect(parent, &entry.unwrap().path(), entries);
            }
        }
    }
    let generated = project_virtual_root(root);
    let parent = generated.parent().unwrap();
    let key = generated.file_name().unwrap().to_str().unwrap();
    let prefix = format!("{key}.");
    let directory = match std::fs::read_dir(parent) {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => panic!("cannot observe {}: {error}", parent.display()),
    };
    let mut entries = Vec::new();
    for entry in directory {
        let entry = entry.unwrap();
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name == key || name.starts_with(prefix.as_str()))
        {
            collect(parent, &entry.path(), &mut entries);
        }
    }
    entries.sort();
    entries
}

/// The public options reader must not touch generated-project storage.
#[test]
fn compiler_options_snapshot_does_not_create_cache_locks() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let control = root.join("live-lease");
    let project = root.join("snapshot");
    std::fs::create_dir(&control).unwrap();
    std::fs::create_dir(&project).unwrap();

    // Corroborate the observer against the real constructor while its plain
    // File lease is alive through the complete snapshot measurement.
    assert!(namespace_entries(&control).is_empty());
    let lease = VirtualProject::new(&control).unwrap();
    std::hint::black_box(&lease);
    let while_live = namespace_entries(&control);
    assert!(
        !while_live.is_empty(),
        "the real live lease must be observable"
    );
    let active = while_live
        .iter()
        .filter(|(path, _, _)| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".active.lock"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        active.len(),
        1,
        "observe the actual constructor's active lease"
    );
    let generated = project_virtual_root(&control);
    let active_path = generated.parent().unwrap().join(&active[0].0);
    let contender = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&active_path)
        .unwrap();
    let blocked_while_alive =
        matches!(contender.try_lock(), Err(std::fs::TryLockError::WouldBlock));
    assert!(
        blocked_while_alive,
        "the constructor must still hold its real lease"
    );
    let config = project.join("tsconfig.json");
    let original = r#"{"compilerOptions":{"strict":true}}"#;
    std::fs::write(&config, original).unwrap();
    let before = namespace_entries(&project);
    assert!(
        before.is_empty(),
        "the snapshot namespace must be untouched"
    );
    let options = snapshot_tsconfig_compiler_options(&project, &config).unwrap();
    let after = namespace_entries(&project);
    // The full candidate namespace has already been observed above. Only now
    // create an observer-owned control lock. GC takes this namespace lock
    // before its active lock, so it cannot race the independent FD's release
    // check after the actual owner's plain File is dropped.
    let observer_lock_path = generated.with_extension("lock");
    let (observer_lock, observer_created_control_lock) = match std::fs::OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&observer_lock_path)
    {
        Ok(file) => (file, true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&observer_lock_path)
                .unwrap(),
            false,
        ),
        Err(error) => panic!("cannot open {}: {error}", observer_lock_path.display()),
    };
    observer_lock.lock().unwrap();
    let with_observer_barrier = namespace_entries(&control);
    drop(lease);
    let available_after_drop = contender.try_lock().is_ok();
    assert!(
        available_after_drop,
        "dropping the actual project releases its lease"
    );
    let after_drop = namespace_entries(&control);
    assert_eq!(
        after_drop, with_observer_barrier,
        "the GC first-lock barrier preserves the complete control observation"
    );
    std::hint::black_box(&observer_lock);
    drop(contender);
    eprintln!(
        "{}",
        serde_json::json!({
            "controlRoot": control,
            "controlWhileLeaseAlive": while_live,
            "controlWithObserverBarrier": with_observer_barrier,
            "controlAfterDrop": after_drop,
            "observerControlLockPath": observer_lock_path,
            "observerCreatedControlLock": observer_created_control_lock,
            "snapshotMeasuredBeforeControlBarrier": true,
            "leaseBlockedWhileAlive": blocked_while_alive,
            "leaseAvailableAfterDrop": available_after_drop,
            "snapshotRoot": project,
            "snapshotBefore": before,
            "snapshotAfter": after,
            "options": options,
        })
    );
    assert_eq!(
        options,
        serde_json::json!({"strict":true})
            .as_object()
            .unwrap()
            .clone()
    );
    assert_eq!(
        after, before,
        "reading compiler options created generated-project cache locks"
    );
    assert_eq!(std::fs::read_to_string(&config).unwrap(), original);
}
