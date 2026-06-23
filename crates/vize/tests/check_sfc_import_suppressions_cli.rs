use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
        .join(format!(
            "check-sfc-import-suppressions-{name}-{}",
            std::process::id()
        ))
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

fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let Some(vue_package) = workspace_vue_package() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package missing",
        ));
    };
    let workspace_node_modules = vue_package.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package has no node_modules parent",
        )
    })?;
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(&vue_package, &target.join("vue"))?;
    let vue_namespace = workspace_node_modules.join("@vue");
    if vue_namespace.exists() {
        symlink_path(&vue_namespace, &target.join("@vue"))?;
    }
    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[test]
fn check_preserves_sfc_import_suppressions_and_wildcard_vue_shims() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("vue-shim");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["ts-shim.d.ts", "src/**/*"]
}"#,
    );
    write(
        &project_root,
        "ts-shim.d.ts",
        r#"declare module "*.vue" {
  const component: unknown;
  export default component;
}

declare module "emoji-mart-vue-fast/src/components";
declare module "emoji-mart-vue-fast/data/all.json";
"#,
    );
    write(
        &project_root,
        "node_modules/emoji-mart-vue-fast/src/components/Emoji.vue",
        r#"<script lang="ts">
export default {};
</script>

<template><span /></template>
"#,
    );
    write(
        &project_root,
        "src/SchoolExamScoreChart.vue",
        r#"<script lang="ts">
// FIXME: types
// @ts-ignore
import Chart from "chart.js/auto/auto";

export default {
  mounted() {
    void Chart;
  },
};
</script>

<template><canvas /></template>
"#,
    );
    write(
        &project_root,
        "src/TeacherSharedCommentReactionBtn.vue",
        r#"<script setup lang="ts">
import Emoji from "emoji-mart-vue-fast/src/components/Emoji.vue";

void Emoji;
</script>

<template><Emoji /></template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    for unexpected in [
        "chart.js/auto/auto",
        "Emoji.vue.ts",
        "Duplicate identifier",
        "TS2300",
        "TS2307",
        "TS7016",
    ] {
        assert!(
            !stdout.contains(unexpected) && !stderr.contains(unexpected),
            "diagnostics should not mention {unexpected}:\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
    let virtual_chart = std::fs::read_to_string(
        project_root.join("node_modules/.vize/canon/src/SchoolExamScoreChart.vue.ts"),
    )
    .unwrap();
    assert!(
        virtual_chart.contains("// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";"),
        "TS suppression must stay adjacent to the hoisted import:\n{virtual_chart}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
