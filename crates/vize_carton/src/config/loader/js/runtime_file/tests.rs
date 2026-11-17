use super::NodeRuntimeFile;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    thread,
    time::{Duration, Instant},
};

mod benchmark;
mod execution;

const SHARED_PROCESS_SCRIPT: &str = "process.stdout.write('shared process');";
const SYMLINK_PROCESS_SCRIPT: &str = "process.stdout.write('owned');";

fn runtime(cache_home: &Path, script: &str) -> NodeRuntimeFile {
    NodeRuntimeFile::new_with_cache_home(script, cache_home).unwrap()
}

fn fixture_command(cache: &Path, mode: &str, result: &Path) -> Command {
    let mut command = Command::new(env::current_exe().unwrap());
    command
        .arg("runtime_cache_process_fixture")
        .arg("--ignored")
        .arg("--nocapture")
        .env("VIZE_RUNTIME_FIXTURE_CACHE", cache)
        .env("VIZE_RUNTIME_FIXTURE_MODE", mode)
        .env("VIZE_RUNTIME_FIXTURE_RESULT", result);
    command
}

fn wait_for_path(path: &Path, child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if path.exists() {
            return;
        }
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "fixture exited before its ready marker");
            panic!("fixture exited without its ready marker");
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("fixture did not publish its ready marker");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_child(child: &mut Child) -> ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("fixture process did not exit");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_for_signal(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(Instant::now() < deadline, "fixture signal timed out");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn unavailable_cache_uses_a_scoped_fallback_on_success_or_child_failure() {
    let project = tempfile::tempdir().unwrap();
    let config = project.path().join("vite.config.ts");
    for (script, expected_exit) in [
        ("process.stdout.write('ok');", 0),
        ("throw new Error('expected runtime failure');", 1),
    ] {
        let cache = tempfile::tempdir().unwrap();
        fs::write(cache.path().join("vize"), "not a directory").unwrap();
        let path;
        {
            let runtime = runtime(cache.path(), script);
            assert!(!runtime.is_cached());
            path = runtime.path().to_path_buf();
            assert!(path.exists());
            let output = runtime
                .command(&config)
                .output()
                .expect("Node must be installed");
            assert_eq!(output.status.code(), Some(expected_exit));
        }
        assert!(!path.exists(), "scoped runtime file must not leak");
    }
}

#[test]
fn live_runtime_is_reused_and_stale_identities_are_bounded() {
    let cache = tempfile::tempdir().unwrap();
    let first = runtime(cache.path(), "process.stdout.write('same');");
    let first_path = first.path().to_path_buf();
    let repeated = runtime(cache.path(), "process.stdout.write('same');");
    assert_eq!(repeated.path(), first_path);
    drop(repeated);

    for identity in 0..8 {
        drop(runtime(
            cache.path(),
            crate::cstr!("process.stdout.write('{identity}');").as_str(),
        ));
    }
    let entries: Vec<_> = fs::read_dir(super::cache_root(cache.path()))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        entries
            .iter()
            .filter(|path| path.extension().is_some_and(|extension| extension == "mjs"))
            .count(),
        super::CACHE_LIMIT
    );
    assert_eq!(
        entries
            .iter()
            .filter(|path| path
                .extension()
                .is_some_and(|extension| extension == "lease"))
            .count(),
        super::CACHE_LIMIT
    );
    assert_eq!(
        entries
            .iter()
            .filter(|path| path.extension().is_some_and(|extension| extension == "tmp"))
            .count(),
        0
    );
    assert!(first_path.exists(), "an active runtime must not be pruned");
    drop(first);
}

#[test]
fn process_cache_reuses_the_validated_runtime_and_shared_lease() {
    let cache = tempfile::tempdir().unwrap();
    let source = super::runtime_source(SHARED_PROCESS_SCRIPT);
    let first = super::process_runtime(cache.path(), &source).unwrap();
    let second = super::process_runtime(cache.path(), &source).unwrap();
    assert_eq!(first.path(), second.path());
    assert!(super::SharedRuntime::ptr_eq(
        first._lease.as_ref().unwrap(),
        second._lease.as_ref().unwrap()
    ));
    drop(first);
    drop(second);
    super::process_cache()
        .lock()
        .unwrap()
        .retain(|entry| entry.home != cache.path());
}
#[test]
fn another_process_can_reuse_a_live_runtime() {
    let cache = tempfile::tempdir().unwrap();
    let result = cache.path().join("child-runtime-path");
    let held = runtime(cache.path(), SHARED_PROCESS_SCRIPT);
    let mut child = fixture_command(cache.path(), "reuse", &result)
        .spawn()
        .unwrap();
    let status = wait_for_child(&mut child);
    assert!(status.success());
    assert_eq!(
        fs::read_to_string(result).unwrap(),
        held.path().file_name().unwrap().to_str().unwrap()
    );
}

