//! Minimal `vue` / `vite` package stubs for check CLI fixtures.

use std::path::Path;

pub(super) fn write_test_vue_stub(target: &Path) -> std::io::Result<()> {
    let vue_dir = target.join("vue");
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export * from "@vue/runtime-dom";
"#,
    )?;
    write_test_vue_runtime_dom_stub(target)?;
    Ok(())
}

pub(super) fn write_test_vue_runtime_dom_stub(target: &Path) -> std::io::Result<()> {
    let runtime_dom_dir = target.join("@vue").join("runtime-dom");
    std::fs::create_dir_all(&runtime_dom_dir)?;
    std::fs::write(
        runtime_dom_dir.join("package.json"),
        r#"{
  "name": "@vue/runtime-dom",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        runtime_dom_dir.join("index.d.ts"),
        include_str!("../support/vue-runtime-dom.d.ts"),
    )?;
    Ok(())
}

pub(super) fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
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
