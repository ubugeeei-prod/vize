//! `vize fmt` routes comment-bearing config JSON (tsconfig, .vscode, …) through
//! the JSONC formatter while keeping plain `.json` strict.

use std::{fs, path::Path, process::Command};

fn write_project_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn fmt_formats_tsconfig_json_as_jsonc_preserving_comments() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "tsconfig.json",
        "{\n// ts compiler options\n\"compilerOptions\":{\"strict\":true, // be strict\n\"target\":\"ES2022\",},\n}",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--write", "tsconfig.json"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "stderr: {stderr}");
    let contents = fs::read_to_string(project.path().join("tsconfig.json")).unwrap();
    assert_eq!(
        contents,
        "{\n  // ts compiler options\n  \"compilerOptions\": {\n    \"strict\": true, // be strict\n    \"target\": \"ES2022\"\n  }\n}\n",
    );
}

#[test]
fn fmt_keeps_plain_json_strict_rejecting_comments() {
    // A non-config `.json` file stays strict JSON: comments are a parse error,
    // so the file is reported as errored rather than silently reformatted. This
    // guards the routing boundary (only known config filenames get JSONC).
    let project = tempfile::tempdir().unwrap();
    let original = "{\n// not allowed here\n\"a\": 1\n}\n";
    write_project_file(project.path(), "data.json", original);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--check", "data.json"])
        .output()
        .unwrap();

    assert_ne!(
        output.status.code(),
        Some(0),
        "strict JSON comments must error"
    );
    let contents = fs::read_to_string(project.path().join("data.json")).unwrap();
    assert_eq!(contents, original, "errored file must be left untouched");
}
