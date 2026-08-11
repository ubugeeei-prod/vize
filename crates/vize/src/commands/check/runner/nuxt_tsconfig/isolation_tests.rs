use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::symlink,
    process::{Command, Stdio},
    sync::{Arc, Barrier},
};

use super::{
    publish_config_atomically, publish_config_atomically_with_hook, validate_config_cache_root,
    write_nuxt_fallback_tsconfig_in_cache,
};
use crate::commands::check::nuxt::NuxtPathAlias;

#[test]
fn shared_dependency_storage_does_not_share_generated_configs() {
    let case = tempfile::tempdir().unwrap();
    let shared_modules = case.path().join("shared-node-modules");
    let cache = case.path().join("vize-cache");
    fs::create_dir_all(&shared_modules).unwrap();

    let alpha = case.path().join("alpha");
    let bravo = case.path().join("bravo");
    for project in [&alpha, &bravo] {
        fs::create_dir_all(project).unwrap();
        symlink(&shared_modules, project.join("node_modules")).unwrap();
    }

    for round in 0..100 {
        let start = Arc::new(Barrier::new(4));
        let alpha_task = prepare_after_barrier(
            Arc::clone(&start),
            alpha.clone(),
            cache.clone(),
            format!("alpha-{round}/*"),
        );
        let repeated_alpha_task = prepare_after_barrier(
            Arc::clone(&start),
            alpha.clone(),
            cache.clone(),
            format!("alpha-{round}/*"),
        );
        let bravo_task = prepare_after_barrier(
            Arc::clone(&start),
            bravo.clone(),
            cache.clone(),
            format!("bravo-{round}/*"),
        );
        start.wait();

        let alpha_config = alpha_task.join().unwrap();
        let repeated_alpha_config = repeated_alpha_task.join().unwrap();
        let bravo_config = bravo_task.join().unwrap();
        let alpha_path = alpha_config.path().unwrap().to_path_buf();
        let bravo_path = bravo_config.path().unwrap().to_path_buf();
        let alpha_content = fs::read(&alpha_path).unwrap();

        assert_ne!(alpha_path, bravo_path);
        assert_eq!(alpha_path, repeated_alpha_config.path().unwrap());
        assert!(String::from_utf8_lossy(&alpha_content).contains(&format!("alpha-{round}")));
        assert!(
            String::from_utf8_lossy(&fs::read(&bravo_path).unwrap())
                .contains(&format!("bravo-{round}"))
        );
        assert_eq!(fs::read(&alpha_path).unwrap(), alpha_content);

        drop(alpha_config);
        drop(repeated_alpha_config);
        assert!(
            alpha_path.exists(),
            "published configs are immutable cache entries"
        );
        assert!(
            bravo_path.exists(),
            "one session must not invalidate a sibling"
        );
        drop(bravo_config);
        assert!(
            bravo_path.exists(),
            "published configs survive process cleanup"
        );
    }

    assert_eq!(
        fs::read_dir(shared_modules).unwrap().count(),
        0,
        "prepared configs must never mutate shared dependency storage"
    );
    assert_eq!(
        pending_count(&cache),
        0,
        "successful concurrent publication must leave no pending files"
    );
    for bucket in cache_directories(&cache) {
        assert!(
            cache_directories(&bucket).len() <= 9,
            "one shard retains at most the current plus eight inactive projects"
        );
        for project in cache_directories(&bucket) {
            assert!(
                cache_directories(&project).len() <= 9,
                "one project retains at most the current plus eight inactive configs"
            );
        }
    }
}

