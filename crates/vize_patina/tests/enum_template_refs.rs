//! The compiler and undefined-ref rule accept the same enum declarations (#7893, #7896).

use vize_patina::Linter;

#[test]
fn regular_and_const_enums_from_both_script_blocks_are_defined_template_refs() {
    for source in [
        include_str!("../../../tests/fixtures/sfc/enum-template-bindings/Badge.vue"),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/enum-template-bindings-7893/Badge.vue.txt"
        ),
    ] {
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["vue/no-undefined-refs".into()]))
            .lint_sfc(source, "Badge.vue");
        assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    }
}

#[test]
fn recognizing_const_enums_does_not_hide_unrelated_undefined_refs() {
    let source = r#"<script setup lang="ts">
const enum Tone { Warn = "warn" }
</script>
<template><span>{{ Tone.Warn }} {{ missing }}</span></template>"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-undefined-refs".into()]))
        .lint_sfc(source, "Badge.vue");
    assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].message,
        "Variable 'missing' is not defined"
    );
}
