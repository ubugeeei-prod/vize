//! Unsupported document formatting reports a complete error without writes.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{fs, path::Path, process::Command};

fn write_project_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn run_fmt(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(dir)
        .args(["fmt"].iter().chain(args).copied().collect::<Vec<_>>())
        .output()
        .unwrap()
}

#[test]
fn fmt_refuses_yaml_without_changing_line_endings_or_final_newline() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(project.path(), "config.yaml", "a: 1\r\nb: 2");

    assert_unsupported_unchanged(project.path(), "config.yaml", "a: 1\r\nb: 2");
}

#[test]
fn fmt_refuses_markdown_without_changing_authored_spaces_or_code() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "README.md",
        "# Title   \n\n```sh\necho hi   \n```\ntext \n",
    );

    assert_unsupported_unchanged(
        project.path(),
        "README.md",
        "# Title   \n\n```sh\necho hi   \n```\ntext \n",
    );
}

#[test]
fn fmt_check_refuses_even_normalized_yaml() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(project.path(), "ok.yaml", "name: vize\n");

    assert_unsupported_unchanged(project.path(), "ok.yaml", "name: vize\n");
}

fn assert_unsupported_unchanged(root: &Path, name: &str, original: &str) {
    for command in ["--check", "--write", "--write", "--write", "--check"] {
        let output = run_fmt(root, &[command, name]);
        let action = if command == "--check" {
            "Checked"
        } else {
            "Formatted"
        };
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert_eq!(output.stdout, b"");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!(
                "Found 1 file(s)\nError formatting {name}: YAML and Markdown formatting is not implemented; these formats are excluded from default discovery\n\n{action} 1 file(s)\n  1 file(s) had errors\n"
            )
        );
        assert_eq!(fs::read(root.join(name)).unwrap(), original.as_bytes());
    }
}

#[test]
fn fmt_refuses_alternate_document_extensions_and_markdown_hard_breaks() {
    let project = tempfile::tempdir().unwrap();
    for (name, original) in [
        ("config.yml", "a:   1\nlist:\n- a\n"),
        ("notes.markdown", "Text with two spaces  \nnext line\n"),
    ] {
        write_project_file(project.path(), name, original);
        assert_unsupported_unchanged(project.path(), name, original);
    }
}

#[test]
fn fmt_accepts_typescript_declaration_files() {
    let project = tempfile::tempdir().unwrap();
    let fixture = include_str!(
        "../../../tests/_fixtures/differential/formatter/standalone-declarations/input.d.ts"
    );
    for name in ["types.d.ts", "types.d.mts", "types.d.cts"] {
        write_project_file(project.path(), name, fixture);
        let output = run_fmt(project.path(), &["--no-config", "--write", name]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let formatted = fs::read_to_string(project.path().join(name)).unwrap();
        assert!(
            formatted.contains("export const foo:"),
            "{name}: {formatted}"
        );
        let output = run_fmt(project.path(), &["--no-config", "--check", name]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
