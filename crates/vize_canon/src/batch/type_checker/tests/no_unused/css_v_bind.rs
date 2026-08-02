//! A setup binding read only by CSS `v-bind()` is consumed, not unused (#1876).
//!
//! `noUnusedLocals` narrows the generated `void <binding>;` anchors to names the
//! template references so genuinely unused user bindings still report `TS6133`.
//! `<style>` was outside that reach, so `div { color: v-bind(color) }` left
//! `color` looking unreferenced and every such binding was published as a
//! `TS6133` error the Nuxt 2 / Vue 2.7 report in #1876 could not distinguish
//! from a real one.
//!
//! The expectations below are the exact diagnostic set `vue-tsc 3.3.4` produces
//! for the same sources under the same `tsconfig.json`: nothing for the four
//! CSS-consumed bindings, and `TS6133` for the three that no template, script,
//! or live `v-bind()` reads.

use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use super::write_no_unused_tsconfig;

/// Bindings a live `v-bind()` reads, in the spellings the extractor must see
/// through: a bare name, a quoted expression, two properties of one rule, and a
/// member expression.
const CONSUMED_BY_CSS: &[(&str, &str)] = &[
    (
        "src/BareName.vue",
        r#"<script setup lang="ts">
const color = 'red'
</script>

<template><div /></template>

<style scoped>
div { color: v-bind(color); }
</style>
"#,
    ),
    (
        "src/QuotedExpression.vue",
        r#"<script setup lang="ts">
const size = 10
</script>

<template><div /></template>

<style scoped>
div { height: v-bind("size + 'px'"); }
</style>
"#,
    ),
    (
        "src/TwoBindings.vue",
        r#"<script setup lang="ts">
const fg = 'red'
const bg = 'blue'
</script>

<template><div /></template>

<style scoped>
div { color: v-bind(fg); background: v-bind(bg); }
</style>
"#,
    ),
    (
        "src/MemberExpression.vue",
        r#"<script setup lang="ts">
const theme = { color: 'red' }
</script>

<template><div /></template>

<style module>
.row { color: v-bind("theme.color"); }
</style>
"#,
    ),
];

/// Bindings no live `v-bind()` reads. `CommentedOut.vue` and `QuotedText.vue`
/// are the guard rails: the SFC parser's `v-bind()` extractor skips CSS comments
/// and string literals, so neither spelling may rescue its binding.
const STILL_UNUSED: &[(&str, &str)] = &[
    (
        "src/CommentedOut.vue",
        r#"<script setup lang="ts">
const color = 'red'
</script>

<template><div /></template>

<style scoped>
/* color: v-bind(color); */
div { color: red; }
</style>
"#,
    ),
    (
        "src/QuotedText.vue",
        r#"<script setup lang="ts">
const icon = 'star'
</script>

<template><div /></template>

<style scoped>
div::after { content: "v-bind(icon)"; }
</style>
"#,
    ),
    (
        "src/BesideALiveVBind.vue",
        r#"<script setup lang="ts">
const color = 'red'
const unusedLocal = 'blue'
</script>

<template><div /></template>

<style scoped>
div { color: v-bind(color); }
</style>
"#,
    ),
];

#[test]
fn css_v_bind_consumes_its_bindings_without_hiding_unused_ones() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let files: Vec<(&str, &str)> = CONSUMED_BY_CSS
        .iter()
        .chain(STILL_UNUSED)
        .copied()
        .collect();
    let project_root = create_project_case_without_node_modules("css-v-bind-no-unused", &files);
    write_no_unused_tsconfig(&project_root);

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    // `snapshot_project_diagnostics` sorts by (file, code, message), so the
    // expected list is the whole diagnostic surface of the project in that
    // order — a new finding anywhere fails this assertion.
    assert_eq!(
        snapshot,
        vec![
            unused(
                "src/BesideALiveVBind.vue",
                3,
                7,
                "'unusedLocal' is declared but its value is never read."
            ),
            unused(
                "src/CommentedOut.vue",
                2,
                7,
                "'color' is declared but its value is never read."
            ),
            unused(
                "src/QuotedText.vue",
                2,
                7,
                "'icon' is declared but its value is never read."
            ),
        ],
        "expected the vue-tsc diagnostic surface for these sources"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn unused(
    file: &str,
    line: u32,
    column: u32,
    message: &str,
) -> (vize_carton::String, Option<u32>, vize_carton::String) {
    (
        vize_carton::String::from(file),
        Some(6133),
        vize_carton::cstr!("{line}:{column}:error {message}"),
    )
}
