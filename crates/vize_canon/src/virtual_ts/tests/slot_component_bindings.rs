use super::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn kebab_case_slot_host_uses_pascal_case_setup_binding() {
    let script = r#"import { ElBadge } from 'element-plus'"#;
    let template =
        r#"<el-badge><template #content="{ value }">{{ value.missing }}</template></el-badge>"#;

    let allocator = vize_carton::Bump::new();
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
        &Default::default(),
    );

    assert_eq!(
        output
            .code
            .matches("typeof ElBadge extends { new (): { $slots: infer __S } }")
            .count(),
        1,
        "{}",
        output.code,
    );
    assert!(
        !output
            .code
            .contains("typeof el_badge extends { new (): { $slots: infer __S } }"),
        "{}",
        output.code,
    );
}
