use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-empty-patterns-{}-{test_name}-{nonce}",
        std::process::id()
    ))
}

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

fn warning_message(patterns: &[&str]) -> String {
    format!(
        "Warning: no .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found matching patterns: {:?}",
        patterns
    )
}

fn no_files_message(patterns: &[&str]) -> String {
    format!(
        "No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found matching patterns: {:?}",
        patterns
    )
}

fn text_summary(stdout: &str) -> (&str, &str) {
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 3, "{stdout}");
    assert_eq!(lines[0], "", "{stdout}");
    let Some(linted_count) = linted_file_count(lines[2]) else {
        panic!("unexpected linted line: {}", lines[2]);
    };
    (lines[1], linted_count)
}

fn linted_file_count(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("Linted ")?;
    let (count, _) = rest.split_once(" files in ")?;
    Some(count)
}

#[test]
fn lint_reports_empty_explicit_pattern_when_other_pattern_matches() {
    let project_root = temp_project_dir("mixed");
    write_project_file(&project_root, "src/b.ts", "export const x = 1;\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "src/**/*.ts", "src/**/*.{vue,ts}"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.as_ref(),
        format!("{}\n", warning_message(&["src/**/*.{vue,ts}"])),
        "{}",
        output_details(&output)
    );
    let (summary, linted_count) = text_summary(stdout.as_ref());
    assert_eq!(
        summary,
        "1 warning in 1 file",
        "{}",
        output_details(&output)
    );
    assert_eq!(linted_count, "1", "{}", output_details(&output));

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_counts_empty_explicit_patterns_for_max_warnings() {
    let project_root = temp_project_dir("max-warnings");
    write_project_file(&project_root, "src/b.ts", "export const x = 1;\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--no-config",
            "--max-warnings",
            "0",
            "src/**/*.ts",
            "src/**/*.{vue,ts}",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{}", output_details(&output));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.as_ref(),
        format!(
            "{}\n\nToo many warnings (1 > max 0)\n",
            warning_message(&["src/**/*.{vue,ts}"])
        ),
        "{}",
        output_details(&output)
    );
    let (summary, linted_count) = text_summary(stdout.as_ref());
    assert_eq!(
        summary,
        "1 warning in 1 file",
        "{}",
        output_details(&output)
    );
    assert_eq!(linted_count, "1", "{}", output_details(&output));

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_collects_root_level_relative_glob_matches() {
    let project_root = temp_project_dir("root-glob");
    write_project_file(&project_root, "foo.vue", "<template><div /></template>\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "foo*.vue", "missing*.vue"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.as_ref(),
        format!("{}\n", warning_message(&["missing*.vue"])),
        "{}",
        output_details(&output)
    );
    let (summary, linted_count) = text_summary(stdout.as_ref());
    assert_eq!(
        summary,
        "1 warning in 1 file",
        "{}",
        output_details(&output)
    );
    assert_eq!(linted_count, "1", "{}", output_details(&output));

    let _ = fs::remove_dir_all(project_root);
}

#[cfg(windows)]
#[test]
fn lint_collects_windows_absolute_glob_from_other_current_dir() {
    let project_root = temp_project_dir("windows-absolute-glob");
    let cwd = temp_project_dir("windows-absolute-cwd");
    fs::create_dir_all(&cwd).unwrap();
    write_project_file(&project_root, "foo.vue", "<template><div /></template>\n");
    let pattern = project_root.join("foo*.vue").display().to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&cwd)
        .args(["lint", "--no-config", pattern.as_str()])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.as_ref(), "", "{}", output_details(&output));
    let (summary, linted_count) = text_summary(stdout.as_ref());
    assert_eq!(
        summary,
        "No problems found in 1 file(s)",
        "{}",
        output_details(&output)
    );
    assert_eq!(linted_count, "1", "{}", output_details(&output));

    let _ = fs::remove_dir_all(project_root);
    let _ = fs::remove_dir_all(cwd);
}

#[test]
fn lint_preserves_single_empty_pattern_message() {
    let project_root = temp_project_dir("single");
    write_project_file(&project_root, "src/b.ts", "export const x = 1;\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "src/**/*.{vue,ts}"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stdout.as_ref(), "", "{}", output_details(&output));
    assert_eq!(
        stderr.as_ref(),
        format!("{}\n", no_files_message(&["src/**/*.{vue,ts}"])),
        "{}",
        output_details(&output)
    );

    let _ = fs::remove_dir_all(project_root);
}
