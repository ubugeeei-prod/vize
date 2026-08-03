#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn check_accepts_asi_separated_v_on_statements() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = create_project();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "--no-check-props",
            "--no-check-emits",
            "src",
            "--format",
            "json",
            "--show-virtual-ts",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!("failed to parse check output: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert!(
        output.status.success(),
        "ASI-separated v-on statements should type-check:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], 0, "{stdout}");
    assert!(
        !stdout.contains("TS1005") && !stderr.contains("TS1005"),
        "the generated handler must not contain a parse error:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let virtual_ts = json["files"]
        .as_array()
        .expect("check JSON should include files")
        .iter()
        .filter_map(|file| file["virtualTs"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        virtual_ts.contains("// @create handler"),
        "the CLI should emit the statements inside a handler scope:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("void (\n      emit('create')"),
        "the CLI must not parenthesize the statements as one expression:\n{virtual_ts}"
    );
}

fn create_project() -> tempfile::TempDir {
    let base = workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests");
    std::fs::create_dir_all(&base).unwrap();
    let project = tempfile::Builder::new()
        .prefix("check-multistatement-v-on-")
        .tempdir_in(base)
        .unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    link_workspace_node_modules(project.path()).unwrap();
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project.path().join("src/App.vue"),
        r#"<script setup lang="ts">
const emit = defineEmits<{
  create: []
  close: []
}>()
</script>

<template>
  <Child
    @create="
      emit('create')
      emit('close')
    "
  />
</template>
"#,
    )
    .unwrap();
    project
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let source = workspace_root().join("node_modules");
    if !source.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace node_modules missing",
        ));
    }
    symlink_dir(&source, &project_root.join("node_modules"))
}

#[cfg(unix)]
fn symlink_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn symlink_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}
