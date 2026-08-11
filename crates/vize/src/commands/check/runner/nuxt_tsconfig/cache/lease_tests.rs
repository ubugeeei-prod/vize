use super::{CacheLock, acquire, collect, collect_projects};
use crate::commands::check::runner::nuxt_tsconfig::cache::ownership::{
    ensure_bucket, ensure_entry, ensure_project,
};
use std::fs;

#[test]
fn acquisition_removes_an_unlocked_lease_even_when_its_pid_is_live() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let digest = format!("{:064x}", 0);
    let bucket = ensure_bucket(&cache, "00").unwrap();
    let project = ensure_project(&bucket, &digest).unwrap();
    let entry = ensure_entry(&project, &digest).unwrap();
    let orphan = entry.join(format!(".lease-{}-orphan", std::process::id()));
    fs::write(&orphan, "dead reader").unwrap();

    let (_, _, lease) = acquire(&cache, &digest, &digest).unwrap();
    assert!(!orphan.exists());
    drop(lease);
}

#[test]
fn acquisitions_in_different_shards_never_share_a_lock() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let alpha = ensure_bucket(&cache, "00").unwrap();
    let _alpha_lock = CacheLock::acquire(&alpha.join(".gc.lock"), "alpha shard").unwrap();
    let (sent, received) = std::sync::mpsc::channel();
    let task = std::thread::spawn(move || {
        let digest = format!("ff{:062x}", 1);
        let (_, _, lease) = acquire(&cache, &digest, &format!("{:064x}", 1)).unwrap();
        sent.send(()).unwrap();
        drop(lease);
    });
    received
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("an unrelated digest shard must not wait for alpha's lock");
    task.join().unwrap();
}

#[test]
fn collection_survives_a_directory_created_before_its_ownership_marker() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let project_digest = format!("00{:062x}", 0);
    let (project, entry, lease) = acquire(&cache, &project_digest, &format!("{:064x}", 0)).unwrap();
    fs::create_dir(project.join(format!("{:064x}", 7))).unwrap();
    fs::create_dir(project.parent().unwrap().join(format!("00{:062x}", 8))).unwrap();

    collect(&project, &entry).unwrap();
    collect_projects(&cache, &project).unwrap();
    assert!(entry.exists());
    drop(lease);
}

#[test]
fn collection_is_project_local_bounded_and_preserves_live_readers() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let project_digest = format!("{:064x}", 0);
    let (project, held_entry, held) =
        acquire(&cache, &project_digest, &format!("{:064x}", 0)).unwrap();
    for index in 1..12 {
        drop(acquire(&cache, &project_digest, &format!("{index:064x}")).unwrap());
    }
    let current = project.join(format!("{:064x}", 11));
    let (_, _, current_lease) = acquire(&cache, &project_digest, &format!("{:064x}", 11)).unwrap();

    collect(&project, &current).unwrap();
    assert!(held_entry.exists());
    assert!(entry_count(&project) <= 10);

    drop(held);
    drop(current_lease);
    collect(&project, &current).unwrap();
    assert!(entry_count(&project) <= 9);
}

#[test]
fn collection_ignores_an_unmarked_crash_window_directory() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let digest = format!("{:064x}", 0);
    let (project, current, lease) = acquire(&cache, &digest, &digest).unwrap();
    let incomplete = project.join(format!("{:064x}", 1));
    fs::create_dir(&incomplete).unwrap();

    collect(&project, &current).unwrap();

    assert!(incomplete.exists(), "unknown directories are not deleted");
    drop(lease);
}

#[test]
fn collection_bounds_project_roots_per_shard_and_preserves_live_readers() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let held_digest = format!("00{:062x}", 0);
    let (held_project, _, held) = acquire(&cache, &held_digest, &format!("{:064x}", 0)).unwrap();
    let mut current_project = held_project.clone();
    for index in 1..12 {
        let project_digest = format!("00{index:062x}");
        let (project, _, lease) =
            acquire(&cache, &project_digest, &format!("{index:064x}")).unwrap();
        drop(lease);
        current_project = project;
    }

    collect_projects(&cache, &current_project).unwrap();
    let bucket = current_project.parent().unwrap();
    assert!(held_project.exists());
    assert!(project_count(bucket) <= 10);

    drop(held);
    collect_projects(&cache, &current_project).unwrap();
    assert!(project_count(bucket) <= 9);
}

#[test]
fn collection_work_does_not_grow_with_unrelated_project_shards() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let current_digest = format!("00{:062x}", 0);
    let (current_project, _, current_lease) =
        acquire(&cache, &current_digest, &format!("{:064x}", 0)).unwrap();
    assert_eq!(collect_projects(&cache, &current_project).unwrap(), 1);

    for shard in 1..=32 {
        let shard = format!("{shard:02x}");
        let bucket = ensure_bucket(&cache, &shard).unwrap();
        for project in 0..2 {
            ensure_project(&bucket, &format!("{shard}{project:062x}")).unwrap();
        }
    }
    assert_eq!(
        collect_projects(&cache, &current_project).unwrap(),
        1,
        "warm collection must inspect only the affected digest shard"
    );
    drop(current_lease);
}

#[test]
fn project_collection_retains_a_recently_reused_cache() {
    let case = tempfile::tempdir().unwrap();
    let cache = case.path().join("cache");
    fs::create_dir(&cache).unwrap();
    let mut projects = Vec::new();
    for index in 0..11 {
        let digest = format!("00{index:062x}");
        let (project, _, lease) = acquire(&cache, &digest, &format!("{index:064x}")).unwrap();
        drop(lease);
        fs::write(project.join(".last-used"), format!("{index:039}\n")).unwrap();
        projects.push(project);
    }
    fs::write(projects[0].join(".last-used"), format!("{:039}\n", 1000)).unwrap();

    collect_projects(&cache, &projects[10]).unwrap();
    assert!(
        projects[0].exists(),
        "a hot cache must survive LRU collection"
    );
    assert!(
        !projects[1].exists(),
        "the oldest inactive cache is collected"
    );
}

fn entry_count(project: &std::path::Path) -> usize {
    fs::read_dir(project)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .count()
}

fn project_count(bucket: &std::path::Path) -> usize {
    fs::read_dir(bucket)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .count()
}
