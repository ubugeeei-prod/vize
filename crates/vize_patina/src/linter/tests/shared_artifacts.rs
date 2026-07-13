use super::{LintPreset, Linter};
use vize_armature::Parser;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::Allocator;
use vize_relief::ReliefSnapshot;

#[test]
fn shared_sfc_artifacts_match_the_production_lint_result() {
    let source = r#"<script setup lang="ts">
const unused = 1
</script>
<template><ul><li v-for="(item, index) in items">{{ item }}</li></ul></template>
<style>.bad { color: red !important; }</style>"#;
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let expected = linter.lint_sfc(source, "Shared.vue");
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Shared.vue".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let template = descriptor.template.as_ref().unwrap();
    let allocator = Allocator::default();
    let (root, parse_errors) = Parser::new(allocator.as_bump(), &template.content).parse();
    let snapshot = ReliefSnapshot::from_root(&root);
    let analysis = crate::linter::engine::analyze_descriptor_for_lint(&descriptor, Some(&root));

    let actual = linter.lint_sfc_with_shared_artifacts(
        source,
        "Shared.vue",
        &descriptor,
        Some((&snapshot, &parse_errors)),
        Some(&analysis),
    );
    let summary = |result: &crate::LintResult| {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.rule_name,
                    diagnostic.severity,
                    diagnostic.start,
                    diagnostic.end,
                    diagnostic.message.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(actual.error_count, expected.error_count);
    assert_eq!(actual.warning_count, expected.warning_count);
    assert_eq!(summary(&actual), summary(&expected));
}
