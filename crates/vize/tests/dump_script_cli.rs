//! Source-built explicit native Program inspection and byte fidelity laws.
#![expect(
    clippy::disallowed_macros,
    reason = "CLI expected bytes use std format"
)]

use std::{fs, process::Command};

#[test]
fn explicit_script_languages_observe_real_programs_without_rewriting() {
    for (language, input, kind) in [
        (
            "js",
            "/* α */ import { ref } from 'vue';\r\nexport const value = ref(0);",
            "ImportDeclaration",
        ),
        (
            "ts",
            "interface Props { value: number } export const value: Props = { value: 1 };",
            "TSInterfaceDeclaration",
        ),
        (
            "jsx",
            "const View = () => <section>{values.map(value => <span>{value}</span>)}</section>;",
            "VariableDeclaration",
        ),
        (
            "tsx",
            "type Props = { value: number }; export const View = ({ value }: Props) => <span>{value}</span>;",
            "TSTypeAliasDeclaration",
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
        assert!(output.stderr.is_empty());
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        assert!(stdout.starts_with(&format!("script: {language} module; hole=None\n")));
        assert!(stdout.contains(&format!("statement {kind} @Span {{")));
        assert!(stdout.ends_with(&format!(
            "dump: l1 roundtrip OK: {} ({} bytes)\n",
            path.display(),
            input.len()
        )));
        assert_eq!(fs::read(&path).unwrap(), input.as_bytes());
    }
}

#[test]
fn explicit_script_goal_and_recovery_preserve_source_and_report_actual_diagnostics() {
    let input = "/* α */ const value: number = ;\r\n// β";
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
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(stdout.starts_with("script: tsx script; hole=Some(Syntax)\n"));
    assert!(stdout.contains("comment SingleLineBlock @Span { start: 0, end: 8 }"));
    assert!(stdout.contains("diagnostic Error: Unexpected token\n"));
    assert!(stdout.contains("label @Span { start: 31, end: 32 }"));
    assert!(stdout.contains("dump: l1 roundtrip OK:"));
    assert_eq!(fs::read(&path).unwrap(), input.as_bytes());
}

#[test]
fn script_inspection_requires_l1_before_reading_or_mutating_any_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args([
            "dump",
            "--level",
            "l2",
            "--script",
            "ts",
            "--roundtrip",
            "missing",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"dump: --script requires --level l1\n");
    for arguments in [
        vec!["dump", "--script", "ts"],
        vec![
            "dump",
            "--level",
            "l1",
            "--roundtrip",
            "missing",
            "--script-goal",
            "script",
        ],
    ] {
        let rejected = Command::new(env!("CARGO_BIN_EXE_vize"))
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(rejected.status.code(), Some(2), "{rejected:?}");
        assert!(rejected.stdout.is_empty());
    }
}
