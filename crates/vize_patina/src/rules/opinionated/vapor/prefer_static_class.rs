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
use crate::ir::ByteRange;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::opinionated::class_values::unconditional_static_parts;
use vize_armature::patterns::attribute_source_span;
use vize_l0::String;
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
/// when that literal could be a plain static `class` instead. Authored Vue
/// bindings retain the same diagnostic and fix ranges on both paths.
impl MarkupRule for PreferStaticClass {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> crate::markup::MarkupTraversal {
        crate::markup::MarkupTraversal::Bindings
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if binding.kind() != MarkupBindingKind::Bind
            || if ctx.is_template() {
                binding.arg_name() != Some("class")
            } else {
                !binding.arg_name_eq("class")
            }
        {
            return;
        }
        let Some(expression) = binding.expression() else {
            return;
        };
        // If a static `class` attribute is already present, the dynamic one is
        // redundant but cannot simply be folded in; just flag it.
        let mut has_static_class = false;
        element.walk_bindings(&mut |other| {
            if other.kind() == MarkupBindingKind::Attribute && other.arg_name_eq("class") {
                has_static_class = true;
            }
        });

        let message = ctx.lint().t("vapor/prefer-static-class.message");
        if is_string_literal(expression.trim()) && has_static_class {
            let help = ctx.lint().t("vapor/prefer-static-class.help");
            ctx.lint().warn_at_with_help(message, binding.range(), help);
        } else if is_string_literal(expression.trim()) && ctx.is_template() {
            let Some(argument) = binding.argument_range() else {
                return;
            };
            let range = binding.range();
            report_static_class(
                ctx.lint(),
                message.as_ref(),
                expression.trim(),
                argument.start,
                range.start,
                range.end,
            );
        } else if is_string_literal(expression.trim()) {
            ctx.lint().warn_at(message, binding.range());
        } else if let Some(retained) = binding.retained_expression() {
            let mut parts = Vec::new();
            unconditional_static_parts(retained.ast, &mut parts);
            for part in parts
                .into_iter()
                .filter(|part| !part.text.trim().is_empty())
            {
                let range = retained
                    .authored_range(part.span)
                    .unwrap_or(binding.range());
                let help = ctx.lint().t("vapor/prefer-static-class.help");
                ctx.lint().warn_at_with_help(message.clone(), range, help);
            }
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
        if directive.name != "bind" {
            return;
        }

        let arg = match &directive.arg {
            Some(ExpressionNode::Simple(s)) if s.content == "class" => s,
            _ => return,
        };

        // Check if the expression is a string literal
        let Some(ref exp) = directive.exp else {
            return;
        };

        let simple = match exp {
            ExpressionNode::Simple(simple) => simple,
            _ => return,
        };

        // Check if it's a simple string literal like "'foo'" or "`foo`" or "\"foo\""
        let trimmed = simple.content.trim();
        if is_string_literal(trimmed) {
            // Check if element already has a static class attribute
            let has_static_class = element.props.iter().any(|p| {
                matches!(p, PropNode::Attribute(attr) if attr.name.eq_ignore_ascii_case("class"))
            });

            let message = ctx.t("vapor/prefer-static-class.message");

            // Create fix: replace :class="'value'" with class="value"
            if !has_static_class {
                report_static_class(
                    ctx,
                    message.as_ref(),
                    trimmed,
                    arg.loc.span.start,
                    directive.loc.span.start,
                    directive.loc.span.end,
                );
            } else {
                ctx.warn_with_help(
                    message,
                    &directive.loc,
                    ctx.t("vapor/prefer-static-class.help"),
                );
            }
        } else if let Some(parsed) = simple.js_ast.and_then(|ast| ast.as_expression()) {
            let mut parts = Vec::new();
            unconditional_static_parts(parsed.ast, &mut parts);
            let start = simple.loc.span.start as usize;
            let end = simple.loc.span.end as usize;
            let authored = ctx.source.get(start..end).unwrap_or_default();
            for part in parts
                .into_iter()
                .filter(|part| !part.text.trim().is_empty())
            {
                let Some(span) = attribute_source_span(authored, simple.content, part.span) else {
                    continue;
                };
                ctx.warn_at_with_help(
                    ctx.t("vapor/prefer-static-class.message"),
                    ByteRange::new(
                        simple.loc.span.start + span.start,
                        simple.loc.span.start + span.end,
                    ),
                    ctx.t("vapor/prefer-static-class.help"),
                );
            }
        }
    }
}

fn report_static_class(
    ctx: &mut LintContext<'_>,
    message: &str,
    expression: &str,
    argument_start: u32,
    binding_start: u32,
    binding_end: u32,
) {
    let inner = expression.get(1..expression.len() - 1).unwrap_or_default();
    let mut replacement = String::from("class=\"");
    replacement.push_str(inner);
    replacement.push('"');
    let fix = Fix::new(
        "Replace with static class attribute",
        TextEdit::replace(binding_start, binding_end, replacement),
    );
    ctx.report(
        crate::diagnostic::LintDiagnostic::warn(META.name, message, argument_start, binding_end)
            .with_fix(fix),
    );
}

/// Check if a string is a simple string literal
fn is_string_literal(s: &str) -> bool {
    let bytes = s.as_bytes();
    let (Some(&first), Some(&last)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    if bytes.len() < 2 {
        return false;
    }

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
    fn test_valid_joined_template_literal_with_expression() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="`prefix-${suffix}`"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_reports_standalone_template_literal_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="`prefix ${suffix}`"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_reports_static_array_items_and_true_object_keys() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="['f', { g: true, conditional: active }]" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2, "{:?}", result.diagnostics);
    }
}
