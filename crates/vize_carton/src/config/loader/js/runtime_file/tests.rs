use super::NodeRuntimeFile;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

const SHARED_PROCESS_SCRIPT: &str = "process.stdout.write('shared process');";

fn runtime(cache_home: &Path, script: &str) -> NodeRuntimeFile {
    NodeRuntimeFile::new_with_cache_home(script, cache_home).unwrap()
}

#[test]
fn large_evaluator_is_a_private_file_instead_of_a_command_argument() {
    let cache = tempfile::tempdir().unwrap();
    let script = " ".repeat(70_000);
    assert!(script.encode_utf16().count() > 32_767);
    let runtime = runtime(cache.path(), &script);
    assert!(runtime.is_cached());
    let config = std::path::Path::new("vite.config.ts");
    let command = runtime.command(config);
    let args: Vec<_> = command.get_args().collect();
    assert_eq!(args.len(), 2);
    assert_eq!(args.first().copied(), Some(runtime.path().as_os_str()));
    assert_eq!(args.last().copied(), Some(config.as_os_str()));
    let argument_units: usize = args
        .iter()
        .map(|arg| arg.to_string_lossy().encode_utf16().count() + 3)
        .sum();
    assert!(argument_units < 4_096);
    let actual = fs::read(runtime.path()).unwrap();
    assert_eq!(
        &actual[..super::ARGV_COMPATIBILITY.len()],
        super::ARGV_COMPATIBILITY
    );
    let script_end = super::ARGV_COMPATIBILITY.len() + script.len();
    assert_eq!(
        &actual[super::ARGV_COMPATIBILITY.len()..script_end],
        script.as_bytes()
    );
    assert_eq!(
        &actual[script_end..],
        b"\n//# sourceURL=vize:config-runtime\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(runtime.path()).unwrap().permissions().mode() & 0o077,
            0
        );
        assert_eq!(
            fs::metadata(super::cache_root(cache.path()))
                .unwrap()
                .permissions()
                .mode()
                & 0o077,
            0
        );
    }
}

#[test]
fn file_execution_preserves_unicode_paths_cache_marker_and_cwd() {
    let root = tempfile::tempdir().unwrap();
    let runtime_cache = root.path().join("cache space 日本語");
    let project = root.path().join("space 日本語 🐱");
    fs::create_dir(&project).unwrap();
    let config = project.join("vite.config.ts");
    let cache = root.path().join("compile cache");
    fs::create_dir(&cache).unwrap();
    let mut script = " ".repeat(70_000);
    script.push_str(
        r#"
process.stdout.write(JSON.stringify({ argv: process.argv.slice(1), cwd: process.cwd() }));
"#,
    );
    for cached in [false, true] {
        let runtime = runtime(&runtime_cache, &script);
        let mut command = runtime.command(&config);
        if cached {
            command.arg(&cache);
        }
        command.arg("--observe-config-inputs");
        let output = command
            .output()
            .expect("Node is required for config runtime transport tests");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let expected = if cached {
            serde_json::json!([config, cache, "--observe-config-inputs"])
        } else {
            serde_json::json!([config, "--observe-config-inputs"])
        };
        assert_eq!(result["argv"], expected);
        let cwd = std::path::Path::new(result["cwd"].as_str().unwrap());
        assert_eq!(cwd.canonicalize().unwrap(), project.canonicalize().unwrap());
    }
}

#[test]
fn file_runtime_keeps_stdin_available_for_the_real_observation_handshake() {
    let project = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let config = project.path().join("vite.config.ts");
    let mut script = " ".repeat(70_000);
    script.push_str(
        r#"
process.stdout.write("B2\n[]");
const { value: ack, done } = await process.stdin[Symbol.asyncIterator]().next();
if (done || ack.length !== 1 || ack[0] !== 6) throw new Error("missing config acknowledgement");
process.stdout.write("R\n" + JSON.stringify(process.argv.slice(1)));
process.stdin.destroy();
"#,
    );
    let runtime = runtime(cache.path(), &script);
    let mut command = runtime.command(&config);
    let mut phases = 0;
    let output =
        super::super::super::js_observation::evaluate_observed(&mut command, &mut |paths| {
            assert!(paths.is_empty());
            phases += 1;
        })
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(phases, 1);
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([config, "--observe-config-inputs"])
    );
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
fn another_process_can_reuse_a_live_runtime() {
    let cache = tempfile::tempdir().unwrap();
    let result = cache.path().join("child-runtime-path");
    let held = runtime(cache.path(), SHARED_PROCESS_SCRIPT);
    let mut child = Command::new(env::current_exe().unwrap())
        .arg("runtime_cache_process_fixture")
        .arg("--ignored")
        .arg("--nocapture")
        .env("VIZE_RUNTIME_FIXTURE_CACHE", cache.path())
        .env("VIZE_RUNTIME_FIXTURE_RESULT", &result)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("another process blocked while reusing a live runtime");
        }
        thread::sleep(Duration::from_millis(10));
    };
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
    let runtime = runtime(&cache, SHARED_PROCESS_SCRIPT);
    fs::write(
        result,
        runtime.path().file_name().unwrap().to_str().unwrap(),
    )
    .unwrap();
}

#[test]
fn damaged_owned_runtime_is_repaired_in_place() {
    let cache = tempfile::tempdir().unwrap();
    let script = "process.stdout.write('repair');";
    let cached = runtime(cache.path(), script);
    let path = cached.path().to_path_buf();
    drop(cached);
    fs::write(&path, "partial").unwrap();

    let repaired = runtime(cache.path(), script);
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
    let cached = runtime(cache.path(), "process.stdout.write('owned');");
    let path = cached.path().to_path_buf();
    drop(cached);
    fs::remove_file(&path).unwrap();
    let foreign = cache.path().join("foreign.mjs");
    fs::write(&foreign, "foreign").unwrap();
    symlink(&foreign, &path).unwrap();

    let fallback = runtime(cache.path(), "process.stdout.write('owned');");
    assert!(!fallback.is_cached());
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
    let first = runtime(cache.path(), script);
    let second = runtime(cache.path(), script);
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
}
