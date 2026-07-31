//! The positive direction: a declared prop referenced nowhere.

use super::{lint_sfc, owned, unused};

// --- The recovered case: an unassigned defineProps -------------------------

#[test]
fn reports_a_prop_referenced_nowhere_issue_3416() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "msg")]);
}

#[test]
fn reports_only_the_prop_the_template_never_reads() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; unused: number }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "unused")]);
}

#[test]
fn reports_a_prop_of_a_runtime_array_declaration() {
    let sfc = r#"<script setup>
defineProps(['msg'])
</script>

<template>
  <div>hi</div>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "msg")]);
}

#[test]
fn reports_a_prop_the_destructuring_pattern_omits() {
    let sfc = r#"<script setup lang="ts">
const { msg } = defineProps<{ msg: string; other: number }>();
const upper = msg.toUpperCase();
</script>

<template>
  <div>{{ upper }}</div>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "other")]);
}

// --- A name that is not a reference must still report ---------------------

#[test]
fn reports_a_prop_named_only_in_an_html_comment_or_text() {
    // Neither is a compiled expression, so neither is a reference.
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <!-- msg -->
  <p title="msg">msg</p>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "msg")]);
}

#[test]
fn reports_a_prop_named_only_inside_a_v_pre_region() {
    // Vue never compiles a `v-pre` region, so `{{ msg }}` there is literal text.
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <pre v-pre>{{ msg }}</pre>
</template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "msg")]);
}
