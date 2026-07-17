use super::generate_virtual_ts_with_offsets;
use vize_croquis::{Analyzer, AnalyzerOptions};

const TEMPLATE: &str =
    r#"<el-badge><template #content="{ value }">{{ value.missing }}</template></el-badge>"#;

#[test]
fn kebab_case_slot_host_uses_pascal_case_setup_binding() {
    let script = r#"import { ElBadge } from 'element-plus'"#;
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
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
    assert!(
        !output.code.contains("declare const el_badge:"),
        "{}",
        output.code
    );
}

#[test]
fn kebab_case_slot_host_uses_ambient_pascal_global_component() {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let options = super::VirtualTsOptions {
        external_template_bindings: vec!["ElBadge".into()],
        ..Default::default()
    };
    let output = generate_virtual_ts_with_offsets(&summary, None, Some(&root), 0, 0, &options);

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
        !output.code.contains("declare const ElBadge:"),
        "{}",
        output.code
    );
    assert!(
        !output
            .code
            .contains("const el_badge: any = undefined as any;"),
        "{}",
        output.code,
    );
    assert!(
        !output.code.contains("declare const el_badge:"),
        "{}",
        output.code
    );
}

#[test]
fn unresolved_slot_host_uses_vue_global_components_fallback() {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output =
        generate_virtual_ts_with_offsets(&summary, None, Some(&root), 0, 0, &Default::default());

    assert!(
        output.code.contains(
            "declare const el_badge: import(\"vue\").GlobalComponents extends { \"el-badge\": infer __C } ? __C : import(\"vue\").GlobalComponents extends { \"ElBadge\": infer __C } ? __C : any;"
        ),
        "{}",
        output.code,
    );
    assert_eq!(
        output
            .code
            .matches("typeof el_badge extends { new (): { $slots: infer __S } }")
            .count(),
        1,
        "{}",
        output.code,
    );
}

#[test]
fn editor_reference_paths_are_escaped_and_deduplicated() {
    let summary = Analyzer::with_options(AnalyzerOptions::full()).finish();
    let options = super::VirtualTsOptions {
        reference_paths: vec![
            "/tmp/app&docs/components.d.ts".into(),
            "/tmp/app&docs/components.d.ts".into(),
            "invalid\npath.d.ts".into(),
        ],
        ..Default::default()
    };

    let output = generate_virtual_ts_with_offsets(&summary, None, None, 0, 0, &options);

    assert_eq!(
        output
            .code
            .matches("/// <reference path=\"/tmp/app&amp;docs/components.d.ts\" />")
            .count(),
        1,
        "{}",
        output.code,
    );
    assert!(!output.code.contains("invalid"), "{}", output.code);
}
