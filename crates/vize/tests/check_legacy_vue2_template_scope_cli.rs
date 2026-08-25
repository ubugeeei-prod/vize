#![cfg(feature = "legacy")]

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[test]
fn legacy_vue2_v_for_template_branch_keeps_alias_narrowing_in_scope() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_project("legacy-vue2-template-branch-scope");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "src/Branch.vue",
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
        "template branch aliases should type-check\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    for unexpected in ["TS2304", "TS2339", "aimAtReport", "guidanceTests"] {
        assert!(
            !stdout.contains(unexpected),
            "template branch scope regressed with {unexpected}:\n{stdout}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn create_project(name: &str) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_test_vue2_stub(&project_root.join("node_modules")).unwrap();
    write_test_vite_stub(&project_root.join("node_modules")).unwrap();
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
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{ "typeChecker": { "legacyVue2": true } }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Branch.vue"),
        r#"<script setup lang="ts">
type GuidanceTest =
  | { testResult: 'Pass'; retestFlag: boolean; passOnly: string }
  | { testResult: 'Fail'; retestFlag?: false; reason: string }

type AimAtReport =
  | { kind: 'summary'; title: string }
  | { kind: 'history'; title: string; guidanceTests: GuidanceTest[] }

const aimAtReports: AimAtReport[] = []
</script>

<template>
  <template v-for="aimAtReport in aimAtReports">
    <template v-if="aimAtReport.kind === 'summary'">
      <span>{{ aimAtReport.title }}</span>
    </template>
    <template v-else>
      <template v-for="guidanceTest in aimAtReport.guidanceTests">
        <span v-if="guidanceTest.retestFlag" class="retest-label mr-2">再 {{ aimAtReport.title }}</span>
        <span v-if="guidanceTest.testResult === 'Pass'" class="test-result-label__pass">{{ guidanceTest.passOnly }}</span>
      </template>
    </template>
  </template>
</template>
"#,
    )
    .unwrap();
    project_root
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str())
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(PathBuf::from(path));
    }

    let workspace_root = workspace_root();
    [
        workspace_root.join("node_modules/.bin/tsgo"),
        workspace_root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn write_test_vue2_stub(target: &Path) -> std::io::Result<()> {
    let vue_types_dir = target.join("vue").join("types");
    std::fs::create_dir_all(&vue_types_dir)?;
    std::fs::write(
        target.join("vue").join("package.json"),
        r#"{ "name": "vue", "types": "types/index.d.ts" }"#,
    )?;
    std::fs::write(
        vue_types_dir.join("index.d.ts"),
        r#"export interface Vue {
  $attrs: Record<string, unknown>;
  $refs: Record<string, any>;
  $slots: Record<string, unknown>;
  $emit: (...args: any[]) => void;
}
export default { version: '2.7.16' };
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{ "name": "vite", "types": "client.d.ts" }"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}
