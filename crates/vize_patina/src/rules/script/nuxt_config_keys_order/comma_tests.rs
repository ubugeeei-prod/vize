use super::tests::{fix_until_stable, lint};
use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use serde_json::Value;
use vize_l0::cstr;

const CASES: &str = include_str!("../../../../tests/fixtures/issue-7963/cases.json");

#[test]
fn preserves_authored_comma_style_comments_and_complete_fixed_point() {
    let cases: Value = serde_json::from_str(CASES).unwrap();
    for case in cases.as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let expected = case["fixed"].as_str().unwrap();
        let fixed = fix_until_stable(source);
        assert_eq!(fixed, expected, "{id}");
        let result = lint(&fixed);
        assert_eq!((result.error_count, result.warning_count), (0, 0), "{id}");
        assert!(result.diagnostics.is_empty(), "{id}: {result:#?}");
        assert_eq!(fix_until_stable(&fixed), expected, "{id}");
    }
}

#[test]
fn original_report_keeps_whole_diagnostic_and_edit_contract() {
    let source = include_str!("../../../../tests/fixtures/issue-7963/nuxt.config.ts.fixture");
    let result = lint(source);
    assert_eq!((result.error_count, result.warning_count), (1, 0));
    let start = source.find("ssr: false").unwrap() as u32;
    let range_start = source.find("  ssr:").unwrap() as u32;
    let range_end = source.find("\n});").unwrap() as u32 + 1;
    let expected = LintDiagnostic::error(
        "nuxt/nuxt-config-keys-order",
        "Expected config key \"ssr\" to come after \"modules\"",
        start,
        start + 10,
    )
    .with_fix(Fix::new(
        "Sort Nuxt config keys",
        TextEdit::replace(
            range_start,
            range_end,
            "  modules: [\"@pinia/nuxt\"],\n  ssr: false\n",
        ),
    ));
    assert_eq!(cstr!("{:?}", result.diagnostics), cstr!("{:?}", [expected]));
}

#[test]
fn preserves_each_environment_objects_independent_final_comma_style() {
    let source = "export default { ssr: false, $test: { ssr: false, modules: [], }, $production: { ssr: false, modules: [] }, modules: [], }";
    let expected = "export default { modules: [], $production: { modules: [], ssr: false }, $test: { modules: [], ssr: false, }, ssr: false, }";
    assert_eq!(fix_until_stable(source), expected);
    assert!(lint(expected).diagnostics.is_empty());
}
