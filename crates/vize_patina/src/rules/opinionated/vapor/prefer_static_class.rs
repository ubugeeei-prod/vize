//! vapor/prefer-static-class
//!
//! Prefer static class over dynamic class when possible.
//!
//! In Vapor mode, static classes can be included directly in the template
//! string, avoiding runtime class manipulation. Dynamic classes require
//! additional runtime processing.
//!
//! ## Examples
//!
//! ### Invalid (can be optimized)
//! ```vue
//! <div :class="'static-class'"></div>
//! <div :class="`always-same`"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div class="static-class"></div>
//! <div :class="dynamicClass"></div>
//! <div :class="{ active: isActive }"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, Severity, TextEdit};
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vapor/prefer-static-class",
    description: "Prefer static class over dynamic class binding for string literals",
    category: RuleCategory::Vapor,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Prefer static class in Vapor mode
pub struct PreferStaticClass;

/// Markup-IR entry point for `vapor/prefer-static-class`.
///
/// A binding-shaped Vapor rule that maps cleanly across backends: a Vue
/// `:class="'a'"` and a JSX `class={'a'}` both project to a
/// [`MarkupBindingKind::Bind`] whose argument is `class` and whose
/// [`MarkupBinding::expression`] is the string literal `'a'`. The rule warns
/// when that literal could be a plain static `class` instead. (The auto-fix
/// stays on the legacy [`Rule`] path; the IR entry point reports through
/// `ByteRange`s that map to the original syntax.)
impl MarkupRule for PreferStaticClass {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if binding.kind() != MarkupBindingKind::Bind || !binding.arg_name_eq("class") {
            return;
        }
        let Some(expression) = binding.expression() else {
            return;
        };
        if !is_string_literal(expression.trim()) {
            return;
        }
        // If a static `class` attribute is already present, the dynamic one is
        // redundant but cannot simply be folded in; just flag it.
        let mut has_static_class = false;
        element.walk_bindings(&mut |other| {
            if other.kind() == MarkupBindingKind::Attribute && other.arg_name_eq("class") {
                has_static_class = true;
            }
        });

        let message = ctx.lint().t("vapor/prefer-static-class.message");
        if has_static_class {
            let help = ctx.lint().t("vapor/prefer-static-class.help");
            ctx.lint().warn_at_with_help(message, binding.range(), help);
        } else {
            ctx.lint().warn_at(message, binding.range());
        }
    }
}

impl Rule for PreferStaticClass {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Check if this is :class or v-bind:class
        if directive.name.as_str() != "bind" {
            return;
        }

        let arg = match &directive.arg {
            Some(ExpressionNode::Simple(s)) if s.content.as_str() == "class" => s,
            _ => return,
        };

        // Check if the expression is a string literal
        let Some(ref exp) = directive.exp else {
            return;
        };

        let exp_content = match exp {
            ExpressionNode::Simple(s) => s.content.as_str(),
            _ => return,
        };

        // Check if it's a simple string literal like "'foo'" or "`foo`" or "\"foo\""
        let trimmed = exp_content.trim();
        if is_string_literal(trimmed) {
            // Extract the string value
            let inner = &trimmed[1..trimmed.len() - 1];

            // Check if element already has a static class attribute
            let has_static_class = element.props.iter().any(|p| {
                matches!(p, PropNode::Attribute(attr) if attr.name.as_str().eq_ignore_ascii_case("class"))
            });

            let message = ctx.t("vapor/prefer-static-class.message");

            // Create fix: replace :class="'value'" with class="value"
            if !has_static_class {
                let mut replacement = String::from("class=\"");
                replacement.push_str(inner);
                replacement.push('"');
                let fix = Fix::new(
                    "Replace with static class attribute",
                    TextEdit::replace(
                        directive.loc.start.offset,
                        directive.loc.end.offset + 1, // Include closing quote
                        replacement,
                    ),
                );

                ctx.report(
                    crate::diagnostic::LintDiagnostic::warn(
                        META.name,
                        message.as_ref(),
                        arg.loc.start.offset,
                        directive.loc.end.offset,
                    )
                    .with_fix(fix),
                );
            } else {
                ctx.warn_with_help(
                    message,
                    &directive.loc,
                    ctx.t("vapor/prefer-static-class.help"),
                );
            }
        }
    }
}

/// Check if a string is a simple string literal
fn is_string_literal(s: &str) -> bool {
    if s.len() < 2 {
        return false;
    }

    let bytes = s.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];

    // Check for 'string', "string", or `string`
    // But not template literals with expressions like `${foo}`
    match (first, last) {
        (b'\'', b'\'') | (b'"', b'"') => true,
        (b'`', b'`') => !s.contains("${"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::PreferStaticClass;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PreferStaticClass));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_string_literal_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="'static-class'"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_static_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="static-class"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="dynamicClass"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_object_class() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div :class="{ active: isActive }"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_template_literal_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="`prefix-${suffix}`"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
