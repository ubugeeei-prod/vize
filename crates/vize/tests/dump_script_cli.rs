//! Complete source-built native Program output and authored-byte fidelity laws.
use std::{fs, path::Path, process::Command};
use vize_l0::{String, cstr};

fn expected_stdout(captured: &[u8], name: &str, path: &Path) -> String {
    let text = std::str::from_utf8(captured).unwrap();
    let captured_path = cstr!("capture/cases/{name}.input");
    let (before, after) = text.split_once(captured_path.as_str()).unwrap();
    assert_eq!(after.matches(captured_path.as_str()).count(), 0);
    cstr!("{before}{}{after}", path.display())
}

#[test]
fn explicit_script_languages_observe_complete_real_programs_without_rewriting() {
    for (name, language, input, stdout) in [
        (
            "js-module",
            "js",
            include_bytes!("fixtures/dump_script_cli/js-module.input").as_slice(),
            include_bytes!("fixtures/dump_script_cli/js-module.stdout").as_slice(),
        ),
        (
            "ts-module",
            "ts",
            include_bytes!("fixtures/dump_script_cli/ts-module.input").as_slice(),
            include_bytes!("fixtures/dump_script_cli/ts-module.stdout").as_slice(),
        ),
        (
            "jsx-module",
            "jsx",
            include_bytes!("fixtures/dump_script_cli/jsx-module.input").as_slice(),
            include_bytes!("fixtures/dump_script_cli/jsx-module.stdout").as_slice(),
        ),
        (
            "tsx-module",
            "tsx",
            include_bytes!("fixtures/dump_script_cli/tsx-module.input").as_slice(),
            include_bytes!("fixtures/dump_script_cli/tsx-module.stdout").as_slice(),
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("authored.source");
        fs::write(&path, input).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .args(["dump", "--level", "l1", "--script", language, "--roundtrip"])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stderr, b"");
        assert_eq!(
            output.stdout,
            expected_stdout(stdout, name, &path).as_bytes()
        );
        assert_eq!(fs::read(&path).unwrap(), input);
    }
}

#[test]
fn explicit_script_goal_and_recovery_keep_full_actual_diagnostics_and_source() {
    let input = include_bytes!("fixtures/dump_script_cli/tsx-script-hole.input");
    let stdout = include_bytes!("fixtures/dump_script_cli/tsx-script-hole.stdout");
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("malformed.tsx");
    fs::write(&path, input).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args([
            "dump",
            "--level",
            "l1",
            "--script",
            "tsx",
            "--script-goal",
            "script",
            "--roundtrip",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stderr, b"");
    assert_eq!(
        output.stdout,
        expected_stdout(stdout, "tsx-script-hole", &path).as_bytes()
    );
    assert_eq!(fs::read(&path).unwrap(), input);
}

#[test]
fn invalid_script_options_keep_complete_errors_and_read_no_authored_file() {
    let cases: &[(&[&str], &[u8])] = &[
        (
            &[
                "dump",
                "--level",
                "l2",
                "--script",
                "ts",
                "--roundtrip",
                "missing",
            ],
            include_bytes!("fixtures/dump_script_cli/wrong-level.stderr"),
        ),
        (
            &["dump", "--script", "ts"],
            include_bytes!("fixtures/dump_script_cli/missing-roundtrip.stderr"),
        ),
        (
            &[
                "dump",
                "--level",
                "l1",
                "--roundtrip",
                "missing",
                "--script-goal",
                "script",
            ],
            include_bytes!("fixtures/dump_script_cli/missing-script.stderr"),
        ),
    ];
    for (arguments, stderr) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .args(*arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr, *stderr);
    }
}