#[test]
fn absolute_alias_targets_survive_logical_and_physical_symlink_spellings() {
    let case = tempfile::tempdir().unwrap();
    let physical = case.path().join("physical-project");
    let logical = case.path().join("logical-project");
    fs::create_dir_all(physical.join("src")).unwrap();
    fs::write(physical.join("src/value.ts"), "export const value = 1;\n").unwrap();
    symlink(&physical, &logical).unwrap();

    let prepared = write_nuxt_fallback_tsconfig_in_cache(
        None,
        &logical,
        &logical,
        &[NuxtPathAlias {
            pattern: "~value".into(),
            targets: vec!["src/value.ts".into()],
        }],
        &case.path().join("cache"),
    )
    .unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared.path().unwrap()).unwrap()).unwrap();
    let target = value["compilerOptions"]["paths"]["~value"][0]
        .as_str()
        .unwrap();

    assert_eq!(
        fs::canonicalize(target).unwrap(),
        fs::canonicalize(physical.join("src/value.ts")).unwrap()
    );
    assert_eq!(
        fs::read_to_string(target).unwrap(),
        "export const value = 1;\n"
    );
}

#[test]
fn cache_location_and_published_bytes_fail_closed() {
    let case = tempfile::tempdir().unwrap();
    let project = case.path().join("project");
    let dependency_cache = project.join("node_modules/.vize/check");
    fs::create_dir_all(&dependency_cache).unwrap();
    assert!(validate_config_cache_root(&dependency_cache, &project).is_err());

    let cache = case.path().join("cache");
    let prepared = write_nuxt_fallback_tsconfig_in_cache(
        None,
        &project,
        &project,
        &[NuxtPathAlias {
            pattern: "~/*".into(),
            targets: vec!["src/*".into()],
        }],
        &cache,
    )
    .unwrap();
    fs::write(prepared.path().unwrap(), "corrupt\n").unwrap();

    let error = write_nuxt_fallback_tsconfig_in_cache(
        None,
        &project,
        &project,
        &[NuxtPathAlias {
            pattern: "~/*".into(),
            targets: vec!["src/*".into()],
        }],
        &cache,
    )
    .err()
    .expect("corrupt immutable cache entry must fail closed");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn killed_publisher_is_cleaned_before_the_next_publisher() {
    let case = tempfile::tempdir().unwrap();
    let final_path = case.path().join("tsconfig.nuxt-fallback.crash.json");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "commands::check::runner::nuxt_tsconfig::isolation_tests::publication_child_waits_after_closing_pending_file",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("VIZE_NUXT_PUBLICATION_CHILD_PATH", &final_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut output = BufReader::new(stdout);
    let mut ready = std::string::String::new();
    loop {
        ready.clear();
        assert_ne!(output.read_line(&mut ready).unwrap(), 0);
        if ready.contains("pending-ready") {
            break;
        }
    }
    child.kill().unwrap();
    child.wait().unwrap();

    assert_eq!(pending_count(case.path()), 1);
    fs::write(
        case.path().join(format!(
            ".vize-nuxt-config-{}-reused.pending",
            std::process::id()
        )),
        "stale same-PID state",
    )
    .unwrap();
    publish_config_atomically(&final_path, b"stable\n").unwrap();
    assert_eq!(fs::read(&final_path).unwrap(), b"stable\n");
    assert_eq!(pending_count(case.path()), 0);
}

#[test]
#[ignore = "subprocess helper for killed_publisher_is_cleaned_before_the_next_publisher"]
fn publication_child_waits_after_closing_pending_file() {
    let path = std::env::var_os("VIZE_NUXT_PUBLICATION_CHILD_PATH").unwrap();
    publish_config_atomically_with_hook(std::path::Path::new(&path), b"never-published\n", |_| {
        println!("pending-ready");
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    })
    .unwrap();
}

fn pending_count(directory: &std::path::Path) -> usize {
    fs::read_dir(directory)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| {
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                return pending_count(&entry.path());
            }
            usize::from(
                entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.ends_with(".pending")),
            )
        })
        .sum()
}

fn cache_directories(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(path)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .collect()
}

fn prepare_after_barrier(
    start: Arc<Barrier>,
    project: std::path::PathBuf,
    cache: std::path::PathBuf,
    target: String,
) -> std::thread::JoinHandle<super::PreparedCheckerTsconfig> {
    std::thread::spawn(move || {
        start.wait();
        write_nuxt_fallback_tsconfig_in_cache(
            None,
            &project,
            &project,
            &[NuxtPathAlias {
                pattern: "~/*".into(),
                targets: vec![target.into()],
            }],
            &cache,
        )
        .unwrap()
    })
}
