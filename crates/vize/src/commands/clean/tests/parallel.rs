//! Exercise a cold shared cache without mutating the parent test environment.

use super::super::{CleanArgs, CleanScope, run};
use std::path::Path;
use std::sync::Barrier;

const CHILD_ROOT_ENV: &str = "VIZE_CLEAN_CACHE_TEST_ROOT";

#[test]
fn shared_cache_containers_survive_clean_and_parallel_materialization() {
    let temp = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "commands::clean::tests::parallel::cold_cache_child",
            "--nocapture",
        ])
        .env(CHILD_ROOT_ENV, temp.path())
        .env("XDG_CACHE_HOME", temp.path().join("cache"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cold-cache child failed:\nstdout: {}\nstderr: {}",
        std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>"),
        std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>")
    );
    assert!(
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .contains("parallel-clean-cold-cache-complete"),
        "the subprocess did not complete the regression helper"
    );
}

#[test]
fn cold_cache_child() {
    let Some(root) = std::env::var_os(CHILD_ROOT_ENV) else {
        return;
    };
    let root = Path::new(&root);
    let first = root.join("first");
    std::fs::create_dir_all(&first).unwrap();
    let current = vize_canon::project_virtual_root(&first);
    assert!(current.starts_with(root.join("cache")));
    let projects = current.parent().unwrap();
    let canon = projects.parent().unwrap();
    std::fs::create_dir_all(&current).unwrap();

    // This cache is initially empty apart from the selected namespace. A
    // production clean must retain the shared containers, even with --force.
    clean(&first, CleanScope::All, true);
    assert!(!current.exists());
    assert!(
        projects.is_dir(),
        "clean removed the shared projects directory"
    );
    assert!(canon.is_dir(), "clean removed the shared Canon directory");

    let barrier = Barrier::new(8);
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for index in 0..8 {
            let project = root.join(vize_carton::cstr!("{index}").as_str());
            std::fs::create_dir_all(&project).unwrap();
            let barrier = &barrier;
            workers.push(scope.spawn(move || {
                let current = vize_canon::project_virtual_root(&project);
                let locks = vize_canon::project_virtual_lock_paths(&project);
                barrier.wait();
                for iteration in 0..64 {
                    std::fs::create_dir_all(&current).unwrap();
                    std::fs::write(current.join("current.ts"), "current").unwrap();
                    for lock in &locks {
                        std::fs::write(lock, "current").unwrap();
                    }
                    let scope = if iteration % 2 == 0 {
                        CleanScope::All
                    } else {
                        CleanScope::Project
                    };
                    clean(&project, scope, iteration % 3 == 0);
                    assert!(!current.exists());
                    assert!(locks.iter().all(|lock| !lock.exists()));
                }
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
    });
    assert!(projects.is_dir());
    assert!(canon.is_dir());
    assert_eq!(std::fs::read_dir(projects).unwrap().count(), 0);
    println!("parallel-clean-cold-cache-complete");
}

fn clean(root: &Path, scope: CleanScope, force: bool) {
    run(CleanArgs {
        root: root.to_path_buf(),
        scope,
        force,
        dry_run: false,
        quiet: true,
    });
}
