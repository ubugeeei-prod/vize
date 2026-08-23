use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const TRES_SCENE: &str = r#"<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
const visible = true
</script>

<template>
  <TresCanvas>
    <TresMesh v-if="visible">
      <TresSpotLight />
    </TresMesh>
  </TresCanvas>
</template>
"#;

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-build-custom-elements-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn needle_count(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn build_js(project_root: &Path, extra_args: &[&str]) -> String {
    write_project_file(project_root, "src/Scene.vue", TRES_SCENE);
    let mut args = vec![
        "build",
        "--format",
        "js",
        "src/Scene.vue",
        "--output",
        "dist",
    ];
    args.extend_from_slice(extra_args);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .args(&args)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));
    fs::read_to_string(project_root.join("dist/Scene.js")).unwrap()
}

#[test]
fn custom_renderer_alone_still_resolves_pascal_case_renderer_tags() {
    let project_root = temp_project_dir("custom-renderer-only");
    let js = build_js(&project_root, &["--no-config", "--custom-renderer"]);
    assert_eq!(
        needle_count(&js, r#"_resolveComponent("TresMesh")"#),
        1,
        "{js}"
    );
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn custom_elements_flag_compiles_matched_pascal_case_tags_as_elements() {
    let project_root = temp_project_dir("custom-elements-flag");
    let js = build_js(
        &project_root,
        &[
            "--no-config",
            "--custom-renderer",
            "--custom-elements",
            "Tres*",
        ],
    );
    assert_eq!(
        needle_count(&js, r#"_resolveComponent("TresMesh")"#),
        0,
        "{js}"
    );
    assert_eq!(
        needle_count(&js, r#"_resolveComponent("TresSpotLight")"#),
        0,
        "{js}"
    );
    assert_eq!(
        needle_count(&js, r#"_createElementBlock("TresMesh""#),
        1,
        "{js}"
    );
    assert_eq!(
        needle_count(&js, r#"_createElementVNode("TresSpotLight""#),
        1,
        "{js}"
    );
    assert_eq!(needle_count(&js, "import { TresCanvas }"), 1, "{js}");
    assert_eq!(
        needle_count(&js, r#"_createBlock(_unref(TresCanvas)"#),
        1,
        "{js}"
    );
    assert_eq!(
        needle_count(&js, r#"_createElementBlock("TresCanvas""#),
        0,
        "{js}"
    );
    assert_eq!(
        needle_count(&js, r#"_createElementVNode("TresCanvas""#),
        0,
        "{js}"
    );
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn custom_elements_config_compiles_matched_pascal_case_tags_as_elements() {
    let project_root = temp_project_dir("custom-elements-config");
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{
  "compiler": {
    "customRenderer": true,
    "customElements": ["Tres*"]
  }
}
"#,
    );
    let js = build_js(&project_root, &[]);
    assert_eq!(
        needle_count(&js, r#"_resolveComponent("TresMesh")"#),
        0,
        "{js}"
    );
    assert_eq!(
        needle_count(&js, r#"_createElementBlock("TresMesh""#),
        1,
        "{js}"
    );
    let _ = fs::remove_dir_all(project_root);
}
