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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vapor/prefer-static-class",
    description: "Prefer static class over dynamic class binding for string literals",
    category: RuleCategory::Vapor,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Prefer static class in Vapor mode
pub struct PreferStaticClass;

impl Rule for PreferStaticClass {
    fn meta(&self) -> &'static RuleMeta {
        &META
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

    let first = s.chars().next().unwrap();
    let last = s.chars().last().unwrap();

    // Check for 'string', "string", or `string`
    // But not template literals with expressions like `${foo}`
    match (first, last) {
        ('\'', '\'') | ('"', '"') => true,
        ('`', '`') => !s.contains("${"),
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
        assert!(result.diagnostics[0].message.contains("static"));
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
