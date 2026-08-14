//! vue/html-quotes
//!
//! Enforce consistent use of quotes in HTML attributes.
//!
//! ## Options
//!
//! - `"double"` (default): Require double quotes
//! - `"single"`: Require single quotes
//!
//! ## Examples
//!
//! ### Invalid (with double option)
//! ```vue
//! <div class='foo'></div>
//! ```
//!
//! ### Valid (with double option)
//! ```vue
//! <div class="foo"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/html-quotes",
    description: "Enforce quotes style of HTML attributes",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Quote style preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HtmlQuotesOption {
    #[default]
    Double,
    Single,
}

/// Enforce HTML attribute quote style
pub struct HtmlQuotes {
    pub style: HtmlQuotesOption,
}

impl Default for HtmlQuotes {
    fn default() -> Self {
        Self {
            style: HtmlQuotesOption::Double,
        }
    }
}

impl Rule for HtmlQuotes {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        for prop in &element.props {
            if let PropNode::Attribute(attr) = prop
                && let Some(value) = &attr.value
            {
                // Use the value span from the attribute location to check quote style:
                // the attribute span covers the full attribute including quotes, so the
                // byte just before the value is the opening quote character (if any).
                let start = value.loc.start.offset as usize;
                if start == 0 || start > ctx.source.len() {
                    continue;
                }

                // Check the character just before the value content (the opening quote)
                let quote_char = ctx.source.as_bytes().get(start.wrapping_sub(1)).copied();

                match self.style {
                    HtmlQuotesOption::Double => {
                        if quote_char == Some(b'\'') {
                            let message = ctx.t("vue/html-quotes.message_double");
                            let help = ctx.t("vue/html-quotes.help");
                            let mut diagnostic = LintDiagnostic::warn(
                                META.name,
                                message,
                                value.loc.start.offset,
                                value.loc.end.offset,
                            )
                            .with_help(help);
                            if let Some(fix) = quote_fix(
                                ctx.source,
                                value.loc.start.offset,
                                value.loc.end.offset,
                                b'"',
                                "Use double quotes",
                            ) {
                                diagnostic = diagnostic.with_fix(fix);
                            }
                            ctx.report(diagnostic);
                        }
                    }
                    HtmlQuotesOption::Single => {
                        if quote_char == Some(b'"') {
                            let message = ctx.t("vue/html-quotes.message_single");
                            let help = ctx.t("vue/html-quotes.help");
                            let mut diagnostic = LintDiagnostic::warn(
                                META.name,
                                message,
                                value.loc.start.offset,
                                value.loc.end.offset,
                            )
                            .with_help(help);
                            if let Some(fix) = quote_fix(
                                ctx.source,
                                value.loc.start.offset,
                                value.loc.end.offset,
                                b'\'',
                                "Use single quotes",
                            ) {
                                diagnostic = diagnostic.with_fix(fix);
                            }
                            ctx.report(diagnostic);
                        }
                    }
                }
            }
        }
    }
}

fn quote_fix(
    source: &str,
    start: u32,
    end: u32,
    expected_quote: u8,
    message: &'static str,
) -> Option<Fix> {
    let opening = start.checked_sub(1)?;
    let closing_end = end.checked_add(1)?;
    let start_index = usize::try_from(start).ok()?;
    let end_index = usize::try_from(end).ok()?;
    let opening_index = usize::try_from(opening).ok()?;
    let actual_quote = *source.as_bytes().get(opening_index)?;
    if !matches!(actual_quote, b'\'' | b'"')
        || source.as_bytes().get(end_index).copied() != Some(actual_quote)
        || source
            .as_bytes()
            .get(start_index..end_index)?
            .contains(&expected_quote)
    {
        return None;
    }

    let replacement = if expected_quote == b'"' { "\"" } else { "'" };
    Some(Fix::with_edits(
        message,
        vec![
            TextEdit::replace(opening, start, replacement),
            TextEdit::replace(end, closing_end, replacement),
        ],
    ))
}

#[cfg(test)]
mod tests {
    use super::{HtmlQuotes, HtmlQuotesOption};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlQuotes::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_double_quotes() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_double_quotes_fix_replaces_both_delimiters() {
        let linter = create_linter();
        let source = r#"<div class='foo'></div>"#;
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1);
        let fix = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("expected quote fix");
        assert_eq!(fix.message, "Use double quotes");
        assert_eq!(fix.edits.len(), 2);
        assert_eq!(fix.apply(source), r#"<div class="foo"></div>"#);
    }

    #[test]
    fn test_double_quotes_fix_is_omitted_when_value_contains_double_quote() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div title='say "hi"'></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(!result.diagnostics[0].has_fix());
    }

    #[test]
    fn test_valid_no_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input disabled />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_single_option_warns_on_double_quotes() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlQuotes {
            style: HtmlQuotesOption::Single,
        }));
        let linter = Linter::with_registry(registry);
        // Parser preserves double quotes, so single-quote preference should warn
        let result = linter.lint_template(r#"<div class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_single_quotes_fix_replaces_both_delimiters() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlQuotes {
            style: HtmlQuotesOption::Single,
        }));
        let linter = Linter::with_registry(registry);
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
    fn test_single_quotes_fix_is_omitted_when_value_contains_single_quote() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlQuotes {
            style: HtmlQuotesOption::Single,
        }));
        let linter = Linter::with_registry(registry);
        let result = linter.lint_template(r#"<div title="don't"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(!result.diagnostics[0].has_fix());
    }
}
