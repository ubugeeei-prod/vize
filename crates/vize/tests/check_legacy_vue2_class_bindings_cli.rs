#![cfg(feature = "legacy")]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn legacy_vue2_merges_static_and_dynamic_component_class_bindings() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("legacy-vue2-component-class-bindings");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
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
    for unexpected in ["TS1117", "multiple properties with the same name"] {
        assert!(
            !stdout.contains(unexpected) && !stderr.contains(unexpected),
            "static and dynamic class bindings must not produce duplicate object keys:\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_test_vue2_6_stub(&project_root.join("node_modules")).unwrap();
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
        project_root.join("vize.config.json"),
        r#"{
  "typeChecker": {
    "legacyVue2": true
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/TeacherCard.vue"),
        r#"<script lang="ts">
export default {
  props: {
    teacher: Object,
  },
}
</script>

<template>
  <article />
</template>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script lang="ts">
import TeacherCard from "./TeacherCard.vue"

export default {
  components: { TeacherCard },
  data() {
    return {
      teacher: { name: "Ada" },
      isLoading: false,
    }
  },
}
</script>

<template>
  <TeacherCard
    class="ma-1"
    :teacher="teacher"
    :class="{ 'loading-place-holder': isLoading }"
  />
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

fn write_test_vue2_6_stub(target: &Path) -> std::io::Result<()> {
    let vue_types_dir = target.join("vue").join("types");
    std::fs::create_dir_all(&vue_types_dir)?;
    std::fs::write(
        target.join("vue").join("package.json"),
        r#"{
  "name": "vue",
  "types": "types/index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_types_dir.join("index.d.ts"),
        r#"export interface Vue {
  $attrs: Record<string, unknown>;
  $refs: Record<string, any>;
  $slots: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}

declare const VueConstructor: {
  version: string;
};

export default VueConstructor;
"#,
    )?;
    Ok(())
}
