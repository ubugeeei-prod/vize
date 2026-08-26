//! vue/no-useless-mustaches
//!
//! Disallow a mustache interpolation whose expression is a constant string
//! literal.
//!
//! `{{ 'x' }}` interpolates a constant string; it is equivalent to the static
//! text `x` but goes through the (slightly more expensive) interpolation path
//! and reads as if it were dynamic.
//!
//! Whitespace-only literals such as `{{ " " }}` are exempt: they are an idiom
//! to force a text node, and the literal replacement would be a
//! whitespace-only text node that the default whitespace handling (condense)
//! drops — changing the rendered output.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div>{{ 'x' }}</div>
//! <div>{{ "x" }}</div>
//! <div>{{ `x` }}</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>x</div>
//! <div>{{ x }}</div>
//! <div>{{ `pre-${x}` }}</div>
//! <span>A</span> {{ " " }} <span>B</span>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-useless-mustaches",
    description: "Disallow a mustache interpolation whose expression is a constant string literal",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow a mustache interpolation whose expression is a constant string
/// literal.
pub struct NoUselessMustaches;

impl Rule for NoUselessMustaches {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        // Only a single simple expression can be a bare string literal; a
        // compound expression always mixes in something dynamic.
        let ExpressionNode::Simple(s) = &interpolation.content else {
            return;
        };
        let Some(inner) = static_string_literal_inner(s.content) else {
            return;
        };
        // `{{ " " }}` forces a text node; its static replacement would be
        // dropped by whitespace condense, changing rendered output (#4954).
        if is_whitespace_only(inner) {
            return;
        }
        ctx.warn_with_help(
            ctx.t("vue/no-useless-mustaches.message"),
            &interpolation.loc,
            ctx.t("vue/no-useless-mustaches.help"),
        );
    }
}

/// When `raw` is a constant string literal (`'x'`, `"x"`, or a template
/// literal with no `${}` interpolation), return its inner content (between
/// the quotes, escape sequences unresolved).
fn static_string_literal_inner(raw: &str) -> Option<&str> {
    let s = raw.trim();
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    let is_literal = match first {
        b'\'' | b'"' => first == last && !s[1..s.len() - 1].contains(first as char),
        b'`' => last == b'`' && !s.contains("${"),
        _ => false,
    };
    is_literal.then(|| &s[1..s.len() - 1])
}

/// Whether the (non-empty) literal content renders as whitespace only,
/// counting both literal whitespace and unresolved whitespace escape
/// sequences (`\t`, `\n`, ...).
fn is_whitespace_only(inner: &str) -> bool {
    if inner.is_empty() {
        return false;
    }
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if !matches!(chars.next(), Some('t' | 'n' | 'r' | 'f' | 'v' | ' ')) {
                return false;
            }
        } else if !c.is_whitespace() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::NoUselessMustaches;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUselessMustaches));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_single_quoted_literal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ 'x' }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_double_quoted_literal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ "x" }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_template_literal_without_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ `x` }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_whitespace_only_literal() {
        // #4954: `{{ " " }}` is an idiom to force a text node. The literal
        // replacement is a whitespace-only text node that the default
        // whitespace handling (condense) drops, changing rendered output.
        let linter = create_linter();
        let result = linter.lint_template(
            "<div>\n  <span>A</span>\n  {{ \" \" }}\n  <span>B</span>\n</div>",
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_whitespace_escape_literal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ "\t" }}{{ '\n' }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_literal_with_visible_content() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ ' x ' }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_dynamic_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ x }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_template_with_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ `pre-${x}` }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_whitespace_only_literal_text_node() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<span>A</span>
{{ " " }}
<span>B</span>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_concatenation_of_literals() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ 'a' + 'b' }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}
