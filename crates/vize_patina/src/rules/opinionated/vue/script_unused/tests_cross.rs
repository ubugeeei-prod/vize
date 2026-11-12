//! #12924: the two script blocks are one module, and a template assignment
//! reads the binding it writes.

use super::tests_issue::reports;

fn unused(line: usize, column: usize, message: &str) -> (usize, usize, &'static str, String) {
    (
        line,
        column,
        "vue/no-unused-script-vars",
        message.to_string(),
    )
}

#[test]
fn cross_script_reads_and_template_assignment_count_as_uses() {
    let normal_to_setup = r#"<script lang="ts">
import { computed } from "vue";
import Ghost from "./Ghost.vue";

export const LIMIT = 3;
</script>

<script setup lang="ts">
const count = computed(() => LIMIT * 2);
</script>

<template>
  <p>{{ count }} / {{ LIMIT }}</p>
</template>
"#;
    assert_eq!(
        reports(normal_to_setup, None),
        vec![unused(
            3,
            8,
            "'Ghost' is defined but never used. Allowed unused vars must match /^_/u",
        )]
    );

    let setup_to_normal = r#"<script lang="ts">
export function double(n: number): number {
  return clamp(n * 2);
}
</script>

<script setup lang="ts">
import { clamp } from "./clamp";

defineProps<{ value: number }>();
</script>

<template>
  <p>{{ value }}</p>
</template>
"#;
    assert_eq!(reports(setup_to_normal, None), Vec::new());

    let normal_to_template = r#"<script lang="ts">
import { clamp } from "./clamp";

function label(n: number): string {
  return `#${n}`;
}
</script>

<script setup lang="ts">
defineProps<{ value: number }>();
</script>

<template>
  <p>{{ label(clamp(value)) }}</p>
</template>
"#;
    assert_eq!(reports(normal_to_template, None), Vec::new());

    let template_assign = r#"<script setup lang="ts">
import { useFilter } from "./useFilter";

const { keyword } = useFilter();
let draft = "";
</script>

<template>
  <SearchBox @update:keyword="keyword = $event" @input="draft = 'x'" />
</template>
"#;
    assert_eq!(
        reports(template_assign, None),
        vec![unused(
            5,
            5,
            "'draft' is assigned a value but never used. Allowed unused vars must match /^_/u",
        )]
    );
}
