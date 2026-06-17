use crate::virtual_ts::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_preserve_unused_diagnostics_marks_static_template_refs_used() {
    let script = r#"const activatorRef = null
const menuRef = null
const decoy = null
"#;
    let template = r#"<div ref="activatorRef"><div ref="menuRef" /></div>"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            preserve_unused_diagnostics: true,
            ..Default::default()
        },
    );

    assert!(output.code.contains("void activatorRef; void menuRef;"));
    assert!(!output.code.contains("void decoy;"));
}
