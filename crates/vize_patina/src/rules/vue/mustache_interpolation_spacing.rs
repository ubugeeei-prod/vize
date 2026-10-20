//! vue/mustache-interpolation-spacing
//!
//! Enforce consistent spacing inside mustache interpolations.
//!
//! ## Reported spans
//!
//! Upstream checks the two delimiters independently and reports each on its own
//! delimiter token, so `{{text}}` is two findings — one on `{{`, one on `}}` —
//! not one finding spanning the interpolation. Under `never` the reported span
//! also covers the offending whitespace, because that is the text the fix
//! removes.
//!
//! ## Examples
//!
//! ### Invalid (default: always)
//! ```vue
//! <div>{{text}}</div>
//! <div>{{ text}}</div>
//! <div>{{text }}</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>{{ text }}</div>
//! <div>{{ foo.bar }}</div>
//! <div>{{ foo + bar }}</div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::InterpolationNode;

static META: RuleMeta = RuleMeta {
    name: "vue/mustache-interpolation-spacing",
    description: "Enforce consistent spacing inside mustache interpolations",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Spacing style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpacingStyle {
    /// Require spaces: {{ foo }}
    #[default]
    Always,
    /// No spaces: {{foo}}
    Never,
}

/// Mustache interpolation spacing rule
pub struct MustacheInterpolationSpacing {
    pub style: SpacingStyle,
}

impl Default for MustacheInterpolationSpacing {
    fn default() -> Self {
        Self {
            style: SpacingStyle::Always,
        }
    }
}

impl Rule for MustacheInterpolationSpacing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        // Note: end.offset is exclusive (points to the character AFTER the last one)
        let start = interpolation.loc.span.start as usize;
        let end = interpolation.loc.span.end as usize;
        let Some(raw) = ctx.source.get(start..end) else {
            return;
        };
        if raw.len() < 4 {
            return;
        }
        let Some(inner) = raw
            .strip_prefix("{{")
            .and_then(|rest| rest.strip_suffix("}}"))
        else {
            return;
        };
        // An interpolation holding only whitespace has no expression, and
        // upstream's `VExpressionContainer[expression!=null]` selector skips it.
        if inner.trim().is_empty() {
            return;
        }

        let opening = interpolation.loc.span.start;
        let closing = interpolation.loc.span.end;
        let leading = leading_whitespace(inner);
        let trailing = trailing_whitespace(inner);
        match self.style {
            SpacingStyle::Always => {
                if leading == 0 {
                    report(
                        ctx,
                        EXPECTED_AFTER,
                        HELP_EXPECTED,
                        opening,
                        opening + 2,
                        TextEdit::insert(opening + 2, " "),
                    );
                }
                if trailing == 0 {
                    report(
                        ctx,
                        EXPECTED_BEFORE,
                        HELP_EXPECTED,
                        closing - 2,
                        closing,
                        TextEdit::insert(closing - 2, " "),
                    );
                }
            }
            SpacingStyle::Never => {
                if leading > 0 {
                    let end = opening + 2 + leading;
                    report(
                        ctx,
                        UNEXPECTED_AFTER,
                        HELP_UNEXPECTED,
                        opening,
                        end,
                        TextEdit::delete(opening + 2, end),
                    );
                }
                if trailing > 0 {
                    let start = closing - 2 - trailing;
                    report(
                        ctx,
                        UNEXPECTED_BEFORE,
                        HELP_UNEXPECTED,
                        start,
                        closing,
                        TextEdit::delete(start, closing - 2),
                    );
                }
            }
        }
    }
}

const EXPECTED_AFTER: &str = "vue/mustache-interpolation-spacing.expected_after";
const EXPECTED_BEFORE: &str = "vue/mustache-interpolation-spacing.expected_before";
const UNEXPECTED_AFTER: &str = "vue/mustache-interpolation-spacing.unexpected_after";
const UNEXPECTED_BEFORE: &str = "vue/mustache-interpolation-spacing.unexpected_before";
const HELP_EXPECTED: &str = "vue/mustache-interpolation-spacing.help_expected";
const HELP_UNEXPECTED: &str = "vue/mustache-interpolation-spacing.help_unexpected";

fn report(
    ctx: &mut LintContext<'_>,
    message: &str,
    help: &str,
    start: u32,
    end: u32,
    edit: TextEdit,
) {
    let message = ctx.t(message).into_owned();
    let help = ctx.t(help).into_owned();
    ctx.report(
        LintDiagnostic::warn(META.name, message, start, end)
            .with_help(help.clone())
            .with_fix(Fix::new(help, edit)),
    );
}

fn leading_whitespace(inner: &str) -> u32 {
    let trimmed = inner.trim_start();
    u32::try_from(inner.len() - trimmed.len()).unwrap_or(0)
}

fn trailing_whitespace(inner: &str) -> u32 {
    let trimmed = inner.trim_end();
    u32::try_from(inner.len() - trimmed.len()).unwrap_or(0)
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
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

    #[test]
    fn reports_safe_spacing_edits() {
        let source = r#"<div>{{title}}</div>"#;
        let mut fixed = source.to_owned();
        let result = {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(MustacheInterpolationSpacing::default()));
            Linter::with_registry(registry).lint_template(source, "test.vue")
        };
        let mut edits: Vec<_> = result
            .diagnostics
            .iter()
            .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
            .collect();
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
        for edit in edits {
            fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        }
        assert_eq!(fixed, r#"<div>{{ title }}</div>"#);
        assert!(always(&fixed).is_empty());
    }

    #[test]
    fn never_reports_safe_whitespace_deletions() {
        let source = r#"<div>{{ title }}</div>"#;
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MustacheInterpolationSpacing {
            style: SpacingStyle::Never,
        }));
        let result = Linter::with_registry(registry).lint_template(source, "test.vue");
        let mut fixed = source.to_owned();
        let mut edits: Vec<_> = result
            .diagnostics
            .iter()
            .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
            .collect();
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));
        for edit in edits {
            fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        }
        assert_eq!(fixed, r#"<div>{{title}}</div>"#);
    }
}
