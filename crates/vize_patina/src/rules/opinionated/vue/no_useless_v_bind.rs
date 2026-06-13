//! vue/no-useless-v-bind
//!
//! Disallow a `v-bind` whose value is a plain string literal.
//!
//! `:foo="'bar'"` binds a constant string; it is equivalent to the static
//! attribute `foo="bar"` but goes through the (slightly more expensive) binding
//! path and reads as if it were dynamic.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div :foo="'bar'"></div>
//! <div :foo="`bar`"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div foo="bar"></div>
//! <div :foo="bar"></div>
//! <div :foo="`pre-${bar}`"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-useless-v-bind",
    description: "Disallow a v-bind whose value is a plain string literal",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow a v-bind whose value is a plain string literal.
pub struct NoUselessVBind;

impl Rule for NoUselessVBind {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "bind" {
            return;
        }
        // Only a static argument (`:foo`), not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        // Modifiers such as `.prop` change semantics; leave them alone.
        if !directive.modifiers.is_empty() {
            return;
        }
        let is_literal = matches!(
            &directive.exp,
            Some(ExpressionNode::Simple(s)) if is_static_string_literal(s.content.as_str())
        );
        if is_literal {
            ctx.warn_with_help(
                ctx.t_fmt(
                    "vue/no-useless-v-bind.message",
                    &[("name", arg.content.as_str())],
                ),
                &directive.loc,
                ctx.t("vue/no-useless-v-bind.help"),
            );
        }
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
    use super::NoUselessVBind;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUselessVBind));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_string_literal() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo="'bar'"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_template_literal_without_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo="`bar`"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_dynamic_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo="bar"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_template_with_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo="`pre-${bar}`"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}
