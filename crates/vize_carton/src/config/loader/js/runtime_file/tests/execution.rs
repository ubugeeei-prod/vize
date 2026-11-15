use std::fs;

use super::runtime;

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
        &actual[..super::super::ARGV_COMPATIBILITY.len()],
        super::super::ARGV_COMPATIBILITY
    );
    let script_end = super::super::ARGV_COMPATIBILITY.len() + script.len();
    assert_eq!(
        &actual[super::super::ARGV_COMPATIBILITY.len()..script_end],
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
            fs::metadata(super::super::cache_root(cache.path()))
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
        crate::config::loader::js_observation::evaluate_observed(&mut command, &mut |paths| {
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