#[test]
#[ignore = "helper process for another_process_can_reuse_a_live_runtime"]
fn runtime_cache_process_fixture() {
    let cache = PathBuf::from(env::var_os("VIZE_RUNTIME_FIXTURE_CACHE").unwrap());
    let result = PathBuf::from(env::var_os("VIZE_RUNTIME_FIXTURE_RESULT").unwrap());
    match env::var("VIZE_RUNTIME_FIXTURE_MODE").unwrap().as_str() {
        "reuse" => {
            let runtime = runtime(&cache, SHARED_PROCESS_SCRIPT);
            fs::write(
                result,
                runtime.path().file_name().unwrap().to_str().unwrap(),
            )
            .unwrap();
        }
        "crash-partial" => {
            let root = super::cache_root(&cache);
            let partial = root.join("crashed.tmp");
            let mut file = super::private_open_options()
                .create_new(true)
                .open(partial)
                .unwrap();
            file.lock().unwrap();
            file.write_all(b"partial").unwrap();
            fs::write(result, b"ready").unwrap();
            thread::sleep(Duration::from_secs(30));
            panic!("crash fixture was not terminated");
        }
        "old-fd" => {
            let root = super::cache_root(&cache);
            let source = super::runtime_source(SHARED_PROCESS_SCRIPT);
            let hash = super::sha256_hex(&source);
            let catalog = super::open_lease(&root.join(super::CATALOG_LEASE)).unwrap();
            catalog.lock_shared().unwrap();
            let lease =
                super::open_lease(&root.join(crate::cstr!("{hash}.lease").as_str())).unwrap();
            fs::write(result.with_extension("ready"), b"ready").unwrap();
            wait_for_signal(&result.with_extension("advance"));
            lease.lock_shared().unwrap();
            catalog.unlock().unwrap();
            fs::write(result.with_extension("locked"), b"locked").unwrap();
            wait_for_signal(&result.with_extension("release"));
            assert!(super::runtime_matches(
                &root.join(crate::cstr!("{hash}.mjs").as_str()),
                &source
            ));
        }
        "symlink" => {
            let runtime = runtime(&cache, SYMLINK_PROCESS_SCRIPT);
            fs::write(
                result,
                if runtime.is_cached() {
                    b"cached".as_slice()
                } else {
                    b"temporary".as_slice()
                },
            )
            .unwrap();
        }
        mode => panic!("unknown fixture mode: {mode}"),
    }
}

#[test]
fn catalog_prevents_old_descriptor_split_during_bounded_eviction() {
    let cache = tempfile::tempdir().unwrap();
    drop(runtime(cache.path(), SHARED_PROCESS_SCRIPT));
    for identity in 0..3 {
        drop(runtime(
            cache.path(),
            crate::cstr!("process.stdout.write('seed-{identity}');").as_str(),
        ));
    }

    let result = cache.path().join("old-descriptor");
    let ready = result.with_extension("ready");
    let locked = result.with_extension("locked");
    let mut child = fixture_command(cache.path(), "old-fd", &result)
        .spawn()
        .unwrap();
    wait_for_path(&ready, &mut child);

    // The child has opened the old lease but intentionally has not locked it.
    // Constructors may proceed under the catalog's shared boundary, while
    // eviction must skip until that descriptor owns its runtime lease.
    for identity in 0..3 {
        drop(runtime(
            cache.path(),
            crate::cstr!("process.stdout.write('transition-{identity}');").as_str(),
        ));
    }
    fs::write(result.with_extension("advance"), b"advance").unwrap();
    wait_for_path(&locked, &mut child);

    drop(runtime(
        cache.path(),
        "process.stdout.write('while-old-lease-is-live');",
    ));
    let source = super::runtime_source(SHARED_PROCESS_SCRIPT);
    let hash = super::sha256_hex(&source);
    let root = super::cache_root(cache.path());
    assert!(root.join(crate::cstr!("{hash}.mjs").as_str()).exists());

    fs::write(result.with_extension("release"), b"release").unwrap();
    assert!(wait_for_child(&mut child).success());
    drop(runtime(
        cache.path(),
        "process.stdout.write('after-old-lease-release');",
    ));
    let entries: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        entries
            .iter()
            .filter(|path| path.extension().is_some_and(|extension| extension == "mjs"))
            .count(),
        super::CACHE_LIMIT
    );
    assert_eq!(
        entries
            .iter()
            .filter(|path| path
                .extension()
                .is_some_and(|extension| extension == "lease"))
            .count(),
        super::CACHE_LIMIT
    );
    assert_eq!(
        entries
            .iter()
            .filter(|path| path
                .file_name()
                .is_some_and(|name| name == super::CATALOG_LEASE))
            .count(),
        1
    );
}

#[test]
fn process_crash_releases_partial_write_for_recovery() {
    let cache = tempfile::tempdir().unwrap();
    drop(runtime(cache.path(), "process.stdout.write('prepare');"));
    let result = cache.path().join("crash-ready");
    let partial = super::cache_root(cache.path()).join("crashed.tmp");
    let mut child = fixture_command(cache.path(), "crash-partial", &result)
        .spawn()
        .unwrap();
    wait_for_path(&result, &mut child);

    drop(runtime(
        cache.path(),
        "process.stdout.write('while-partial-is-live');",
    ));
    assert!(partial.exists());
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());

    drop(runtime(
        cache.path(),
        "process.stdout.write('after-process-crash');",
    ));
    assert!(!partial.exists());
}

