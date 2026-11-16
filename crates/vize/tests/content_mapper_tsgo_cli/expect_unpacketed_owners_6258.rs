//! #6258: an expectation owns the listeners of a native element whose
//! sibling binds too, and the replayed checks of a component in a slot loop.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const FRAME: &str = "<template><div><slot name=\"menu\" /></div></template>\n";

const ITEM: &str = r#"<script setup lang="ts">
defineProps<{ name: string }>()
</script>
<template><p>{{ name }}</p></template>
"#;

const SCRIPT: &str = r#"<script setup lang="ts">
import { ref } from "vue"
import Frame from "./Frame.vue"
import Item from "./Item.vue"
const items = ref([])
const typed = ref([{ key: 1 }])
const memo = "x"
const title = "y"
const onUpdate = (value: string) => console.log(value)
</script>
"#;

/// `(file, template)`: each template's comment says what Vue reports.
const CASES: [(&str, &str); 10] = [
    // The reported shapes and their controls report nothing.
    (
        "VForInSlot",
        r#"  <Frame><template #menu>
    <!-- @vue-expect-error --><Item v-for="item in items" :key="item.key" name="a" />
  </template></Frame>"#,
    ),
    (
        "VForOutsideSlot",
        r#"  <div>
    <!-- @vue-expect-error --><Item v-for="item in items" :key="item.key" name="a" />
  </div>"#,
    ),
    (
        "InputBesideBoundSibling",
        r#"  <div>
    <!-- @vue-expect-error --><input :value="memo" @change="(event) => onUpdate(event?.target?.value)" />
    <p :title="title" />
  </div>"#,
    ),
    (
        "InputWithoutBinding",
        r#"  <div>
    <!-- @vue-expect-error --><input @change="(event) => onUpdate(event?.target?.value)" />
    <p :title="title" />
  </div>"#,
    ),
    (
        "InputBesideUnboundSibling",
        r#"  <div>
    <!-- @vue-expect-error --><input :value="memo" @change="(event) => onUpdate(event?.target?.value)" />
    <p />
  </div>"#,
    ),
    (
        "InputInLoop",
        r#"  <ul><li v-for="row in [memo]" :key="row">
    <!-- @vue-expect-error --><input :value="row" @change="(event) => onUpdate(event?.target?.value)" />
    <p :title="row" />
  </li></ul>"#,
    ),
    // Negative controls: a sibling's own error stays visible and an
    // expectation without an error is unused.
    (
        "SlotSiblingErrors",
        r#"  <Frame><template #menu>
    <!-- @vue-expect-error --><Item v-for="item in items" :key="item.key" name="a" />
    <Item :name="memo.nope" />
  </template></Frame>
  <p :title="memo.nope" />"#,
    ),
    (
        "SlotUnused",
        r#"  <Frame><template #menu>
    <!-- @vue-expect-error --><Item v-for="item in typed" :key="item.key" name="a" />
  </template></Frame>"#,
    ),
    (
        "InputSiblingError",
        r#"  <div>
    <!-- @vue-expect-error --><input :value="memo" @change="(event) => onUpdate(event?.target?.value)" />
    <p :title="memo.nope" />
  </div>"#,
    ),
    (
        "InputUnused",
        r#"  <div>
    <!-- @vue-expect-error --><input :value="memo" @change="(event) => onUpdate(String(event))" />
    <p :title="title" />
  </div>"#,
    ),
];

#[test]
fn expect_owns_unpacketed_listeners_and_replayed_checks() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-expect-unpacketed-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    std::fs::write(project.join("src/Frame.vue"), FRAME).unwrap();
    std::fs::write(project.join("src/Item.vue"), ITEM).unwrap();
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
            "src/InputSiblingError.vue(14,21): error TS2339: Property 'nope' does not exist on type '\"x\"'.",
            "src/InputUnused.vue(13,10): error vize4: Unused '@vue-expect-error' directive",
            "src/SlotSiblingErrors.vue(14,23): error TS2339: Property 'nope' does not exist on type '\"x\"'.",
            "src/SlotSiblingErrors.vue(16,19): error TS2339: Property 'nope' does not exist on type '\"x\"'.",
            "src/SlotUnused.vue(13,10): error vize4: Unused '@vue-expect-error' directive",
        ],
        "{}",
        output_text(&output)
    );
}
