//! Camel-case aliases must not erase a prop's requiredness (#3781).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

/// Oracle: vue-tsc 3.3.4 with TypeScript 6.0.3 reports one TS2345 at 7:4,
/// 8:4, 9:4 and 10:4 respectively, while accepting every complete/optional
/// usage.
#[test]
fn a_single_required_camel_prop_stays_required() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "single-required-camel-prop",
        &[
            (
                "src/OneProp.vue",
                r#"<script setup lang="ts">
defineProps<{ someValue: number }>();
</script>
<template><span>{{ someValue }}</span></template>
"#,
            ),
            (
                "src/TwoProps.vue",
                r#"<script setup lang="ts">
defineProps<{ someValue: number; other: string }>();
</script>
<template><span>{{ someValue }}{{ other }}</span></template>
"#,
            ),
            (
                "src/OptionalProp.vue",
                r#"<script setup lang="ts">
defineProps<{ someValue?: number }>();
</script>
<template><span>{{ someValue }}</span></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import OneProp from './OneProp.vue'
import TwoProps from './TwoProps.vue'
import OptionalProp from './OptionalProp.vue'
</script>
<template>
  <OneProp />
  <TwoProps />
  <TwoProps :some-value="1" />
  <TwoProps other="ok" />
  <OneProp :some-value="1" />
  <OneProp :someValue="1" />
  <OptionalProp />
  <OptionalProp :some-value="1" />
  <OptionalProp :someValue="1" />
  <TwoProps :some-value="1" other="ok" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let native_tail = "__VizePublicComponentAttrs & { 'aria-activedescendant'?: unknown; 'aria-atomic'?: unknown; 'aria-autocomplete'?: unknown; 'aria-busy'?: unknown; 'aria-checked'?: unknown; 'aria-colcount'?: unknown; ... 184 more ...; ref_key?: unknown; } & __VizeCustomDataFallthroughAttrs & Partial<...>";
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/Parent.vue"),
                Some(2345),
                cstr!(
                    "10:4:error Argument of type '{{ other: string; }}' is not assignable to parameter of type '__VizeComponentCheckProps<Props, {native_tail}>'.\nProperty 'someValue' is missing in type '{{ other: string; }}' but required in type '{{ readonly someValue: number; readonly other: string; }}'."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                cstr!(
                    "7:4:error Argument of type '{{}}' is not assignable to parameter of type '__VizeComponentCheckProps<Props, {native_tail}>'.\nProperty 'someValue' is missing in type '{{}}' but required in type '{{ readonly someValue: number; }}'."
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                cstr!(
                    "8:4:error Argument of type '{{}}' is not assignable to parameter of type '__VizeComponentCheckProps<Props, {native_tail}>'.\nType '{{}}' is missing the following properties from type '{{ readonly someValue: number; readonly other: string; }}': someValue, other"
                ),
            ),
            (
                String::from("src/Parent.vue"),
                Some(2345),
                cstr!(
                    "9:4:error Argument of type '{{ someValue: number; }}' is not assignable to parameter of type '__VizeComponentCheckProps<Props, {native_tail}>'.\nProperty 'other' is missing in type '{{ someValue: number; }}' but required in type '{{ readonly someValue: number; readonly other: string; }}'."
                ),
            ),
        ],
        "complete vue-tsc oracle and clean camel/kebab usages"
    );
}
