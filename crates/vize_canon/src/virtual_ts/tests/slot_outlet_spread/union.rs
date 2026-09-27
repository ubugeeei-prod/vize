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

/// External authored slot functions retain the existing main-line inference:
/// their last signature supplies the payload. Generated inferred outlets have
/// already been merged into one signature before this boundary.
#[test]
fn slot_payload_helper_keeps_authored_signature_fallback() {
    let code = generate(
        r#"<child><template #panel="{ viewMode }"><preview :viewMode="viewMode" /></template></child>"#,
    );
    assert_eq!(
        code.matches(
            "NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never"
        )
        .count(),
        1,
        "authored slot inference must keep its existing fallback:\n{code}"
    );
}

/// Repeated same-named outlets retain string discriminants before merging.
/// Single outlets preserve their existing public string payload type.
#[test]
fn inferred_outlet_payload_keeps_static_attribute_literals() {
    let code = generate(
        r#"<div><slot name="side" viewMode="sp" :count="1" /><slot name="side" viewMode="pc" /><slot name="flag" disabled /></div>"#,
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

#[test]
fn single_outlet_payload_preserves_string_widening() {
    let code = generate(r#"<slot name="single" viewMode="sp" :count="1" />"#);
    assert!(
        code.contains("\"viewMode\": \"sp\",") && !code.contains("as const"),
        "one outlet must retain its public string payload type:\n{code}"
    );
}
