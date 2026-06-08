use std::process::Command;

#[test]
fn curator_env_outputs_bug_report_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["curator", "env"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    for prefix in [
        "Vize: ",
        "OS: ",
        "Architecture: ",
        "Node.js: ",
        "Package manager: ",
        "Rust: ",
    ] {
        assert!(stdout.contains(prefix), "missing {prefix} in:\n{stdout}");
    }
}
