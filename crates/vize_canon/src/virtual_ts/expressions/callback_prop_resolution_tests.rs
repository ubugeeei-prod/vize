use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

#[test]
fn callback_resolution_keeps_inference_and_branch_selection_separate() {
    let script = r#"import Child from "./Child.vue"
"#;
    let callback = "(item) => item.id";
    let template = r#"<Child kind="text" :items="[{ id: 1 }]" :pick="(item) => item.id" />"#;

    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert_eq!(
        output
            .code
            .matches("const __vize_resolved_Child_props_0 = (undefined as unknown as __VizeResolvedContextProps)((undefined as unknown as __VizePropsResolver<typeof Child>)({")
            .count(),
        1,
        "inference resolver must keep the authored callback:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const __vize_selected_Child_props_0 = (undefined as unknown as __VizePropsSelector<typeof __vize_resolved_Child_props_0>)({")
            .count(),
        1,
        "selector must be constrained by the inferred props union:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("__VizeResolvedProp<typeof __vize_resolved_Child_props_0, typeof __vize_selected_Child_props_0, 'pick'")
            .count(),
        1,
        "mapped owner must use both inference and selected-branch types:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches(callback).count(),
        2,
        "only the inference call and mapped owner keep the callback"
    );
    assert_eq!(
        output
            .code
            .matches(
                "// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n    \"pick\": (item) => item.id,"
            )
            .count(),
        1,
        "inference-only callback line must not own duplicate diagnostics:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches(r#""pick": undefined as any"#).count(),
        1,
        "the selector must erase the callback"
    );
    assert_eq!(
        output.code.matches(r#""pick": undefined as never"#).count(),
        1,
        "the whole-props checker must not diagnose the erased callback"
    );
    assert_eq!(
        output
            .code
            .matches("type __VizeMissingProp = { readonly __vizeMissingProp: unique symbol };")
            .count(),
        1,
        "present `never` must remain distinguishable from a missing key:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("K extends keyof R ? { value: R[K] } : __VizeMissingProp")
            .count(),
        1,
        "present `never` must remain distinguishable from a missing key:\n{}",
        output.code
    );

    let callback_start = template.find(callback).expect("callback present");
    let callback_range = callback_start..callback_start + callback.len();
    assert_eq!(
        output
            .mapping
            .spans()
            .iter()
            .flat_map(|mapping| &mapping.sub_spans)
            .filter(|span| span.src_range == callback_range)
            .count(),
        1,
        "the mapped per-prop owner must remain the sole diagnostic owner"
    );
}

#[test]
fn multiline_callback_starts_on_the_ignored_property_line() {
    let script = "import Guard from './Guard.vue'\n";
    let template = r#"<Guard
  :check="
    (params) =>
      !!params.id
  "
/>"#;

    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert_eq!(
        output
            .code
            .matches(
                "// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.\n    \"check\": (params) =>"
            )
            .count(),
        1,
        "the ignored line must contain the callback parameter token:\n{}",
        output.code
    );
}

#[test]
fn callback_parameter_after_a_broken_parenthesis_stays_on_the_ignored_line() {
    let script = "import Guard from './Guard.vue'\n";
    let template = "<Guard\n  :check=\"(\n  params\n) => !!params.id\"\n/>";

    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let ignored = output
        .code
        .lines()
        .find(|line| line.contains("\"check\":"))
        .unwrap_or("");
    assert_eq!(
        ignored, "    \"check\": (   params ) => !!params.id,",
        "the ignored property line must keep the parameter:\n{ignored}\n{}",
        output.code
    );
}
