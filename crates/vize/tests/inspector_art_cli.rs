use std::{fs, process::Command};

#[test]
fn inspector_compare_preserves_musea_define_art_script_setup() {
    if !dev_vue_compiler_available() {
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("CmsButton.art.vue"),
        r#"<script setup lang="ts">
defineArt("./CmsButton.vue", {
  title: "CmsButton",
  category: "Components",
  status: "draft",
  tags: ["button", "form"],
});
</script>

<art>
  <variant name="Primary" default>
    <CmsButton color="primary">Primary</CmsButton>
  </variant>
</art>
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["inspector", "src/CmsButton.art.vue", "--format", "compare"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let vize_code = json["files"][0]["vize"]["code"].as_str().unwrap();
    let official_code = json["files"][0]["official"]["code"].as_str().unwrap();
    assert!(official_code.contains("defineArt"), "{official_code}");
    assert!(vize_code.contains("defineArt"), "{vize_code}");
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
