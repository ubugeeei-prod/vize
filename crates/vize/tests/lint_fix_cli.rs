use std::process::Command;

#[test]
fn lint_fix_exits_non_zero_while_unsupported() {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["lint", "--fix", "--no-config", "does-not-exist.vue"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("`vize lint --fix` is not supported yet"));
}
