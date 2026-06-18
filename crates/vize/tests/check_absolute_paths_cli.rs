use std::{path::PathBuf, process::Command};

fn unique_case_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vize-{name}-{}", std::process::id()))
}

#[test]
fn check_rejects_absolute_input_outside_project_root_with_tsconfig() {
    let case_root = unique_case_dir("check-absolute-outside-root");
    let _ = std::fs::remove_dir_all(&case_root);
    let project_root = case_root.join("project");
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(project_root.join("tsconfig.json"), "{}").unwrap();

    let outside = case_root.join("Outside.vue");
    std::fs::write(&outside, "<template />\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["check", "--tsconfig", "tsconfig.json"])
        .arg(&outside)
        .output()
        .unwrap();

    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        !output.status.success(),
        "check unexpectedly passed: {stderr}"
    );
    assert!(
        stderr.contains("outside project root"),
        "missing outside-root error in: {stderr}"
    );
    assert!(
        !stderr.contains("Building Corsa virtual project"),
        "outside-root input reached Corsa setup: {stderr}"
    );

    let _ = std::fs::remove_dir_all(&case_root);
}
