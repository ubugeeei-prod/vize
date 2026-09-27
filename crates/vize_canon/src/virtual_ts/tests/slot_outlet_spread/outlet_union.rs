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
        "type __VizeSlotPayloadOf<__F> = __F extends { (props: infer __A0, ...args: any[]): any;",
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

/// An inferred outlet payload types a static string attribute as the plain
/// object literal does, `string`: vue-tsc exposes `<slot str="str" />` as
/// `{ str: string }`, so keeping the literal would diverge from it.
#[test]
fn inferred_outlet_payload_widens_static_attributes() {
    let code = generate(r#"<div><slot name="side" viewMode="sp" :count="1" /></div>"#);
    assert_eq!(
        (
            code.matches("\"viewMode\": \"sp\",").count(),
            code.matches("\"count\": 1,").count(),
            code.matches("as const").count(),
        ),
        (1, 1, 0),
        "{code}"
    );
}
