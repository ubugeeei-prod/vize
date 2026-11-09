#![cfg(unix)]

use std::fs;
use std::path::Path;

use super::TypeSourceSnapshot;
#[cfg(target_os = "linux")]
use super::directory_index::Probe;

#[test]
fn proc_names_match_realpath_for_every_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("real/sub")).unwrap();
    fs::write(root.join("real/sub/file.ts"), "").unwrap();
    std::os::unix::fs::symlink(root.join("real"), root.join("linked")).unwrap();
    for path in [
        root.join("real/sub"),
        root.join("real/sub/file.ts"),
        root.join("linked/sub/file.ts"),
        root.join("real/sub/../sub/file.ts"),
        root.join("real//sub"),
        root.join("real/missing"),
        Path::new("relative").to_path_buf(),
    ] {
        assert_eq!(
            super::realpath::canonicalize(&path).ok(),
            fs::canonicalize(&path).ok(),
            "{path:?}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn probes_answer_like_lstat_until_a_directory_earns_a_listing() {
    for sources in [
        TypeSourceSnapshot::default(),
        TypeSourceSnapshot::for_request(),
    ] {
        probe_like_lstat(sources);
    }
}

#[cfg(target_os = "linux")]
fn probe_like_lstat(sources: TypeSourceSnapshot) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::write(root.join("file.ts"), "").unwrap();
    fs::create_dir(root.join("dir.ts")).unwrap();
    std::os::unix::fs::symlink(root.join("file.ts"), root.join("link.ts")).unwrap();
    let probe = |name: &str| sources.probe_in(&root, name.as_ref());
    let listed = || sources.directories.lock().unwrap()[root.as_path()].listed();
    assert_eq!(probe("file.ts"), Probe::Found(root.join("file.ts")));
    assert_eq!(probe("link.ts"), Probe::Unknown);
    assert_eq!(probe("dir.ts"), Probe::Unknown);
    assert_eq!(probe("missing.ts"), Probe::Absent(root.join("missing.ts")));
    assert!(!listed());
    assert_eq!(probe("file.ts"), Probe::Found(root.join("file.ts")));
    assert!(listed());
    // A listing freezes hits only: later files and links still probe.
    fs::write(root.join("later.ts"), "").unwrap();
    assert_eq!(probe("later.ts"), Probe::Found(root.join("later.ts")));
    assert_eq!(probe("link.ts"), Probe::Unknown);
    assert_eq!(probe("missing.ts"), Probe::Absent(root.join("missing.ts")));
}

#[cfg(target_os = "linux")]
#[test]
fn request_snapshots_hold_bounded_handles_until_they_drop() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("app")).unwrap();
    fs::create_dir_all(root.join("shared")).unwrap();
    fs::write(root.join("app/App.vue"), "").unwrap();
    fs::write(root.join("shared/types.ts"), "").unwrap();
    let sources = TypeSourceSnapshot::for_request();
    assert_eq!(
        sources.resolve_import(&root.join("app/App.vue"), "../shared/types"),
        Some(root.join("shared/types.ts"))
    );
    // The root, the walked `shared` and the found module.
    assert_eq!(sources.held.len(), 3);
    for index in 0..100 {
        let name = vize_carton::cstr!("f{index}.ts");
        fs::write(root.join("shared").join(name.as_str()), "").unwrap();
        sources.probe_in(&root.join("shared"), name.as_str().as_ref());
    }
    assert_eq!(sources.held.len(), 64);
    // Long-lived snapshots hold nothing.
    let editor = TypeSourceSnapshot::default();
    editor.resolve_import(&root.join("app/App.vue"), "../shared/types");
    assert_eq!(editor.held.len(), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn a_second_root_shares_a_request_snapshot_without_holding_more() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("app")).unwrap();
    fs::create_dir_all(root.join("shared")).unwrap();
    for name in ["First.vue", "Second.vue"] {
        fs::write(root.join("app").join(name), "").unwrap();
    }
    fs::write(root.join("shared/types.ts"), "").unwrap();
    fs::write(root.join("shared/other.ts"), "").unwrap();
    let sources = TypeSourceSnapshot::for_request();
    let types = Some(root.join("shared/types.ts"));
    assert_eq!(
        sources.resolve_import(&root.join("app/First.vue"), "../shared/types"),
        types
    );
    assert_eq!(
        sources.resolve_import(&root.join("app/First.vue"), "../shared/types"),
        types
    );
    assert!(sources.held.enabled());
    assert_eq!(sources.held.len(), 3);
    // A host burst: the second root shares the snapshot, which lists and
    // stats as other snapshots do and holds nothing more.
    assert_eq!(
        sources.resolve_import(&root.join("app/Second.vue"), "../shared/other"),
        Some(root.join("shared/other.ts"))
    );
    assert!(!sources.held.enabled());
    assert_eq!(sources.held.len(), 3);
}

#[test]
fn linked_roots_keep_their_target_identity_and_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("real")).unwrap();
    fs::create_dir_all(root.join("app")).unwrap();
    fs::write(root.join("real/App.vue"), "").unwrap();
    fs::write(root.join("real/api.ts"), "").unwrap();
    std::os::unix::fs::symlink(root.join("real/App.vue"), root.join("app/App.vue")).unwrap();
    let sources = TypeSourceSnapshot::default();
    let root_file = root.join("app/App.vue");
    assert_eq!(
        sources.public_identity(&root_file),
        root.join("real/App.vue")
    );
    assert_eq!(
        sources.resolve_import(&root_file, "./api"),
        Some(root.join("real/api.ts"))
    );
}
