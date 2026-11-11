#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-targets-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn output_details(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_reports_all_supported_extensions_when_patterns_match_no_files() {
    let project_root = temp_project_dir("no-matching-files");
    fs::create_dir_all(&project_root).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "src/**/*.nothing"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found"
        ),
        "{stderr}"
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_json_emits_an_empty_array_when_patterns_match_no_files() {
    let project_root = temp_project_dir("json-no-matching-files");
    fs::create_dir_all(&project_root).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "**/*.vue", "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2), "{}", output_details(&output));
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stdout, serde_json::json!([]));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found"
        ),
        "{stderr}"
    );

    let _ = fs::remove_dir_all(project_root);
}

fn ignored_file_project(test_name: &str) -> PathBuf {
    let project_root = temp_project_dir(test_name);
    for (path, content) in [
        (
            "vite.config.mjs",
            crate::support::vite_config::source(
                r#"{ "ignores": ["tests/**"], "linter": { "preset": "incremental" } }"#,
            ),
        ),
        (
            "src/App.vue",
            "<template>\n  <div>ok</div>\n</template>\n".into(),
        ),
        (
            "tests/Ignored.vue",
            "<template>\n  <div>ok</div>\n</template>\n".into(),
        ),
    ] {
        let file = project_root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, content).unwrap();
    }
    project_root
}

fn lint_in(project_root: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .args(["lint", "-c", "vite.config.mjs"])
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn lint_treats_explicitly_named_ignored_files_as_ignored() {
    let project_root = ignored_file_project("explicit-ignored-file");

    let output = lint_in(&project_root, &["--", "tests/Ignored.vue"]);
    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--no-warn-ignored"),
        "{}",
        output_details(&output)
    );

    let output = lint_in(
        &project_root,
        &[
            "--max-warnings",
            "0",
            "--",
            "src/App.vue",
            "tests/Ignored.vue",
        ],
    );
    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));

    let output = lint_in(
        &project_root,
        &[
            "--max-warnings",
            "0",
            "--no-warn-ignored",
            "--",
            "src/App.vue",
            "tests/Ignored.vue",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("ignored"),
        "{}",
        output_details(&output)
    );

    for missing in ["src/Missing.vue", "tests/**/*.vue"] {
        let output = lint_in(&project_root, &["--", missing]);
        assert_eq!(output.status.code(), Some(2), "{}", output_details(&output));
    }

    let _ = fs::remove_dir_all(project_root);
}
