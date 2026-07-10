use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn experimental_self_reference_uses_internal_component_ref() {
    let script = "const Self = {}\ndefineProps<{ label: string }>()\n";
    let template = r#"<Self :label="123" />"#;
    let allocator = vize_carton::Bump::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions {
            experimental_self_reference: true,
            ..Default::default()
        },
    );

    assert!(output.code.contains("const __VizeSelf:"), "{}", output.code);
    assert!(
        output
            .code
            .contains("type __Self_Props_0 = typeof __VizeSelf"),
        "{}",
        output.code
    );
    assert!(!output.code.contains("type __Self_Props_0 = typeof Self"));
}
