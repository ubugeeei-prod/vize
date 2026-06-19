use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn fmt_check_supports_plain_ts_inputs() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(project.path(), "vite.config.ts", "export default {foo:1}\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--no-config", "--check", "vite.config.ts"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"));
    assert!(stderr.contains("Would reformat: vite.config.ts"));
    assert!(stderr.contains("Checked 1 file(s)"));
    assert!(!stderr.contains("No .vue"));
}

#[test]
fn fmt_write_stabilizes_plain_ts_until_fixed_point() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "app/features/authSignup/use.ts",
        r#"import { z } from "zod"

const schema = z.object({
  confirmCode: z
    .string()
    .regex(/^\d{6}$/, {
      message: t("form.validation.exactDigits", {
        target: t("form.field.confirmCode.label"),
        digits: 6,
      }),
    }),
})
"#,
    );
    write_project_file(
        project.path(),
        "app/features/layout/position.ts",
        r#"function getSlotPosition(slotName: string): { style: { left: string, top: string, width: string, height: string }, inPortal: boolean } | null {
  return null
}
"#,
    );

    let write = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--no-config",
            "--write",
            "app/features/authSignup/use.ts",
            "app/features/layout/position.ts",
        ])
        .output()
        .unwrap();
    assert_eq!(write.status.code(), Some(0), "{}", output_details(&write));

    let check = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--no-config",
            "--check",
            "app/features/authSignup/use.ts",
            "app/features/layout/position.ts",
        ])
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(0), "{}", output_details(&check));

    let formatted =
        fs::read_to_string(project.path().join("app/features/authSignup/use.ts")).unwrap();
    assert!(
        formatted.contains("confirmCode: z.string().regex(/^\\d{6}$/, {"),
        "first write must produce the fixed point used by the second write"
    );
    let formatted =
        fs::read_to_string(project.path().join("app/features/layout/position.ts")).unwrap();
    assert!(
        formatted.starts_with("function getSlotPosition(slotName: string): {"),
        "plain TS files must use the same fixed point as SFC script blocks"
    );
}

#[test]
fn lint_supports_plain_ts_inputs() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "vite.config.ts",
        r#"import { getCurrentInstance } from "vue";

const instance = getCurrentInstance();
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--no-config",
            "--preset",
            "opinionated",
            "--format",
            "text",
            "--help-level",
            "none",
            "vite.config.ts",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("script/no-get-current-instance"));
    assert!(stdout.contains("Linted 1 files"));
}
