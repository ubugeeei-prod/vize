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

fn write_entry_config(root: &Path) {
    write_project_file(
        root,
        "vize.config.json",
        r#"{
  "entries": [
    {
      "name": "design-system",
      "basePath": "design-system",
      "files": ["src/**/*.vue", "src/**/*.art.vue"]
    }
  ]
}"#,
    );
}

fn write_entry_ts_config(root: &Path) {
    write_project_file(
        root,
        "vize.config.ts",
        r#"export default {
  entries: [
    {
      name: "design-system",
      basePath: "design-system",
      files: ["src/**/*.vue", "src/**/*.art.vue"],
    },
  ],
}
"#,
    );
}

#[test]
fn fmt_check_supports_root_relative_paths_for_config_entries() {
    let project = tempfile::tempdir().unwrap();
    write_entry_config(project.path());
    write_project_file(
        project.path(),
        "design-system/src/AfsButton.vue",
        "<template><div>{{msg}}</div></template>\n<script setup lang=\"ts\">const msg=1</script>\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.json",
            "--check",
            "design-system/src/AfsButton.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"));
    assert!(stderr.contains("Would reformat: design-system/src/AfsButton.vue"));
    assert!(!stderr.contains("No .vue"));
}

#[test]
fn fmt_check_supports_root_relative_art_vue_paths_for_config_entries() {
    let project = tempfile::tempdir().unwrap();
    write_entry_ts_config(project.path());
    write_project_file(
        project.path(),
        "design-system/src/AfsButton.art.vue",
        "<template><div>{{msg}}</div></template>\n<script setup lang=\"ts\">const msg=1</script>\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.ts",
            "--check",
            "design-system/src/AfsButton.art.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"), "{stderr}");
    assert!(
        stderr.contains("Would reformat: design-system/src/AfsButton.art.vue"),
        "{stderr}"
    );
    assert!(!stderr.contains("No .vue"), "{stderr}");
}

#[test]
fn fmt_check_supports_root_relative_ts_paths_for_config_entries() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "vize.config.ts",
        r#"export default {
  entries: [
    { name: "app", basePath: ".", files: ["*.ts", "components/**/*.vue"] },
    { name: "scripts", basePath: "scripts", files: ["**/*.ts"] },
  ],
}
"#,
    );
    write_project_file(project.path(), "scripts/lint.ts", "export   const ok=1\n");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.ts",
            "--check",
            "scripts/lint.ts",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"), "{stderr}");
    assert!(
        stderr.contains("Would reformat: scripts/lint.ts"),
        "{stderr}"
    );
    assert!(!stderr.contains("No .vue"), "{stderr}");
}

#[test]
fn fmt_check_resolves_entry_relative_art_vue_paths_from_monorepo_root() {
    let project = tempfile::tempdir().unwrap();
    write_entry_ts_config(project.path());
    write_project_file(
        project.path(),
        "design-system/src/AfsButton.art.vue",
        "<template><div>{{msg}}</div></template>\n<script setup lang=\"ts\">const msg=1</script>\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.ts",
            "--check",
            "src/AfsButton.art.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"), "{stderr}");
    assert!(
        stderr.contains("Would reformat: design-system/src/AfsButton.art.vue"),
        "{stderr}"
    );
    assert!(!stderr.contains("No .vue"), "{stderr}");
}

#[test]
fn fmt_check_fails_when_explicit_patterns_match_no_files() {
    let project = tempfile::tempdir().unwrap();
    write_entry_config(project.path());

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "fmt",
            "--config",
            "vize.config.json",
            "--check",
            "design-system/src/**/*.art.vue",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No .vue, .js, .ts, .jsx, .tsx, or .json files found"));
}

#[test]
fn fmt_check_honors_top_level_ignores_during_directory_discovery() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "vize.config.json",
        r#"{
  "ignores": [
    "src/generated/schema.ts",
    "src/framework/static/sw.js"
  ]
}"#,
    );
    write_project_file(
        project.path(),
        "src/App.vue",
        "<template><div>{{msg}}</div></template>\n<script setup lang=\"ts\">const msg=1</script>\n",
    );
    write_project_file(
        project.path(),
        "src/generated/schema.ts",
        "export   const schema={value:1}\n",
    );
    write_project_file(
        project.path(),
        "src/framework/static/sw.js",
        "self.addEventListener('install',()=>{})\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--config", "vize.config.json", "--check", "src"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Found 1 file(s)"), "{stderr}");
    assert!(stderr.contains("Would reformat: src/App.vue"), "{stderr}");
    assert!(!stderr.contains("src/generated/schema.ts"), "{stderr}");
    assert!(!stderr.contains("src/framework/static/sw.js"), "{stderr}");
}

#[test]
fn fmt_write_pretty_prints_json_inputs() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "package.json",
        r#"{"name":"acme","version":"0.0.1","keywords":["vue","cli"]}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--write", "package.json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    let contents = fs::read_to_string(project.path().join("package.json")).unwrap();
    assert_eq!(
        contents,
        "{\n  \"name\": \"acme\",\n  \"version\": \"0.0.1\",\n  \"keywords\": [\n    \"vue\",\n    \"cli\"\n  ]\n}\n",
    );
}

#[test]
fn fmt_check_reports_already_formatted_json_as_unchanged() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "package.json",
        "{\n  \"name\": \"acme\",\n  \"version\": \"0.0.1\"\n}\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--check", "package.json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 file(s) already formatted"),
        "expected already-formatted summary, got: {stderr}",
    );
}

#[test]
fn fmt_write_preserves_comments_in_jsonc_inputs() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "biome.jsonc",
        "{\n// compiler\n\"compilerOptions\":{\"strict\":true, // be strict\n\"target\":\"ES2022\",},\n}",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--write", "biome.jsonc"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    let contents = fs::read_to_string(project.path().join("biome.jsonc")).unwrap();
    assert_eq!(
        contents,
        "{\n  // compiler\n  \"compilerOptions\": {\n    \"strict\": true, // be strict\n    \"target\": \"ES2022\"\n  }\n}\n",
    );
}

#[test]
fn fmt_check_reports_already_formatted_jsonc_as_unchanged() {
    let project = tempfile::tempdir().unwrap();
    write_project_file(
        project.path(),
        "settings.jsonc",
        "{\n  // editor settings\n  \"tabSize\": 2\n}\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["fmt", "--check", "settings.jsonc"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 file(s) already formatted"),
        "expected already-formatted summary, got: {stderr}",
    );
}
