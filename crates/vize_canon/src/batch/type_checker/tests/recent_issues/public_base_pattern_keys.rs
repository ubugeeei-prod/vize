//! #11445: a `$${string}` key on `ComponentCustomProperties` must not erase
//! the literal public instance members of a generated component.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const AUGMENT: &str = r#"import "vue";

declare module "vue" {
  interface ComponentCustomProperties {
    [key: `$${string}`]: unknown;
  }
}
"#;

const FOO: &str = r#"<template>
  <div />
</template>
"#;

const PROBE: &str = r#"import { defineComponent } from "vue";
import Foo from "./Foo.vue";

export const Bar = defineComponent({}) as typeof Foo;
declare const instance: InstanceType<typeof Foo>;
instance.$nextTick(() => {});
instance.$forceUpdate();
export const bad: string = instance.$forceUpdate;
"#;

fn check(
    name: &str,
    vue_options: &str,
    augment: &str,
) -> Option<Vec<(vize_carton::String, Option<u32>, vize_carton::String)>> {
    let root = super::real_vue::create_project_case(
        name,
        &[
            ("src/augment.d.ts", augment),
            ("src/Foo.vue", FOO),
            ("src/Probe.ts", PROBE),
        ],
    );
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
  "include": ["src/**/*.ts", "src/**/*.vue"]
}}"#
    );
    std::fs::write(root.join("tsconfig.json"), tsconfig.as_str()).unwrap();
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    snapshot
}

#[test]
fn pattern_custom_properties_keep_literal_public_instance_members() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    // The inferred `$el`/`$refs` instance omits members from the same base.
    for (name, vue_options) in [
        ("public-base-pattern-keys", ""),
        (
            "public-base-pattern-keys-inferred",
            r#"
  "vueCompilerOptions": { "inferComponentDollarEl": true, "inferComponentDollarRefs": true },"#,
        ),
    ] {
        assert_eq!(
            check(name, vue_options, AUGMENT),
            Some(vec![(
                "src/Probe.ts".into(),
                Some(2322),
                "8:14:error Type '() => void' is not assignable to type 'string'.".into(),
            )]),
            "{name}"
        );
    }
}

#[test]
fn large_custom_properties_do_not_explode_the_public_instance_union() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let members = (0..300)
        .map(|index| format!("    prop{index}: string;"))
        .collect::<Vec<_>>()
        .join("\n");
    let augment = format!(
        r#"import "vue";

declare module "vue" {{
  interface ComponentCustomProperties {{
{members}
  }}
}}
"#
    );

    assert_eq!(
        check("public-base-large-custom-properties", "", &augment),
        Some(vec![(
            "src/Probe.ts".into(),
            Some(2322),
            "8:14:error Type '() => void' is not assignable to type 'string'.".into(),
        )])
    );
}
