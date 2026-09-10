use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn strict_template_context_does_not_recheck_script_setup_enum() {
    let template = r#"<div>{{ status === Status.Ready }}</div>"#;
    let script = "enum Status { Idle, Ready }\nconst status = Status.Idle\n";

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);

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
            strict_instance_globals: true,
            ..Default::default()
        },
    );

    assert!(output.code.contains("Status.Ready"), "{}", output.code);
    assert!(
        !output
            .code
            .contains("__vize_strict_template_context.Status"),
        "{}",
        output.code
    );
}
