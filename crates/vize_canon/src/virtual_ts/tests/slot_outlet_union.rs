use crate::virtual_ts::generate_virtual_ts;

/// The virtual TS of a component without a script block.
fn generate(template: &str) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts(&summary, None, Some(&root), 0).code
}

/// A parent's slot payload is resolved through the overload-aware helper, so a
/// child that renders one named slot from several `<slot>` outlets (an
/// intersection of one function per outlet) contributes every outlet's
/// payload, not only the last overload's. The helper is declared exactly once,
/// next to the payload alias that references it.
#[test]
fn slot_payload_helper_collects_every_overload() {
    let code = generate(
        r#"<child><template #panel="{ viewMode }"><preview :viewMode="viewMode" /></template></child>"#,
    );
    for alias in [
        "type __VizeSlotPayloadOf<__F> = __F extends { (props: infer __A, ...args: any[]): any;",
        "type __VizeSlotPayloadUnify<__P> = __VizeIsAny<__P> extends true ? __P : [__P] extends [__VizeSlotPayloadIntersection<__P>] ? __P : __VizeSlotPayloadMerge<__P>;",
        "__VizeSlotPayloadUnify<__VizeSlotPayloadOf<NonNullable<__S[__K]>>>",
    ] {
        assert_eq!(
            code.matches(alias).count(),
            1,
            "the slot payload helper must be declared once and used by the payload alias:\n{code}"
        );
    }
}

/// An inferred outlet payload keeps a static string attribute as its literal
/// type, as the runtime value is exactly that string; the checking-side
/// literal is contextually typed and stays as authored.
#[test]
fn inferred_outlet_payload_keeps_static_attribute_literals() {
    let code = generate(
        r#"<div><slot name="side" viewMode="sp" :count="1" /><slot name="flag" disabled /></div>"#,
    );
    assert!(
        code.contains("\"viewMode\": \"sp\" as const,"),
        "a static string attribute keeps its literal type:\n{code}"
    );
    assert!(
        code.contains("\"count\": 1,") && code.contains("\"disabled\": true,"),
        "bound values and valueless attributes are unchanged:\n{code}"
    );
    let script = "defineSlots<{ side(props: { viewMode: string }): any }>()";
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(
        &allocator,
        r#"<div><slot name="side" viewMode="sp" /></div>"#,
    );
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let checked = generate_virtual_ts(&summary, Some(script), Some(&root), 0).code;
    assert!(
        checked.contains("\"viewMode\": \"sp\",") && !checked.contains("as const"),
        "the checking-side literal stays as authored:\n{checked}"
    );
}
