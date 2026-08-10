use super::{MaterializedDelta, MaterializedSnapshot, VirtualProject};
use crate::batch::source_policy::SourceFilePolicy;
use std::path::PathBuf;
use vize_carton::FxHashMap;

#[test]
fn snapshot_diff_classifies_created_changed_and_deleted_files() {
    let previous = snapshot(&[("/virtual/changed.ts", 1), ("/virtual/deleted.ts", 2)]);
    let current = snapshot(&[("/virtual/changed.ts", 3), ("/virtual/created.ts", 4)]);

    assert_eq!(
        current.diff(&previous),
        MaterializedDelta {
            changed: vec![PathBuf::from("/virtual/changed.ts")],
            created: vec![PathBuf::from("/virtual/created.ts")],
            deleted: vec![PathBuf::from("/virtual/deleted.ts")],
        }
    );
}

#[cfg(unix)]
#[test]
fn snapshot_keeps_symlinked_typescript_files_as_diagnostic_inputs() {
    use crate::file_uri::path_to_file_uri;
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("temporary project should be created");
    let virtual_root = temp.path().join("canon");
    let target = temp.path().join("external.ts");
    let linked = virtual_root.join("linked.ts");
    std::fs::create_dir_all(&virtual_root).expect("virtual root should be created");
    std::fs::write(&target, "export const value = 1\n").expect("target should be written");
    symlink(&target, &linked).expect("source symlink should be created");

    let before = MaterializedSnapshot::capture(&virtual_root, SourceFilePolicy::default())
        .expect("materialized snapshot should be captured");
    std::fs::write(&target, "export const value = 2\n").expect("target should be updated");
    let after = MaterializedSnapshot::capture(&virtual_root, SourceFilePolicy::default())
        .expect("updated materialized snapshot should be captured");

    assert!(after.revisions.contains_key(&linked));
    assert_eq!(after.uris, vec![path_to_file_uri(&linked)]);
    assert_eq!(after.diff(&before).changed, vec![linked]);
}

#[test]
fn snapshot_tracks_in_place_diagnostic_source_changes() {
    use crate::file_uri::path_to_file_uri;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("aliased.ts");
    std::fs::write(&source, "export const value = 1\n").unwrap();
    let mut project = VirtualProject::new(temp.path()).unwrap();
    std::fs::create_dir_all(project.virtual_root()).unwrap();
    project.set_diagnostic_paths([source.as_path()]);
    let mut before =
        MaterializedSnapshot::capture(project.virtual_root(), SourceFilePolicy::default()).unwrap();
    before.extend_diagnostic_paths(&project).unwrap();

    std::fs::write(&source, "export const value = 2\n").unwrap();
    let mut after =
        MaterializedSnapshot::capture(project.virtual_root(), SourceFilePolicy::default()).unwrap();
    after.extend_diagnostic_paths(&project).unwrap();

    let source = source.canonicalize().unwrap();
    assert_eq!(after.uris, vec![path_to_file_uri(&source)]);
    assert_eq!(after.diff(&before).changed, vec![source]);
}

fn snapshot(entries: &[(&str, u64)]) -> MaterializedSnapshot {
    MaterializedSnapshot {
        revisions: entries
            .iter()
            .map(|(path, revision)| (PathBuf::from(path), *revision))
            .collect::<FxHashMap<_, _>>(),
        uris: Vec::new(),
    }
}
