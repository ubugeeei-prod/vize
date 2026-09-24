#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use super::*;

#[test]
fn slot_outlet_spreads_reject_incompatible_nested_payloads() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let clean = r#"<script setup lang="ts">
const row = { item: { id: 1, label: 'ok' } };
defineSlots<{ row(props: { item: { id: number; label: string } }): unknown }>();
</script>
<template>
  <slot name="row" v-bind="row" />
</template>
"#;
    for (name, declaration, line) in [
        ("spread-nested", "", 6),
        (
            "spread-shadowed-generic-extract",
            "type Extract<T, U> = { unrelated: boolean };\n",
            7,
        ),
        ("spread-shadowed-extract", "type Extract = string;\n", 7),
    ] {
        let clean = clean.replacen("const row", &format!("{declaration}const row"), 1);
        let broken = clean.replace("id: 1", "id: 'wrong'");
        let project = create_case_with_files(name, &[("src/App.vue", &clean)]);
        for target in ["src/App.vue", "src"] {
            assert_clean(&project, &corsa_path, target);
            std::fs::write(project.join("src/App.vue"), &broken).unwrap();
            let report = run_check_json(&project, &corsa_path, target);
            assert!(
                !report.status.success(),
                "bad spread passed: {}",
                report.json
            );
            assert_eq!(report.json["errorCount"], 1, "{}", report.json);
            let message = concat!(
                "Argument of type '{ item: { id: string; label: string; }; }' is not assignable to parameter of type '{ item: { id: number; label: string; }; }'.",
                "\nThe types of 'item.id' are incompatible between these types.",
                "\nType 'string' is not assignable to type 'number'.",
            );
            assert_diagnostic(
                &report.json,
                "src/App.vue",
                line,
                4,
                2345,
                &format!("error:{line}:4 [TS2345] {message}"),
            );
            std::fs::write(project.join("src/App.vue"), &clean).unwrap();
            assert_clean(&project, &corsa_path, target);
        }
        std::fs::remove_dir_all(project).unwrap();
    }
}

#[test]
fn slot_outlet_spreads_preserve_partial_ordered_optional_and_generic_payloads() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = create_case_with_files(
        "spread-valid",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
const row = { item: { id: 1, label: 'ok' } };
const wrong = { item: { id: 'wrong', label: 'bad' } };
const callbacks = { format: (value: number) => value.toFixed() };
const optional = undefined as unknown as { item?: { id: number; label: string } };
defineSlots<{
  row(props: { item: { id: number; label: string }; format: (value: number) => string }): unknown;
  optional(props: { item?: { id: number; label: string } }): unknown;
}>();
</script>
<template>
  <slot name="row" v-bind="row" :format="value => value.toFixed()" />
  <slot name="row" v-bind="callbacks" :item="row.item" />
  <slot name="row" v-bind="wrong" :item="row.item" :format="callbacks.format" />
  <slot name="row" v-bind="optional" :item="row.item" :format="callbacks.format" />
  <slot name="row" v-bind="{ ...row, ...callbacks }" />
  <slot name="row" v-bind="{ item: row.item, format: value => value.toFixed() }" />
  <slot name="row" v-bind="{ item: row.item, format: (value: number) => value.toFixed() }" />
  <slot name="optional" v-bind="optional" />
</template>
"#,
            ),
            (
                "src/Generic.vue",
                r#"<script setup lang="ts" generic="M extends string">
type SlotProps<T> = T extends string ? { modelValue: T } : Record<string, unknown>;
defineSlots<{ default(props: SlotProps<M>): unknown }>();
const slotProps = undefined as unknown as SlotProps<M>;
</script>
<template>
  <slot v-bind="slotProps" />
</template>
"#,
            ),
            (
                "src/Unknown.vue",
                r#"<script setup lang="ts">
defineSlots<{ [name: string]: (props: unknown) => unknown }>();
const slotName = 'default';
const slotProps = undefined as unknown;
</script>
<template>
  <slot :name="slotName" v-bind="slotProps" />
</template>
"#,
            ),
            (
                "src/Empty.vue",
                r#"<script setup lang="ts">
defineSlots<{ default(props: {}): unknown }>();
</script>
<template>
  <slot v-bind="null" />
  <slot v-bind="undefined" />
  <slot v-bind="false" />
</template>
"#,
            ),
        ],
    );
    for target in [
        "src/App.vue",
        "src/Generic.vue",
        "src/Unknown.vue",
        "src/Empty.vue",
        "src",
    ] {
        assert_clean(&project, &corsa_path, target);
    }
    std::fs::remove_dir_all(project).unwrap();
}

