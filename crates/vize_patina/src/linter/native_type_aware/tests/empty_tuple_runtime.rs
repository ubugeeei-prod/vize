use super::super::{RULE_NO_FLOATING_PROMISES, lint_sfc_with_corsa};
use super::{corsa_available, failure_context};
use crate::{LintPreset, Linter};

const CASES: [(&str, &str); 4] = [
    (
        "EmptyAsConst.vue",
        r#"<script setup lang="ts">
const results = [] as const;
Promise.resolve("type-aware control");
</script>

<template><p>{{ results.length }}</p></template>"#,
    ),
    (
        "EmptyPromiseAll.vue",
        r#"<script setup lang="ts">
const all = Promise.all([]);
Promise.resolve("type-aware control");
</script>

<template><p>{{ all }}</p></template>"#,
    ),
    (
        "EmptySpread.vue",
        r#"<script setup lang="ts">
const top = Math.max(...[]);
Promise.resolve("type-aware control");
</script>

<template><p>{{ top }}</p></template>"#,
    ),
    (
        "FilledAsConst.vue",
        r#"<script setup lang="ts">
const results = ["a"] as const;
Promise.resolve("type-aware control");
</script>

<template><p>{{ results.length }}</p></template>"#,
    ),
];

#[test]
fn empty_tuple_queries_do_not_discard_the_files_type_rules() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_type_aware_lint(true)
        .with_enabled_rules(Some(vec![RULE_NO_FLOATING_PROMISES.into()]));
    for (filename, source) in CASES {
        let result = lint_sfc_with_corsa(&linter, source, filename);
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "type/corsa-runtime"),
            "{}",
            failure_context(&result)
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == RULE_NO_FLOATING_PROMISES),
            "{}",
            failure_context(&result)
        );
    }
}
