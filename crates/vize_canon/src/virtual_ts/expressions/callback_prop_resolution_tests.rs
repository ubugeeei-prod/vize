use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

#[test]
fn callback_resolution_keeps_inference_and_branch_selection_separate() {
    let script = r#"import Child from "./Child.vue"
"#;
    let callback = "(item) => item.id";
    let template = r#"<Child kind="text" :items="[{ id: 1 }]" :pick="(item) => item.id" />"#;

    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        output.code.contains(
            "const __vize_resolved_Child_props_0 = (undefined as unknown as __VizePropsResolver<typeof Child>)({"
        ),
        "inference resolver must keep the authored callback:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "const __vize_selected_Child_props_0 = (undefined as unknown as __VizePropsSelector<typeof __vize_resolved_Child_props_0>)({"
        ),
        "selector must be constrained by the inferred props union:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "__VizeResolvedProp<typeof __vize_resolved_Child_props_0, typeof __vize_selected_Child_props_0, 'pick'"
        ),
        "mapped owner must use both inference and selected-branch types:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches(callback).count(),
        2,
        "only the inference call and mapped owner keep the callback"
    );
    assert!(
        output.code.contains(
            "    // @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n    \"pick\": (item) => item.id,"
        ),
        "inference-only callback line must not own duplicate diagnostics:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches(r#""pick": undefined as any"#).count(),
        2,
        "the selector and whole-props checker must erase the callback"
    );
    assert!(
        output
            .code
            .contains("type __VizeMissingProp = { readonly __vizeMissingProp: unique symbol };")
            && output
                .code
                .contains("K extends keyof R ? { value: R[K] } : __VizeMissingProp"),
        "present `never` must remain distinguishable from a missing key"
    );

    let callback_start = template.find(callback).expect("callback present");
    let callback_range = callback_start..callback_start + callback.len();
    assert_eq!(
        output
            .mappings
            .iter()
            .flat_map(|mapping| &mapping.sub_spans)
            .filter(|span| span.src_range == callback_range)
            .count(),
        1,
        "the mapped per-prop owner must remain the sole diagnostic owner"
    );
}
