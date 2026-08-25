use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn check_vue2_dialect_allows_vuetify_style_prop_aliases() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("vue2-vuetify-prop-aliases");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--config",
            "vize.config.json",
            "--tsconfig",
            "tsconfig.json",
            "src/App.vue",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(
        json["errorCount"], 0,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    for unexpected in ["keyof Props", "isMini", "hideDetails", "chips", "width"] {
        assert!(
            !stdout.contains(unexpected),
            "Vue 2 dialect should not report Vuetify alias props through keyed Props fallback:\n{stdout}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_test_vite_stub(&project_root.join("node_modules")).unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{ "vue": { "version": "2.7" } }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
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
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
type Props = { mini?: boolean } & { dense?: boolean }
defineProps<Props>()
</script>

<template>
  <div>{{ isMini }} {{ hideDetails }} {{ chips }} {{ width }}</div>
</template>
"#,
    )
    .unwrap();
    project_root
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(path.into());
    }

    let root = workspace_root();
    [
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}
