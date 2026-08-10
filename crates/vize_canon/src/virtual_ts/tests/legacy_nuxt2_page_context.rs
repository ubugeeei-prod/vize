use super::{
    VirtualTsGenerationOptions, VirtualTsOptions, generate_virtual_ts,
    generate_virtual_ts_with_offsets_and_checks, generate_virtual_ts_with_offsets_legacy_vue2,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn test_legacy_nuxt2_page_validate_gets_contextual_type() {
    let script = r#"export default {
  name: 'StudentPage',
  validate({ params }) {
    return !Number.isNaN(Number(params.studentId))
  }
}
"#;
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_plain(script);
    let summary = analyzer.finish();

    let output = generate_virtual_ts_with_offsets_and_checks(
        &summary,
        Some(script),
        None,
        0,
        0,
        &Default::default(),
        VirtualTsGenerationOptions {
            legacy_vue2: true,
            ..Default::default()
        },
    );

    assert!(
        output
            .code
            .contains("validate?: (context: __VizeNuxt2Context) => unknown;"),
        "legacy Nuxt 2 page options should declare a contextual validate type:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("type __VizeNuxt2PageOptions = ThisType<any> & {")
            && output.code.contains("layout?: any;"),
        "legacy Nuxt 2 page options should accept page-only options without narrowing `this`:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("const __default__ = __vizeDefineComponent({"),
        "plain object exports should be wrapped by the legacy helper:\n{}",
        output.code
    );
}

#[test]
fn modern_public_constructor_contract_does_not_leak_into_vue2() {
    let script = "defineProps<{ someValue: string }>()\n";
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let summary = analyzer.finish();

    let modern = generate_virtual_ts(&summary, Some(script), None, 0).code;
    let legacy = generate_virtual_ts_with_offsets_legacy_vue2(
        &summary,
        Some(script),
        None,
        0,
        0,
        &VirtualTsOptions::default(),
    )
    .code;

    assert!(
        modern.contains("$props: Props;")
            && modern.contains("new <__VizeAuthoredProps = unknown>(props?:")
            && modern.contains("} & __VizeComponentPublicBase;"),
        "Vue 3 should separate strict public props from call-site inputs:\n{modern}"
    );
    assert!(
        !modern.contains("__VizeVue2ComponentInstance"),
        "Vue 2 instance members must not leak into Vue 3:\n{modern}"
    );
    assert!(
        legacy.contains("$props: __VizeComponentProps<Props>;")
            && legacy.contains(
                "type __VizeComponentConstructor = new (...args: any[]) => __VizeComponentInstance;"
            )
            && legacy.contains("} & __VizeVue2ComponentInstance;"),
        "Vue 2 should keep its existing permissive constructor contract:\n{legacy}"
    );
    assert!(
        !legacy.contains("new <__VizeAuthoredProps"),
        "the Vue 3 authored-input constructor must stay outside Vue 2:\n{legacy}"
    );
}
