//! Issue #7911: retain authored assertions and complete legacy filter findings.

use vize_l0::config::VueVersion;
use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale};

const RULE: &str = "vue/no-deprecated-filter";
const MESSAGE: &str =
    "Filters were removed in Vue 3; replace the '|' filter with a method call or computed property";
const HELP: &str = "Replace the filter with a method call or computed property (e.g. {{ capitalize(message) }} instead of {{ message | capitalize }}).";

fn linter(version: Option<VueVersion>) -> Linter {
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
        .with_vue_version(version)
}

fn assert_result(actual: LintResult, filename: &str, diagnostics: Vec<LintDiagnostic>) {
    let expected = LintResult {
        filename: filename.into(),
        error_count: diagnostics.len(),
        warning_count: 0,
        diagnostics,
    };
    assert_eq!(format!("{actual:#?}"), format!("{expected:#?}"));
}

#[test]
fn original_reporter_full_sfc_has_no_filter_findings() {
    let source = include_str!("fixtures/issue-7911/TextField.vue.fixture");
    assert_result(
        linter(None).lint_sfc(source, "TextField.vue"),
        "TextField.vue",
        vec![],
    );
}

#[test]
fn outer_runtime_filter_keeps_complete_authored_range_and_finding() {
    for expression in [
        "(value as A | B) | format",
        "(value satisfies A | B) | format",
        "(item: A | B) => (item as A | B) | format",
    ] {
        let source = format!("<!-- 雪😀 -->\r\n<div :id=\"{expression}\" />\r\n");
        let start = source.find(expression).unwrap() as u32;
        let end = start + expression.len() as u32;
        let finding = LintDiagnostic::error(RULE, MESSAGE, start, end).with_help(HELP);
        assert_result(
            linter(None).lint_template(&source, "Filter.vue"),
            "Filter.vue",
            vec![finding.clone()],
        );
        // The legacy raw-template constructor still routes VueDialect::Vue.
        // Preserve its actual complete finding; it does not select Vue 2 dialect.
        assert_result(
            linter(Some(VueVersion::V2)).lint_template(&source, "Filter.vue"),
            "Filter.vue",
            vec![finding],
        );
    }
}

#[test]
fn union_types_in_interpolations_and_nested_assertions_are_clean() {
    for expression in [
        "props.id as string | undefined",
        "value satisfies { kind: A | B; handler: () => A | B }",
        "String((value as Array<A | B>)[0])",
    ] {
        let source = format!("<div>{{{{ {expression} }}}}</div>");
        assert_result(
            linter(None).lint_template(&source, "Union.vue"),
            "Union.vue",
            vec![],
        );
    }
}
