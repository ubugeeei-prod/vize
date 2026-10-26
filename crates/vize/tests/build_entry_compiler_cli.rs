#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
//! #10781: `vue.entries[].compiler` applies to the matching cwd-relative
//! inputs of `vize build`, exactly like the same setting at the root.

use std::{fs, path::Path, process::Command};

const COMPONENT: &str = r#"<script setup lang="ts">
const label = "x";
</script>

<template>
  <button type="button">
    {{ label }}
  </button>
</template>
"#;

fn build(root: &Path, vue: &str) -> (Vec<u8>, Vec<u8>) {
    fs::write(
        root.join("vite.config.mjs"),
        format!("export default {{ vue: {vue} }};\n"),
    )
    .unwrap();
    let _ = fs::remove_dir_all(root.join("out"));
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["build", "src", "-o", "out"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    (
        fs::read(root.join("out/ds/a.js")).unwrap(),
        fs::read(root.join("out/app/b.js")).unwrap(),
    )
}

#[test]
fn entry_compiler_whitespace_reaches_cli_builds() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::write(root.join("package.json"), r#"{ "type": "module" }"#).unwrap();
    for file in ["src/ds/a.vue", "src/app/b.vue"] {
        let path = root.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, COMPONENT).unwrap();
    }

    let (scoped_ds, scoped_app) = build(
        root,
        r#"{ compiler: { whitespace: "vue2-preserve" }, entries: [{ files: ["src/ds/**/*.vue"], compiler: { whitespace: "condense" } }] }"#,
    );
    let (condensed_ds, _) = build(root, r#"{ compiler: { whitespace: "condense" } }"#);
    let (preserved_ds, preserved_app) =
        build(root, r#"{ compiler: { whitespace: "vue2-preserve" } }"#);
    assert_ne!(condensed_ds, preserved_ds, "the two modes must differ");
    assert_eq!(scoped_ds, condensed_ds, "entry scope");
    assert_eq!(scoped_app, preserved_app, "root setting outside the entry");
}
