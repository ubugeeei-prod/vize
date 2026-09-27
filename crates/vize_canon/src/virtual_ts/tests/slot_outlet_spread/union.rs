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

/// Static attributes retain normal object-literal widening even when the same
/// name occurs repeatedly. Explicit bound expressions keep authored assertions.
#[test]
fn inferred_outlet_payload_widens_static_attributes() {
    let code = generate(
        r#"<div><slot name="side" viewMode="sp" :count="1" /><slot name="side" viewMode="pc" /></div>"#,
    );
    assert_eq!(
        (
            code.matches("\"viewMode\": \"sp\",").count(),
            code.matches("\"viewMode\": \"pc\",").count(),
            code.matches("as const").count(),
        ),
        (1, 1, 0),
        "{code}"
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
