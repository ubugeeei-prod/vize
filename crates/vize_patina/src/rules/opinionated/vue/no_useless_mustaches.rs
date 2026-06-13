//! vue/no-useless-mustaches
//!
//! Disallow a mustache interpolation whose expression is a constant string
//! literal.
//!
//! `{{ 'x' }}` interpolates a constant string; it is equivalent to the static
//! text `x` but goes through the (slightly more expensive) interpolation path
//! and reads as if it were dynamic.
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
        if !is_static_string_literal(s.content.as_str()) {
            return;
        }
        ctx.warn_with_help(
            ctx.t("vue/no-useless-mustaches.message"),
            &interpolation.loc,
            ctx.t("vue/no-useless-mustaches.help"),
        );
    }
}

/// Whether `raw` is a constant string literal (`'x'`, `"x"`, or a template
/// literal with no `${}` interpolation).
fn is_static_string_literal(raw: &str) -> bool {
    let s = raw.trim();
    let bytes = s.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    match first {
        b'\'' | b'"' => first == last && !s[1..s.len() - 1].contains(first as char),
        b'`' => last == b'`' && !s.contains("${"),
        _ => false,
    }
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
    fn allows_concatenation_of_literals() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ 'a' + 'b' }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}
