//! `vue/mustache-interpolation-spacing` behaviour, pinned span by span.
//!
//! Each expectation is the complete diagnostic list, and every entry carries the
//! source text its range covers. The spans match the ones
//! `eslint-plugin-vue@10.9.2` reports for the same input: one finding per
//! offending delimiter, on that delimiter.

use super::{MustacheInterpolationSpacing, SpacingStyle};
use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn findings(style: SpacingStyle, source: &str) -> Vec<(Severity, u32, u32, &str, String)> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(MustacheInterpolationSpacing { style }));
    Linter::with_registry(registry)
        .lint_template(source, "test.vue")
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                &source[diagnostic.start as usize..diagnostic.end as usize],
                diagnostic.message.to_string(),
            )
        })
        .collect()
}

fn always(source: &str) -> Vec<(Severity, u32, u32, &str, String)> {
    findings(SpacingStyle::Always, source)
}

#[test]
fn a_spaced_interpolation_is_clean() {
    assert_eq!(always(r#"<div>{{ text }}</div>"#), vec![]);
}

#[test]
fn both_delimiters_are_reported_separately() {
    assert_eq!(
        always(r#"<div>{{text}}</div>"#),
        vec![
            (
                Severity::Warning,
                5,
                7,
                "{{",
                "Expected 1 space after '{{', but not found".to_string(),
            ),
            (
                Severity::Warning,
                11,
                13,
                "}}",
                "Expected 1 space before '}}', but not found".to_string(),
            ),
        ]
    );
}

#[test]
fn only_the_missing_side_is_reported() {
    assert_eq!(
        always(r#"<div>{{text }}</div>"#),
        vec![(
            Severity::Warning,
            5,
            7,
            "{{",
            "Expected 1 space after '{{', but not found".to_string(),
        )]
    );
    assert_eq!(
        always(r#"<div>{{ text}}</div>"#),
        vec![(
            Severity::Warning,
            12,
            14,
            "}}",
            "Expected 1 space before '}}', but not found".to_string(),
        )]
    );
}

#[test]
fn a_newline_counts_as_spacing() {
    assert_eq!(always("<div>{{\n  text\n}}</div>"), vec![]);
}

#[test]
fn a_tab_counts_as_spacing() {
    assert_eq!(always("<div>{{\ttext\t}}</div>"), vec![]);
}

#[test]
fn an_interpolation_without_an_expression_is_skipped() {
    assert_eq!(always(r#"<div>{{}}</div>"#), vec![]);
    assert_eq!(always(r#"<div>{{   }}</div>"#), vec![]);
}

#[test]
fn never_reports_the_delimiter_together_with_its_whitespace() {
    assert_eq!(
        findings(SpacingStyle::Never, r#"<div>{{ text }}</div>"#),
        vec![
            (
                Severity::Warning,
                5,
                8,
                "{{ ",
                "Expected no space after '{{', but found".to_string(),
            ),
            (
                Severity::Warning,
                12,
                15,
                " }}",
                "Expected no space before '}}', but found".to_string(),
            ),
        ]
    );
}

#[test]
fn never_is_clean_on_a_tight_interpolation() {
    assert_eq!(
        findings(SpacingStyle::Never, r#"<div>{{text}}</div>"#),
        vec![]
    );
}
