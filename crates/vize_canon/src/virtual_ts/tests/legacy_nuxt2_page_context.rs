use super::{VirtualTsGenerationOptions, generate_virtual_ts_with_offsets_and_checks};

#[test]
fn test_legacy_nuxt2_page_validate_gets_contextual_type() {
    use vize_croquis::{Analyzer, AnalyzerOptions};

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
