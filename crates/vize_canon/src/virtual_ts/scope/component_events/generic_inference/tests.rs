//! A guarded emit-props inference call stays out of the template's control
//! flow and restates the whole guard inside its own arrow.
use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

const SCRIPT: &str = r#"import Child from './Child.vue'
import Host from './Host.vue'
const rows = [{ ready: true, children: ['text'] }]
const enabled = true
const value = 'text'
const onSave = (value: number) => value
const save = (row: unknown, value: unknown) => value
"#;

fn generate(template: &str) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(SCRIPT);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(SCRIPT),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions::default(),
    )
    .code
}

#[test]
fn a_guarded_inference_call_is_not_an_immediately_invoked_arrow() {
    let code = generate(r#"<Child v-if="enabled" :value="value" @save="onSave" />"#);
    assert_eq!(
        code.matches("const __vize_emit_props_0 = (void 0, () => {\n")
            .count(),
        1,
        "{code}"
    );
    assert_eq!(code.matches(" = (() => {").count(), 0, "{code}");
    assert_eq!(code.matches("if ((enabled)) return ").count(), 1, "{code}");
    assert_eq!(
        code.matches("  return undefined as never;\n").count(),
        1,
        "{code}"
    );
}

#[test]
fn a_listener_block_prefix_is_restated_inside_the_arrow() {
    let code = generate(
        r#"<template v-for="row in rows"><Host v-if="row.ready"><template #content><template v-for="child in row.children"><Child v-if="enabled" :value="child" @save="() => save(row, value)" /></template></template></Host></template>"#,
    );
    // The open block enforces the v-for alias term, a strict prefix of the
    // usage guard. The inlined call used to test only the remaining term.
    let block = code
        .find("// @ts-ignore Listener branch guard; authored v-if checks own diagnostics.\n")
        .unwrap_or_else(|| panic!("{code}"));
    let after_block = code.get(block..).unwrap_or_default();
    assert_eq!(
        after_block.lines().nth(1).map(str::trim_start),
        Some("if ((row.ready)) {"),
        "{code}"
    );
    let arrow = after_block
        .find("const __vize_emit_props_0 = (void 0, () => {\n")
        .unwrap_or_else(|| panic!("{code}"));
    let mut body = after_block.get(arrow..).unwrap_or_default().lines();
    assert_eq!(
        body.nth(1).map(str::trim_start),
        Some("// @ts-ignore Inference-only guard; authored binding checks own diagnostics."),
        "{code}"
    );
    assert_eq!(
        body.next()
            .map(str::trim_start)
            .and_then(|guard| guard.split_once(" return "))
            .map(|(condition, _)| condition),
        Some("if ((row.ready) && (enabled))"),
        "{code}"
    );
}
