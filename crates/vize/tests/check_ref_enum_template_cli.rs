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
        .join(cstr!("check-ref-enum-template-{name}-{}", std::process::id()).as_str())
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
    let target = project_root.join("node_modules");
    if target.exists() || target.is_symlink() {
        return Ok(());
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
fn check_template_ref_enum_comparisons_keep_declared_ref_value_type() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("modal-state");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_node_modules(&project_root).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r##"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"##,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/imports.d.ts"),
        r#"declare module '#imports' {
  export { ref, shallowRef } from 'vue';
}
"#,
    )
    .unwrap();
    std::fs::write(project_root.join("src/DirectModal.vue"), modal_sfc("vue")).unwrap();
    std::fs::write(
        project_root.join("src/NuxtImportsModal.vue"),
        modal_sfc("#imports"),
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
            "--show-virtual-ts",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "check failed:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        !stdout.contains("TS2367") && !stdout.contains("no overlap"),
        "enum ref comparisons should not narrow to the initial member:\n{stdout}"
    );

    let virtual_ts = json["files"]
        .as_array()
        .expect("check JSON should include files")
        .iter()
        .filter_map(|file| file["virtualTs"].as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        virtual_ts.contains("type __VizeWidenTemplateRef<T>"),
        "template refs should use the widening helper:\n{virtual_ts}"
    );
    for name in ["modalWindowState", "shallowModalWindowState"] {
        assert!(
            virtual_ts.contains(&format!("var {name}: __U<__R_{name}> = undefined as any;")),
            "template should shadow `{name}` with an unwrapped widened alias:\n{virtual_ts}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn modal_sfc(source: &str) -> String {
    cstr!(
        r#"<script setup lang="ts">
import {{ ref, shallowRef }} from '{source}'

enum ModalWindowState {{
  ProfileEdit = 'ProfileEdit',
  PasswordChange = 'PasswordChange',
  None = 'None',
}}

const modalWindowState = ref<ModalWindowState>(ModalWindowState.None)
const shallowModalWindowState = shallowRef<ModalWindowState>(ModalWindowState.None)
</script>

<template>
  <ModalWindow :is-opened="modalWindowState === ModalWindowState.ProfileEdit" />
  <ModalWindow :is-opened="modalWindowState === ModalWindowState.PasswordChange" />
  <ModalWindow :is-opened="shallowModalWindowState === ModalWindowState.ProfileEdit" />
  <ModalWindow :is-opened="shallowModalWindowState === ModalWindowState.PasswordChange" />
</template>
"#
    )
    .into()
}
