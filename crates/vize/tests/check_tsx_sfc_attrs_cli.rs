use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_carton::cstr;

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
        .join(cstr!("check-tsx-sfc-attrs-{name}-{}", std::process::id()).as_str())
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

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let workspace_node_modules = workspace_root().join("node_modules");
    if !workspace_node_modules.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace node_modules missing",
        ));
    }
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(&workspace_node_modules.join("vue"), &target.join("vue"))?;
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

#[test]
fn check_tsx_story_allows_sfc_class_and_style_attrs() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("fallthrough");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "types": ["vue/jsx"],
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/AfButton.vue"),
        r#"<script setup lang="ts">
defineProps<{ color: 'primary' | 'secondary' }>()
defineEmits<{ click: [event: MouseEvent] }>()
</script>
<template><button /></template>
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/AfButton.stories.tsx"),
        r#"import AfButton from './AfButton.vue';

export const Example = () => (
  <AfButton
    class="af-mb-2"
    style="width: 200px"
    color="primary"
    onClick={(event: MouseEvent) => event.preventDefault()}
  />
);
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src/AfButton.stories.tsx",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    let helpers =
        std::fs::read_to_string(project_root.join("node_modules/.vize/canon/__vize_helpers.d.ts"))
            .unwrap();
    assert!(helpers.contains("interface IntrinsicAttributes"));
    assert!(helpers.contains("class?: unknown; style?: unknown"));

    let _ = std::fs::remove_dir_all(&project_root);
}
