use crate::virtual_ts::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn check_tail_line(code: &str) -> &str {
    code.lines()
        .find(|line| {
            line.as_bytes()
                .starts_with(b"  type __VizeComponentCheckTail")
        })
        .expect("component prop helpers should declare a check tail")
}

fn generate_parent_usage(check_unknown_props: bool) -> vize_carton::String {
    let script = r#"import Child from './Child.vue'
"#;
    let template = r#"<Child title="ok" :gradient="false" disable />"#;
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            check_options: VirtualTsCheckOptions {
                check_unknown_props,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .code
}

#[test]
fn unknown_props_off_opens_generated_check_tail() {
    assert_eq!(
        check_tail_line(&generate_parent_usage(false)),
        "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;",
    );
}

#[test]
fn unknown_props_on_keeps_native_fallthrough_tail() {
    assert_eq!(
        check_tail_line(&generate_parent_usage(true)),
        "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & __VizeGlobalHtmlAttrs & __VizeAllowedFallthroughAttrs<C> : Record<string, unknown>;",
    );
}
