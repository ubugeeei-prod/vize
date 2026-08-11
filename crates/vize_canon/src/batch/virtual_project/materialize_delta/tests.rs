use std::fs;
use std::path::PathBuf;

use super::{MaterializedFileSnapshot, materialize_package_links, remove_stale_package_links};
use vize_carton::{FxHashMap, FxHashSet};

fn snapshot(entries: &[(&str, u64)]) -> MaterializedFileSnapshot {
    MaterializedFileSnapshot {
        revisions: entries
            .iter()
            .map(|(path, revision)| (PathBuf::from(path), *revision))
            .collect::<FxHashMap<_, _>>(),
        package_links: FxHashMap::default(),
    }
}

#[test]
fn diff_classifies_editor_materialization_changes() {
    let previous = snapshot(&[("/mirror/changed.ts", 1), ("/mirror/deleted.ts", 2)]);
    let current = snapshot(&[("/mirror/changed.ts", 3), ("/mirror/created.ts", 4)]);
    let delta = current.diff(&previous);

    assert_eq!(delta.changed, vec![PathBuf::from("/mirror/changed.ts")]);
    assert_eq!(delta.created, vec![PathBuf::from("/mirror/created.ts")]);
    assert_eq!(delta.deleted, vec![PathBuf::from("/mirror/deleted.ts")]);
    assert!(!delta.is_empty());
}

#[test]
fn changed_project_and_package_configs_are_topology_changes() {
    for path in [
        "/mirror/tsconfig.json",
        "/mirror/node_modules/pkg/package.json",
    ] {
        let delta = super::MaterializedFileDelta {
            changed: vec![PathBuf::from(path)],
            ..Default::default()
        };
        assert!(delta.has_topology_changes(), "{path}");
    }
    let source = super::MaterializedFileDelta {
        changed: vec![PathBuf::from("/mirror/src/App.vue.ts")],
        ..Default::default()
    };
    assert!(!source.has_topology_changes());
}

#[test]
fn editor_snapshot_reports_package_link_retarget_as_topology() {
    let mut previous = snapshot(&[]);
    previous.package_links.insert(
        PathBuf::from("/mirror/node_modules/pkg"),
        PathBuf::from("/store/pkg-v1"),
    );
    let mut current = snapshot(&[]);
    current.package_links.insert(
        PathBuf::from("/mirror/node_modules/pkg"),
        PathBuf::from("/store/pkg-v2"),
    );

    let delta = current.diff(&previous);
    assert_eq!(
        delta.changed,
        vec![PathBuf::from("/mirror/node_modules/pkg")]
    );
    assert!(delta.topology_changed);
    assert!(delta.has_topology_changes());
}

#[test]
fn capture_hashes_only_the_owned_expected_files() {
    let root = tempfile::tempdir().unwrap();
    let owned = root.path().join("owned.ts");
    let unrelated = root.path().join("unrelated.ts");
    std::fs::write(&owned, "export const owned = 1\n").unwrap();
    std::fs::write(&unrelated, "export const unrelated = 1\n").unwrap();

    let snapshot =
        MaterializedFileSnapshot::capture(&FxHashSet::from_iter([owned.clone()])).unwrap();

    assert!(snapshot.revisions.contains_key(&owned));
    assert!(!snapshot.revisions.contains_key(&unrelated));
}

#[test]
fn incremental_package_links_create_retarget_and_remove_exactly() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first/node_modules");
    let second = root.path().join("second/node_modules");
    let mirror = root.path().join("mirror/node_modules");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();

    let first_links = FxHashMap::from_iter([(mirror.clone(), first.clone())]);
    materialize_package_links(&first_links).unwrap();
    assert_eq!(
        fs::read_link(&mirror).unwrap(),
        first.canonicalize().unwrap()
    );

    let second_links = FxHashMap::from_iter([(mirror.clone(), second.clone())]);
    remove_stale_package_links(&first_links, &second_links).unwrap();
    assert!(!mirror.exists());
    materialize_package_links(&second_links).unwrap();
    assert_eq!(
        fs::read_link(&mirror).unwrap(),
        second.canonicalize().unwrap()
    );

    remove_stale_package_links(&second_links, &FxHashMap::default()).unwrap();
    assert!(!mirror.exists());
}

#[cfg(unix)]
#[test]
fn persistent_project_retargets_and_removes_only_the_affected_link_scope() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let project_root = root.path().join("workspace");
    let source = project_root.join("apps/app/src/entry.ts");
    let real_link = project_root.join("apps/app/node_modules");
    let first = root.path().join("store/first");
    let second = root.path().join("store/second");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    symlink(&first, &real_link).unwrap();
    fs::write(&source, "export const value = 1\n").unwrap();
    let source = source.canonicalize().unwrap();

    let mut project = crate::batch::virtual_project::VirtualProject::new(&project_root).unwrap();
    project.register_path(&source).unwrap();
    project.materialize().unwrap();
    project.capture_materialized_package_links();
    project.discard_incremental_materialization();
    let first = first.canonicalize().unwrap();
    let mirror_link = project
        .materialized_package_links
        .iter()
        .find_map(|(path, target)| (target == &first).then(|| path.clone()))
        .unwrap_or_else(|| {
            panic!(
                "missing package link for {} from {:?}",
                project.find_by_original(&source).map_or_else(
                    || source.display().to_string(),
                    |file| file.virtual_path.display().to_string()
                ),
                project.materialized_package_links
            )
        });
    assert_eq!(fs::read_link(&mirror_link).unwrap(), first);

    fs::remove_file(&real_link).unwrap();
    symlink(&second, &real_link).unwrap();
    fs::write(&source, "export const value = 2\n").unwrap();
    project.register_path(&source).unwrap();
    let retargeted = project.materialize_incremental_delta().unwrap();
    assert!(!retargeted.full_topology_rebuild);
    assert!(retargeted.delta.topology_changed);
    assert!(retargeted.delta.changed.contains(&mirror_link));
    assert_eq!(
        fs::read_link(&mirror_link).unwrap(),
        second.canonicalize().unwrap()
    );

    project.remove_registered_source(&source);
    let removed = project.materialize_incremental_delta().unwrap();
    assert!(!removed.full_topology_rebuild);
    assert!(removed.delta.topology_changed);
    assert!(removed.delta.deleted.contains(&mirror_link));
    assert!(!mirror_link.exists());
}
