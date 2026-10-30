//! A runtime prop with a `default` is published as optional to the parent even
//! when it is `required: true`, the way Vue's `$props` (and therefore `vue-tsc`)
//! publishes it; a required prop without a default and a required plain
//! `Boolean` prop stay required.

use super::super::{analyze_options_api_script, generate_virtual_ts_with_offsets_options_api};
use crate::virtual_ts::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

fn has_line(code: &str, expected: &str) -> bool {
    code.lines().any(|line| line == expected)
}

#[test]
fn defaulted_required_runtime_props_are_optional_for_the_parent() {
    let script = "defineProps({\n  label: { type: String, default: '', required: true },\n  flag: { type: Boolean, default: false, required: true },\n  title: { type: String, required: true },\n  note: { type: String, default: undefined, required: true },\n  plain: { type: String },\n})\n";
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();
    let output =
        generate_virtual_ts_with_offsets(&summary, Some(script), None, 0, 0, &Default::default());
    let code = output.code.as_str();

    for expected in [
        "  \"label\"?: string;",
        "  \"flag\": boolean;",
        "  \"title\": string;",
        "  \"note\"?: string;",
        "  \"plain\"?: string;",
    ] {
        assert!(
            has_line(code, expected),
            "the published Props must contain `{expected}`:\n{code}"
        );
    }
    assert!(
        !has_line(code, "  \"label\": string;"),
        "a defaulted prop must not be published as required:\n{code}"
    );
}

#[test]
fn options_api_prop_shape_relaxes_defaulted_required_props() {
    let script = "import { defineComponent } from 'vue'\n\nexport default defineComponent({\n  props: {\n    label: { type: String, default: '', required: true },\n    flag: { type: Boolean, default: false, required: true },\n  },\n})\n";
    let summary = analyze_options_api_script(script);
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
    );
    let code = output.code.as_str();

    assert!(
        has_line(
            code,
            "type $OptionsPropRequired<T> = T extends { required: true } ? T extends { default: any } ? T extends { type: BooleanConstructor } ? true : false : true : false;"
        ),
        "the deferred prop shape must relax a defaulted required prop for the parent:\n{code}"
    );
    assert!(
        has_line(
            code,
            "export type Props = $OptionsPropShape<Awaited<ReturnType<typeof __setup>>[\"__vize_options_props\"]>;"
        ),
        "the published Props must still come from the deferred runtime object:\n{code}"
    );
}
