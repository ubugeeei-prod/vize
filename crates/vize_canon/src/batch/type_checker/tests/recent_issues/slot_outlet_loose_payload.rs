//! Upstream vue-language-tools #4979: a `<slot>` payload's excess key stays an
//! error under the loose unknown-props policy, which only forgives undeclared
//! component attributes.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

const CHILD: &str = r#"<script setup lang="ts">
defineSlots<{
  default: (props: { foo: string }) => any;
}>();
</script>

<template>
  <slot bar="..."></slot>
  <slot foo="ok"></slot>
</template>
"#;

const PARENT: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>

<template>
  <Child :buffer="200" />
</template>
"#;

#[test]
fn slot_outlet_excess_keys_survive_the_loose_component_key_filter() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "slot-outlet-loose-payload-4979",
        &[("src/Child.vue", CHILD), ("src/Parent.vue", PARENT)],
    );
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "types": []
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    let diagnostics = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(root);
    // The whole packet: the slot payload's undeclared `bar` is reported at its
    // key, while the parent's undeclared component attribute stays forgiven.
    assert_eq!(
        diagnostics.expect("the native checker should be available"),
        vec![(
            "src/Child.vue".into(),
            Some(2353),
            "8:9:error Object literal may only specify known properties, and '\"bar\"' does not exist in type '{ foo: string; }'.".into(),
        )],
    );
}
