//! Authored setup lines reach TypeScript verbatim: multi-line literals keep
//! their values, and a nested setup error keeps its authored position (#12481).

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const LITERALS: &str = r#"<script setup lang="ts">
const sql = `
SELECT *
FROM t`
const exact: "\nSELECT *\nFROM t" = sql
const marker = `
export default 1
`
const markerExact: "\nexport default 1\n" = marker
const continued = "a\
b"
const joined: "ab" = continued
</script>
<template><div>{{ exact }}{{ markerExact }}{{ joined }}</div></template>
"#;

const NESTED_ERROR: &str = r#"<script setup lang="ts">
function pick(flag: boolean) {
  if (flag) {
    const wrong: number = "s"
    return wrong
  }
  return 0
}
</script>
<template><div>{{ pick(true) }}</div></template>
"#;

#[test]
fn setup_literals_keep_values_and_nested_errors_keep_positions() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-setup-verbatim-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    std::fs::write(project.join("src/Literals.vue"), LITERALS).unwrap();
    std::fs::write(project.join("src/NestedError.vue"), NESTED_ERROR).unwrap();
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler",
                "types": [], "lib": ["ES2022", "DOM"]
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = check_project(&tsgo, project, "tsconfig.json");
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        "src/NestedError.vue(4,11): error TS2322: Type 'string' is not assignable to type 'number'.\n",
        "{}",
        output_text(&output)
    );
}
