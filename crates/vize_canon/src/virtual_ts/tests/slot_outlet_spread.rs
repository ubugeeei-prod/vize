use crate::virtual_ts::generate_virtual_ts;

#[test]
fn infers_the_spread_expression_type() {
    let script = r#"const attrs = { label: 'ok' }
defineSlots<{ default(props: { label: string }): any }>()
"#;
    let template = r#"<slot v-bind="attrs" />"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("function __vizeSlotOutletSpread<__Expected>()"),
        "slot outlet spread helper must split supplied and inferred generics:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            r#"...__vizeSlotOutletSpread<__VizeSlotOutletPayload<Slots, "default">>()(attrs),"#
        ),
        "slot outlet spread call must infer its expression argument:\n{}",
        output.code
    );
}
