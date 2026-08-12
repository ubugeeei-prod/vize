use std::{path::Path, process::Command};

#[test]
fn tui_requires_an_interactive_terminal_before_analysis() {
    let directory = tempfile::tempdir().unwrap();
    let output = doctor(directory.path(), &["--tui"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        std::str::from_utf8(&output.stderr)
            .unwrap()
            .contains("--tui requires an interactive stdin and stdout")
    );
}

#[test]
fn tui_rejects_machine_report_formats_before_terminal_detection() {
    let directory = tempfile::tempdir().unwrap();
    let output = doctor(directory.path(), &["--tui", "--format", "json"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        std::str::from_utf8(&output.stderr)
            .unwrap()
            .contains("--tui cannot be combined with --format json or sarif")
    );
}

fn doctor(root: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .arg("doctor")
        .args(arguments)
        .current_dir(root)
        .output()
        .unwrap()
}
