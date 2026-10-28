//! A kebab-case `aria-*` binding still satisfies its declared camelCase prop.
//!
//! `aria-*` also remains an HTML attribute for optional-value compatibility:
//! the whole-props check must not reject `string | undefined` under
//! `exactOptionalPropertyTypes`, while the dedicated per-prop check continues
//! to reject a genuinely wrong value (#11048, preserving #9658).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn issue_11048_kebab_aria_satisfies_declared_prop_without_weakening_its_value() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-11048-declared-aria-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ ariaLabel: string; itemLabel: string }>()
</script>

<template><span>{{ ariaLabel }} {{ itemLabel }}</span></template>
"#,
            ),
            (
                "src/Optional.vue",
                r#"<script setup lang="ts">
defineProps<{ ariaLabel?: string }>()
</script>

<template><span>{{ ariaLabel }}</span></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
import Optional from './Optional.vue'

const maybe = undefined as string | undefined
</script>

<template>
  <Child aria-label="kebab aria" item-label="kebab item" />
  <Child ariaLabel="camel aria" itemLabel="camel item" />
  <Optional :aria-label="maybe" />
  <Optional aria-hidden="true" />
  <Child :aria-label="123" item-label="wrong aria value" />
</template>
"#,
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "exactOptionalPropertyTypes": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "vueCompilerOptions": { "strictTemplates": true },
  "include": ["src/**/*"]
}"#,
    )
    .expect("enable exact optional prop semantics");

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot.len(),
        1,
        "only the authored wrong ariaLabel value must fail: {snapshot:#?}"
    );
    let (file, code, message) = &snapshot[0];
    assert_eq!(file.as_str(), "src/Parent.vue");
    assert_eq!(*code, Some(2322));
    assert_eq!(
        message, "13:11: Type 'number' is not assignable to type 'string'.",
        "the declared prop value error must stay on aria-label: {snapshot:#?}"
    );
}
