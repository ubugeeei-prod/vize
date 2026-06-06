use std::{fs, process::Command};

#[test]
fn inspector_json_supports_single_file_payloads() {
    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("App.vue"),
        r#"<script setup lang="ts">
const msg: string = "hello";
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "inspector",
            "src/App.vue",
            "--format",
            "json",
            "--target",
            "ssr",
            "--template-syntax",
            "quirks",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let snapshot = serde_json::json!({
        "version": json["version"],
        "target": json["target"],
        "selectedFile": json["selectedFile"],
        "options": json["options"],
        "files": json["files"].as_array().unwrap().iter().map(|file| {
            serde_json::json!({
                "path": file["path"],
                "source": file["source"],
            })
        }).collect::<Vec<_>>(),
    });

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "inspector_json_supports_single_file_payloads",
            serde_json::to_string_pretty(&snapshot).unwrap()
        );
    });
}

#[test]
fn inspector_url_supports_batch_payloads() {
    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("App.vue"), "<template><div>one</div></template>\n").unwrap();
    fs::write(
        src.join("Nested.vue"),
        "<template><span>two</span></template>\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "inspector",
            "src",
            "--format",
            "url",
            "--playground-url",
            "https://example.test/play/",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.starts_with("https://example.test/play/?tab=inspector#inspector="),
        "{stdout}"
    );
    assert!(stdout.contains("App.vue"), "{stdout}");
    assert!(stdout.contains("Nested.vue"), "{stdout}");
}

#[test]
fn inspector_default_glob_respects_gitignore() {
    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    let ignored = project.path().join("ignored");
    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&ignored).unwrap();
    fs::write(project.path().join(".gitignore"), "ignored/\n").unwrap();
    fs::write(
        src.join("App.vue"),
        "<template><div>included</div></template>\n",
    )
    .unwrap();
    fs::write(
        ignored.join("Ignored.vue"),
        "<template><div>ignored</div></template>\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["inspector", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let files = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(files, ["src/App.vue"]);
}

#[test]
fn inspector_agent_outputs_report_with_graph() {
    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("App.vue"),
        r#"<script setup lang="ts">
import Child from './Child'
</script>

<template>
  <Child />
</template>
"#,
    )
    .unwrap();
    fs::write(
        src.join("Child.vue"),
        r#"<template>
  <span>child</span>
</template>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "inspector",
            "src",
            "--format",
            "agent",
            "--playground-url",
            "https://example.test/play/",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    insta::with_settings!({
        snapshot_path => "snapshots"
    }, {
        insta::assert_snapshot!(
            "inspector_agent_outputs_report_with_graph",
            serde_json::to_string_pretty(&json).unwrap()
        );
    });
}

#[test]
fn inspector_compare_reports_missing_node_runtime() {
    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("App.vue"),
        "<template><div>hello</div></template>\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env(
            "VIZE_INSPECTOR_NODE",
            "vize-inspector-node-command-that-does-not-exist",
        )
        .args(["inspector", "src/App.vue", "--format", "compare"])
        .output()
        .unwrap();

    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success(), "{stderr}");
    assert!(
        stderr.contains("--format compare requires a local Node.js runtime"),
        "{stderr}"
    );
}

#[test]
fn inspector_compare_outputs_cli_diff_report_when_dev_runtime_is_available() {
    if !dev_vue_compiler_available() {
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("App.vue"),
        r#"<script setup lang="ts">
const msg: string = "hello";
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["inspector", "src/App.vue", "--format", "compare"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["schema"], "vize.inspector.compare");
    assert_eq!(json["target"], "dom");
    assert_eq!(json["summary"]["fileCount"], 1);
    assert_eq!(json["files"][0]["path"], "src/App.vue");
    assert!(json["files"][0]["official"]["code"].as_str().is_some());
    assert!(json["files"][0]["vize"]["code"].as_str().is_some());
    assert!(json["files"][0]["diff"].as_array().is_some());
}

fn dev_vue_compiler_available() -> bool {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent().and_then(|path| path.parent()) else {
        return false;
    };
    Command::new("node")
        .current_dir(workspace_root)
        .args(["--input-type=module", "-e", "import('vue/compiler-sfc')"])
        .status()
        .is_ok_and(|status| status.success())
}
