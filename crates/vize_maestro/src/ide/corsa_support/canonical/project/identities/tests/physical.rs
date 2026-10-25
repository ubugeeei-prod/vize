use super::{ServerState, UriMembership, assert_sequence, file_uri};
use std::{fs, path::Path, thread};
use vize_l0::cstr;

fn link(target: &Path, alias: &Path) {
    std::os::unix::fs::symlink(target, alias).expect("symlink");
}

#[test]
fn discarded_physical_alias_never_suppresses_a_distinct_virtual_owner() {
    let directory = tempfile::tempdir().expect("directory");
    let real_path = directory.path().join("real.ts");
    let alias_path = directory.path().join("alias.ts");
    fs::write(&real_path, "export {};").expect("source");
    link(&real_path, &alias_path);
    let real = file_uri(&real_path);
    let alias = file_uri(&alias_path);
    let parsed = tower_lsp::lsp_types::Url::parse(&alias).expect("URI");
    let remote = cstr!("https://remote.example{}", parsed.path());
    let legacy = ServerState::new();
    let indexed = ServerState::new();
    assert_sequence(&legacy, &indexed, &[], &[&real, &alias, &remote]);
    let mut owners = UriMembership::candidate_first([]);
    assert!(owners.insert_if_absent(&indexed, &real));
    assert!(!owners.insert_if_absent(&indexed, &alias));
    assert!(owners.insert_if_absent(&indexed, &remote));
    assert_eq!(
        owners
            .accepted
            .iter()
            .map(|entry| entry.raw.as_str())
            .collect::<Vec<_>>(),
        vec![real.as_str(), remote.as_str()],
    );
}

#[test]
fn early_raw_hit_does_not_observe_later_seed_symlinks() {
    let directory = tempfile::tempdir().expect("directory");
    let first_path = directory.path().join("first.ts");
    let other_path = directory.path().join("other.ts");
    let alias_path = directory.path().join("alias.ts");
    fs::write(&first_path, "export const first = 1;").expect("first");
    fs::write(&other_path, "export const other = 2;").expect("other");
    link(&first_path, &alias_path);
    let first = file_uri(&first_path);
    let other = file_uri(&other_path);
    let alias = file_uri(&alias_path);
    let state = ServerState::new();
    let mut owners = UriMembership::candidate_first([first.as_str(), alias.as_str()]);
    assert!(owners.contains(&state, &first));
    assert_eq!(owners.work.physical_resolutions, 0);
    fs::remove_file(&alias_path).expect("remove first alias");
    link(&other_path, &alias_path);
    // No notification/clear: the legacy early hit never observed this alias.
    assert!(owners.contains(&state, &other));
}

#[test]
fn observed_symlink_keeps_legacy_sticky_identity_until_cache_clear() {
    let directory = tempfile::tempdir().expect("directory");
    let first_path = directory.path().join("first.ts");
    let other_path = directory.path().join("other.ts");
    let alias_path = directory.path().join("alias.ts");
    fs::write(&first_path, "first").expect("first");
    fs::write(&other_path, "other").expect("other");
    link(&first_path, &alias_path);
    let first = file_uri(&first_path);
    let other = file_uri(&other_path);
    let alias = file_uri(&alias_path);
    let state = ServerState::new();
    let mut owners = UriMembership::candidate_first([alias.as_str()]);
    assert!(owners.contains(&state, &first));
    fs::remove_file(&alias_path).expect("remove alias");
    link(&other_path, &alias_path);
    assert!(!owners.contains(&state, &other), "unnotified legacy cache");

    // Clear on another thread, without an await or a fresh native request.
    // Reusing this exact index must discard its old physical membership.
    thread::scope(|scope| {
        scope
            .spawn(|| state.invalidate_canonical_editor_paths())
            .join()
            .expect("cache clear worker");
    });
    assert!(owners.contains(&state, &other));
    assert_eq!(owners.work.physical_resets, 2);
}
