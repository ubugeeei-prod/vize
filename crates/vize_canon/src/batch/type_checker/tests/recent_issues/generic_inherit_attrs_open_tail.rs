//! #11446: a generic child with `inheritAttrs: false` and no `$attrs` target
//! accepts undeclared attributes unless unknown props are checked.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const FOO: &str = r#"<script setup lang="ts" generic="T extends string">
defineOptions({ inheritAttrs: false });
defineProps<{ bar: T }>();
</script>

<template>
  <div />
</template>
"#;

const PROBE: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo bar="a" p0 />
</template>
"#;

const WRONG_VALUE: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo :bar="1" p0 />
</template>
"#;

type Diagnostics = Vec<(vize_carton::String, Option<u32>, vize_carton::String)>;

fn check(name: &str, vue_options: &str, probes: &[(&str, &str)]) -> Diagnostics {
    let mut files = vec![("src/Foo.vue", FOO)];
    files.extend_from_slice(probes);
    let root = super::real_vue::create_project_case(name, &files);
    let tsconfig = vize_carton::cstr!(
        r#"{{
  "compilerOptions": {{
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext"
  }},{vue_options}
  "include": ["src/**/*.vue"]
}}"#
    );
    std::fs::write(root.join("tsconfig.json"), tsconfig.as_str()).unwrap();
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    snapshot.unwrap()
}

#[test]
fn generic_child_without_attrs_target_follows_check_unknown_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let wrong_value = (
        "src/WrongValue.vue".into(),
        Some(2322),
        "6:9:error Type 'number' is not assignable to type 'string'.".into(),
    );
    assert_eq!(
        check(
            "generic-inherit-attrs-default",
            "",
            &[
                ("src/Probe.vue", PROBE),
                ("src/WrongValue.vue", WRONG_VALUE)
            ],
        ),
        vec![wrong_value],
    );
    assert_eq!(
        check(
            "generic-inherit-attrs-unchecked",
            r#"
  "vueCompilerOptions": { "checkUnknownProps": false },"#,
            &[("src/Probe.vue", PROBE)],
        ),
        Vec::new(),
    );
    assert_eq!(
        check(
            "generic-inherit-attrs-strict",
            r#"
  "vueCompilerOptions": { "strictTemplates": true },"#,
            &[("src/Probe.vue", PROBE)],
        ),
        vec![(
            "src/Probe.vue".into(),
            Some(2353),
            r#"6:16:error Object literal may only specify known properties, and '"p0"' does not exist in type '$SetupProps & VNodeProps & AllowedComponentProps & ComponentCustomProps & { abbr?: unknown; about?: unknown; "accent-height"?: unknown; ... 679 more ...; zoomAndPan?: unknown; } & { ...; }'."#
                .into(),
        )],
    );
}
