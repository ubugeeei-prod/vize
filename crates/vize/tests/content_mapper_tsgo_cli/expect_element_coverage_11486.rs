//! #11486: one `@vue-expect-error` covers every check its element owns, and
//! none of a sibling's. TypeScript holds one interval per expectation.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ items: number[] }>()
defineEmits<{ select: [item: number] }>()
</script>
<template><div /></template>
"#;

const SCRIPT: &str = r#"<script setup lang="ts">
import Child from "./Child.vue"
declare function takesNumber(n: number): void
declare function takesString(s: string): void
const s = "x"
</script>
"#;

/// `(file, template)`: each template's comment says what Vue reports.
const CASES: [(&str, &str); 6] = [
    // A later sibling repeats the listener: the expect still owns it.
    (
        "Before",
        r#"  <!-- @vue-expect-error -->
  <Child :items="[1]" @select="(n) => takesString(n)" />
  <Child :items="[1]" @select="(n) => takesNumber(n)" />"#,
    ),
    (
        "After",
        r#"  <Child :items="[1]" @select="(n) => takesNumber(n)" />
  <!-- @vue-expect-error -->
  <Child :items="[1]" @select="(n) => takesString(n)" />"#,
    ),
    (
        "Both",
        r#"  <!-- @vue-expect-error -->
  <Child :items="[1]" @select="(n) => takesString(n)" />
  <!-- @vue-expect-error -->
  <Child :items="['x']" @select="(n) => takesNumber(n)" />"#,
    ),
    // Negative controls: the sibling's own error stays visible and the
    // expectation without an error is unused.
    (
        "Sibling",
        r#"  <Child :items="['x']" @select="(n) => takesNumber(n)" />
  <!-- @vue-expect-error -->
  <Child :items="[1]" @select="(n) => takesNumber(n)" />"#,
    ),
    (
        "Statement",
        r#"  <!-- @vue-expect-error -->
  <Child :items="[1]" @select="takesString($event)" />
  <Child :items="[1]" @select="takesNumber($event)" />"#,
    ),
    (
        "Native",
        r#"  <!-- @vue-expect-error -->
  <div :title="s.nope" @click="takesNumber(1)" />
  <div :title="s.nope" />"#,
    ),
];

#[test]
fn expect_covers_its_element_and_no_sibling() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-expect-coverage-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    std::fs::write(project.join("src/Child.vue"), CHILD).unwrap();
    for (name, template) in CASES {
        std::fs::write(
            project.join(cstr!("src/{name}.vue").as_str()),
            cstr!("{SCRIPT}<template>\n{template}\n</template>\n").as_str(),
        )
        .unwrap();
    }
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
    let mut lines = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "src/Native.vue(10,18): error TS2339: Property 'nope' does not exist on type '\"x\"'.",
            "src/Sibling.vue(8,19): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Sibling.vue(9,8): error vize4: Unused '@vue-expect-error' directive",
        ],
        "{}",
        output_text(&output)
    );
}