#[test]
fn slot_outlet_spreads_report_exact_required_ordered_and_callback_errors() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let cases = [
        (
            "spread-missing",
            r#"<script setup lang="ts">
const row = { label: 'ok' };
defineSlots<{ row(props: { item: number; label: string }): unknown }>();
</script>
<template>
  <slot name="row" v-bind="row" />
</template>
"#,
            6,
            4,
            2741,
            "error:6:4 [TS2741] Property 'item' is missing in type '{ label: string; }' but required in type '{ item: number; label: string; }'.",
        ),
        (
            "spread-optional",
            r#"<script setup lang="ts">
const row = undefined as unknown as { item?: number; label: string };
defineSlots<{ row(props: { item: number; label: string }): unknown }>();
</script>
<template>
  <slot name="row" v-bind="row" />
</template>
"#,
            6,
            4,
            2345,
            concat!(
                "error:6:4 [TS2345] Argument of type '{ item?: number; label: string; }' is not assignable to parameter of type '{ item: number; label: string; }'.",
                "\nTypes of property 'item' are incompatible.",
                "\nType 'number | undefined' is not assignable to type 'number'.",
                "\nType 'undefined' is not assignable to type 'number'.",
            ),
        ),
        (
            "spread-order",
            r#"<script setup lang="ts">
const row = { item: 1, label: 'ok' };
const wrong = { item: 'wrong' };
defineSlots<{ row(props: { item: number; label: string }): unknown }>();
</script>
<template>
  <slot name="row" v-bind="{ ...row, ...wrong }" />
</template>
"#,
            7,
            4,
            2345,
            concat!(
                "error:7:4 [TS2345] Argument of type '{ label: string; item: string; }' is not assignable to parameter of type '{ item: number; label: string; }'.",
                "\nTypes of property 'item' are incompatible.",
                "\nType 'string' is not assignable to type 'number'.",
            ),
        ),
        (
            "spread-callback",
            r#"<script setup lang="ts">
const row = { item: 1 };
defineSlots<{ row(props: { item: number; format: (value: number) => string }): unknown }>();
</script>
<template>
  <slot name="row" v-bind="row" :format="value => value.toUpperCase()" />
</template>
"#,
            6,
            57,
            2339,
            "error:6:57 [TS2339] Property 'toUpperCase' does not exist on type 'number'.",
        ),
        (
            "spread-inline-callback",
            r#"<script setup lang="ts">
const row = { item: 1 };
defineSlots<{ row(props: { item: number; format: (value: number) => string }): unknown }>();
</script>
<template>
  <slot name="row" v-bind="{ item: row.item, format: value => value.toUpperCase() }" />
</template>
"#,
            6,
            69,
            2339,
            "error:6:69 [TS2339] Property 'toUpperCase' does not exist on type 'number'.",
        ),
    ];
    for (name, source, line, column, code, expected) in cases {
        let project = create_case_with_files(name, &[("src/App.vue", source)]);
        for target in ["src/App.vue", "src"] {
            let report = run_check_json(&project, &corsa_path, target);
            assert!(!report.status.success(), "{}", report.json);
            assert_eq!(report.json["errorCount"], 1, "{}", report.json);
            assert_diagnostic(&report.json, "src/App.vue", line, column, code, expected);
        }
        std::fs::remove_dir_all(project).unwrap();
    }
}
