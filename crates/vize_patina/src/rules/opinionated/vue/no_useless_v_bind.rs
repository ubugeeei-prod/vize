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
use crate::markup::{
    MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

use super::no_useless_string_literal::static_string_literal_inner;

static META: RuleMeta = RuleMeta {
    name: "vue/no-useless-v-bind",
    description: "Disallow a v-bind whose value is a plain string literal",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow a v-bind whose value is a plain string literal.
pub struct NoUselessVBind;

impl MarkupRule for NoUselessVBind {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Bindings
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if !ctx.is_template() || binding.kind() != MarkupBindingKind::Bind {
            return;
        }
        let Some(name) = binding.arg_name() else {
            return;
        };
        if !binding.is_static_unqualified_arg_exact(name) {
            return;
        }
        let mut has_modifier = false;
        binding.walk_modifiers(&mut |_| has_modifier = true);
        if has_modifier || !binding.expression().is_some_and(is_static_string_literal) {
            return;
        }

        let message = ctx
            .lint()
            .t_fmt("vue/no-useless-v-bind.message", &[("name", name)]);
        let help = ctx.lint().t("vue/no-useless-v-bind.help");
        ctx.lint().warn_at_with_help(message, binding.range(), help);
    }
}

impl Rule for NoUselessVBind {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name != "bind" {
            return;
        }
        // Only a static argument (`:foo`), not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        if !arg.is_static {
            return;
        }
        // Modifiers such as `.prop` change semantics; leave them alone.
        if !directive.modifiers.is_empty() {
            return;
        }
        let is_literal = matches!(
            &directive.exp,
            Some(ExpressionNode::Simple(s)) if is_static_string_literal(s.content)
        );
        if is_literal {
            ctx.warn_with_help(
                ctx.t_fmt("vue/no-useless-v-bind.message", &[("name", arg.content)]),
                &directive.loc,
                ctx.t("vue/no-useless-v-bind.help"),
            );
        }
    }
}

/// Whether `raw` is a constant string literal (`'x'`, `"x"`, or a template
/// literal with no `${}` interpolation).
fn is_static_string_literal(raw: &str) -> bool {
    static_string_literal_inner(raw).is_some()
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
    fn allows_dynamic_argument() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :[attr]="'bar'" v-bind:[other]="'baz'"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_template_with_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :foo="`pre-${bar}`"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn template_literals_are_delimiter_and_escape_aware() {
        let linter = create_linter();
        for source in [
            r#"<div :foo="`a` + `b`" />"#,
            r#"<div :foo="`pre-${value}`" />"#,
        ] {
            assert_eq!(linter.lint_template(source, "App.vue").warning_count, 0);
        }
        assert_eq!(
            linter
                .lint_template(r#"<div :foo="`pre-\${value}`" />"#, "App.vue")
                .warning_count,
            1
        );
    }

    #[test]
    fn reports_literals_with_escaped_quotes_but_not_unterminated_literals() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<input :placeholder="'Don\'t panic'" :title='"Say \"hi\""' />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 2);

        let unterminated = linter.lint_template(r#"<div :id="'\'" />"#, "App.vue");
        assert_eq!(unterminated.warning_count, 0);
    }
}
