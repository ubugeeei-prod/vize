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
) -> Option<Vec<(vize_carton::String, Option<u32>, vize_carton::String)>> {
    let root = super::real_vue::create_project_case(
        name,
        &[
            ("src/augment.d.ts", AUGMENT),
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
            check(name, vue_options),
            Some(vec![(
                "src/Probe.ts".into(),
                Some(2322),
                "8:14:error Type '() => void' is not assignable to type 'string'.".into(),
            )]),
            "{name}"
        );
    }
}
