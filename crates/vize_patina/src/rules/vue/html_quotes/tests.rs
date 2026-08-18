//! `vue/html-quotes` behaviour, pinned span by span.
//!
//! Every expectation is the complete diagnostic list for the template, and each
//! entry carries the exact source text the reported range covers. The spans are
//! the ones `eslint-plugin-vue@10.9.2` produces for the same input — it reports
//! the attribute value node, delimiters included.

use super::{HtmlQuotes, HtmlQuotesOption};
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn linter_with(style: HtmlQuotesOption) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(HtmlQuotes { style }));
    Linter::with_registry(registry)
}

/// Every diagnostic as `(rule, severity, start, end, covered text)`.
fn findings(
    style: HtmlQuotesOption,
    source: &str,
) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    linter_with(style)
        .lint_template(source, "test.vue")
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                &source[diagnostic.start as usize..diagnostic.end as usize],
            )
        })
        .collect()
}

fn double(source: &str) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    findings(HtmlQuotesOption::Double, source)
}

#[test]
fn double_quoted_values_are_clean() {
    assert_eq!(double(r#"<div class="foo"></div>"#), vec![]);
}

#[test]
fn a_single_quoted_value_is_reported_with_its_delimiters() {
    let source = r#"<div class='foo'></div>"#;
    assert_eq!(
        double(source),
        vec![("vue/html-quotes", Severity::Warning, 11, 16, r#"'foo'"#)]
    );
}

#[test]
fn an_empty_single_quoted_value_is_the_two_delimiters() {
    let source = r#"<div title=''></div>"#;
    assert_eq!(
        double(source),
        vec![("vue/html-quotes", Severity::Warning, 11, 13, "''")]
    );
}

#[test]
fn an_unquoted_value_is_reported_over_the_bare_text() {
    let source = r#"<div class=bare></div>"#;
    assert_eq!(
        double(source),
        vec![("vue/html-quotes", Severity::Warning, 11, 15, "bare")]
    );
}

#[test]
fn a_component_attribute_is_checked_like_a_native_one() {
    let source = r#"<MyComponent label='hi' />"#;
    assert_eq!(
        double(source),
        vec![("vue/html-quotes", Severity::Warning, 19, 23, r#"'hi'"#)]
    );
}

#[test]
fn directive_values_are_checked() {
    let source = r#"<div v-if='ok' :class='cls' @click='go()' v-html='raw'></div>"#;
    assert_eq!(
        double(source),
        vec![
            ("vue/html-quotes", Severity::Warning, 10, 14, r#"'ok'"#),
            ("vue/html-quotes", Severity::Warning, 22, 27, r#"'cls'"#),
            ("vue/html-quotes", Severity::Warning, 35, 41, r#"'go()'"#),
            ("vue/html-quotes", Severity::Warning, 49, 54, r#"'raw'"#),
        ]
    );
}

#[test]
fn an_equals_sign_inside_a_handler_does_not_shift_the_span() {
    let source = r#"<div @click='a = b'></div>"#;
    assert_eq!(
        double(source),
        vec![("vue/html-quotes", Severity::Warning, 12, 19, r#"'a = b'"#)]
    );
}

#[test]
fn valueless_attributes_and_same_name_shorthands_are_skipped() {
    assert_eq!(double(r#"<input disabled />"#), vec![]);
    assert_eq!(double(r#"<div :foo></div>"#), vec![]);
}

#[test]
fn a_double_quoted_value_holding_single_quotes_is_clean() {
    assert_eq!(double(r#"<div title="say 'hi'"></div>"#), vec![]);
}

#[test]
fn a_directive_value_holding_the_required_quote_is_left_alone() {
    // Glyph prints directive expressions with double-quoted JS strings and then
    // must delimit the attribute with single quotes; reporting this would make
    // formatter output fail this very rule. Upstream's `avoidEscape: true`
    // agrees; its default would report and escape.
    assert_eq!(
        double(r#"<Story :layout='{ type: "grid" }'></Story>"#),
        vec![]
    );
}

#[test]
fn a_plain_attribute_holding_the_required_quote_is_still_reported() {
    // Glyph never rewrites a literal attribute value, so there is no formatter
    // conflict here and the upstream default stands.
    let source = r#"<div title='say "hi"'></div>"#;
    assert_eq!(
        double(source),
        vec![(
            "vue/html-quotes",
            Severity::Warning,
            11,
            21,
            r#"'say "hi"'"#
        )]
    );
}

#[test]
fn single_option_reports_double_quoted_values() {
    let source = r#"<div class="foo"></div>"#;
    assert_eq!(
        findings(HtmlQuotesOption::Single, source),
        vec![("vue/html-quotes", Severity::Warning, 11, 16, r#""foo""#)]
    );
}

#[test]
fn the_double_quote_fix_replaces_both_delimiters() {
    let linter = linter_with(HtmlQuotesOption::Double);
    let source = r#"<div class='foo'></div>"#;
    let result = linter.lint_template(source, "test.vue");
    let fix = result.diagnostics[0]
        .fix
        .as_ref()
        .expect("expected quote fix");
    assert_eq!(fix.message, "Use double quotes");
    assert_eq!(fix.edits.len(), 2);
    assert_eq!(fix.apply(source), r#"<div class="foo"></div>"#);
}

#[test]
fn the_unquoted_fix_encloses_the_value() {
    let linter = linter_with(HtmlQuotesOption::Double);
    let source = r#"<div class=bare></div>"#;
    let result = linter.lint_template(source, "test.vue");
    let fix = result.diagnostics[0]
        .fix
        .as_ref()
        .expect("expected quote fix");
    assert_eq!(fix.edits.len(), 1);
    assert_eq!(fix.apply(source), r#"<div class="bare"></div>"#);
}

#[test]
fn the_fix_is_omitted_when_the_value_already_holds_the_target_quote() {
    let linter = linter_with(HtmlQuotesOption::Double);
    let result = linter.lint_template(r#"<div title='say "hi"'></div>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert!(!result.diagnostics[0].has_fix());
}

#[test]
fn the_single_quote_fix_replaces_both_delimiters() {
    let linter = linter_with(HtmlQuotesOption::Single);
    let source = r#"<div class="foo"></div>"#;
    let result = linter.lint_template(source, "test.vue");
    let fix = result.diagnostics[0]
        .fix
        .as_ref()
        .expect("expected quote fix");
    assert_eq!(fix.message, "Use single quotes");
    assert_eq!(fix.edits.len(), 2);
    assert_eq!(fix.apply(source), "<div class='foo'></div>");
}

#[test]
fn the_single_quote_fix_is_omitted_when_the_value_holds_a_single_quote() {
    let linter = linter_with(HtmlQuotesOption::Single);
    let result = linter.lint_template(r#"<div title="don't"></div>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert!(!result.diagnostics[0].has_fix());
}
