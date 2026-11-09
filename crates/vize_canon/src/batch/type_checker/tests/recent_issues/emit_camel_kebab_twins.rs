//! #11453: events declared as both `barBaz` and `bar-baz` must not merge two
//! handler types into one `onBarBaz` key.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const FOO: &str = r#"<script setup lang="ts">
interface Emits {
  (event: "barBaz", value: number): void;
  (event: "bar-baz", value: number): void;
  (event: string, ...args: any[]): void;
}
defineEmits<Emits>();
</script>

<template>
  <div />
</template>
"#;

const PROBE: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo @bar-baz="(value) => value.toFixed()" />
  <Foo @barBaz="(value) => value.toFixed()" />
</template>
"#;

const WRONG_HANDLER: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo @bar-baz="(value: string) => value.length" />
</template>
"#;

const WRONG_MEMBER: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo @bar-baz="(value) => value.nope()" />
</template>
"#;

#[test]
fn camel_and_kebab_event_twins_keep_one_handler_tuple() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = super::real_vue::create_project_case(
        "emit-camel-kebab-twins",
        &[
            ("src/Foo.vue", FOO),
            ("src/Probe.vue", PROBE),
            ("src/WrongHandler.vue", WRONG_HANDLER),
            ("src/WrongMember.vue", WRONG_MEMBER),
        ],
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
    "target": "ESNext"
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        snapshot,
        Some(vec![
            (
                "src/WrongHandler.vue".into(),
                Some(2322),
                "6:9:error Type '(value: string) => number' is not assignable to type '(value: number) => any'.\nTypes of parameters 'value' and 'value' are incompatible.\nType 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/WrongMember.vue".into(),
                Some(2339),
                "6:35:error Property 'nope' does not exist on type 'number'.".into(),
            ),
        ])
    );
}