#[test]
fn damaged_owned_runtime_is_repaired_in_place() {
    let cache = tempfile::tempdir().unwrap();
    let script = "process.stdout.write('repair');";
    let cached = runtime(cache.path(), script);
    let path = cached.path().to_path_buf();
    drop(cached);
    fs::write(&path, "partial").unwrap();

    // A child spawned by a parallel test can hold the dropped lease until its
    // exec closes it. The repair then falls back to a private file, so retry.
    let mut repaired = runtime(cache.path(), script);
    for _ in 0..100 {
        if repaired.is_cached() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
        repaired = runtime(cache.path(), script);
    }
    assert!(repaired.is_cached());
    assert_eq!(repaired.path(), path);
    assert_eq!(fs::read(path).unwrap(), super::runtime_source(script));
}

#[test]
fn startup_recovers_only_unlocked_partial_writes() {
    let cache = tempfile::tempdir().unwrap();
    drop(runtime(cache.path(), "process.stdout.write('prepare');"));
    let root = super::cache_root(cache.path());
    let abandoned = root.join("abandoned.tmp");
    fs::write(&abandoned, "partial").unwrap();
    let active = root.join("active.tmp");
    let mut active_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&active)
        .unwrap();
    active_file.write_all(b"partial").unwrap();
    active_file.lock().unwrap();

    drop(runtime(cache.path(), "process.stdout.write('recover');"));
    assert!(!abandoned.exists());
    assert!(active.exists());
    active_file.unlock().unwrap();
    drop(active_file);
    drop(runtime(
        cache.path(),
        "process.stdout.write('recover again');",
    ));
    assert!(!active.exists());
}

#[cfg(unix)]
#[test]
fn symlink_runtime_leaf_is_rejected_without_mutating_its_target() {
    use std::os::unix::fs::symlink;

    let cache = tempfile::tempdir().unwrap();
    let cached = runtime(cache.path(), SYMLINK_PROCESS_SCRIPT);
    let path = cached.path().to_path_buf();
    drop(cached);
    fs::remove_file(&path).unwrap();
    let foreign = cache.path().join("foreign.mjs");
    fs::write(&foreign, "foreign").unwrap();
    symlink(&foreign, &path).unwrap();

    let result = cache.path().join("symlink-result");
    let mut child = fixture_command(cache.path(), "symlink", &result)
        .spawn()
        .unwrap();
    assert!(wait_for_child(&mut child).success());
    assert_eq!(fs::read_to_string(result).unwrap(), "temporary");
    assert_eq!(fs::read_to_string(&foreign).unwrap(), "foreign");
    assert!(
        fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn runtime_source_identity_preserves_authored_errors_across_retries() {
    let project = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let config = project.path().join("vite.config.mjs");
    let authored_source = "throw new Error('authored-specificreason 日本語');\n";
    fs::write(&config, authored_source).unwrap();
    let script = "const { pathToFileURL } = await import('node:url');\nawait import(pathToFileURL(process.argv[1]).href);\n";
    let source = super::runtime_source(script);
    let first = super::process_runtime(cache.path(), &source).unwrap();
    let second = super::process_runtime(cache.path(), &source).unwrap();
    assert_eq!(first.path(), second.path());
    let output = first.command(&config).output().unwrap();
    let repeated = second.command(&config).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout.as_slice(), b"");
    assert_eq!(repeated.status, output.status);
    assert_eq!(repeated.stdout, output.stdout);
    assert_eq!(repeated.stderr, output.stderr);

    // A direct Node execution independently owns the authored file/source/error
    // header. The wrapper may name its own frame, but must retain this header.
    let direct = std::process::Command::new("node")
        .arg(&config)
        .output()
        .unwrap();
    assert_eq!(direct.status.code(), Some(1));
    let direct_stderr = std::str::from_utf8(&direct.stderr).unwrap();
    let observed_stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        observed_stderr.lines().take(5).collect::<Vec<_>>(),
        direct_stderr.lines().take(5).collect::<Vec<_>>()
    );
    assert_eq!(
        observed_stderr.lines().nth(4),
        Some("Error: authored-specificreason 日本語")
    );

    fs::write(&config, "throw new Error('changed-specificreason');\n").unwrap();
    let changed_message = first.command(&config).output().unwrap();
    assert_eq!(changed_message.status.code(), Some(1));
    assert_ne!(changed_message.stderr, output.stderr);
    assert_eq!(
        std::str::from_utf8(&changed_message.stderr)
            .unwrap()
            .lines()
            .nth(4),
        Some("Error: changed-specificreason")
    );

    let other_config = project.path().join("another.config.mjs");
    fs::write(&other_config, authored_source).unwrap();
    let changed_path = first.command(&other_config).output().unwrap();
    assert_eq!(changed_path.status.code(), Some(1));
    assert_ne!(changed_path.stderr, output.stderr);
    drop(first);
    drop(second);
    super::process_cache()
        .lock()
        .unwrap()
        .retain(|entry| entry.home != cache.path());
}
