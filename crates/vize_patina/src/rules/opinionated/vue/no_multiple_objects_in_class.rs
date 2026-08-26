//! vue/no-multiple-objects-in-class
//!
//! Disallow multiple object literals inside a `:class` array binding.
//!
//! `:class="[{ a }, { b }]"` spreads class state across several object
//! literals when a single merged object reads more clearly and produces the
//! same result. Prefer `:class="{ a, b }"`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div :class="[{ a }, { b }]"></div>
//! <div :class="[{ active: isActive }, { error: hasError }]"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div :class="{ a, b }"></div>
//! <div :class="[{ active: isActive }, 'static']"></div>
//! <div :class="[foo, bar]"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpressionElement, Expression};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-multiple-objects-in-class",
    description: "Disallow multiple object literals inside a :class array binding",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow multiple object literals inside a `:class` array binding.
pub struct NoMultipleObjectsInClass;

impl Rule for NoMultipleObjectsInClass {
    fn meta(&self) -> &'static RuleMeta {
        &META
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
        // Only a static `:class` argument, not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        if !arg.is_static {
            return;
        }
        if arg.content != "class" {
            return;
        }
        let Some(ExpressionNode::Simple(exp)) = &directive.exp else {
            return;
        };
        if count_top_level_objects_in_array(exp.content) >= 2 {
            ctx.warn_with_help(
                ctx.t("vue/no-multiple-objects-in-class.message"),
                &directive.loc,
                ctx.t("vue/no-multiple-objects-in-class.help"),
            );
        }
    }
}

/// Count object literals that sit directly in a `:class` array binding.
fn count_top_level_objects_in_array(raw: &str) -> usize {
    let source = raw.trim();
    if !source.starts_with('[') || !source.ends_with(']') {
        return 0;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let Ok(parsed) = Parser::new(&allocator, source, source_type).parse_expression() else {
        return 0;
    };
    let Some(rest) = source.get(parsed.span().end as usize..) else {
        return 0;
    };
    if !rest.trim().is_empty() {
        return 0;
    }

    let Expression::ArrayExpression(array) = unwrap_expression(&parsed) else {
        return 0;
    };

    array
        .elements
        .iter()
        .filter(|element| array_element_is_object(element))
        .count()
}

fn array_element_is_object<'a>(element: &'a ArrayExpressionElement<'a>) -> bool {
    match element {
        ArrayExpressionElement::ObjectExpression(_) => true,
        ArrayExpressionElement::ParenthesizedExpression(paren) => {
            expression_is_object(&paren.expression)
        }
        ArrayExpressionElement::TSAsExpression(ts_as) => expression_is_object(&ts_as.expression),
        ArrayExpressionElement::TSNonNullExpression(ts_non_null) => {
            expression_is_object(&ts_non_null.expression)
        }
        ArrayExpressionElement::TSSatisfiesExpression(ts_satisfies) => {
            expression_is_object(&ts_satisfies.expression)
        }
        _ => false,
    }
}

fn expression_is_object<'a>(expression: &'a Expression<'a>) -> bool {
    matches!(
        unwrap_expression(expression),
        Expression::ObjectExpression(_)
    )
}

fn unwrap_expression<'a>(mut expression: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        match expression {
            Expression::ParenthesizedExpression(paren) => expression = &paren.expression,
            Expression::TSAsExpression(ts_as) => expression = &ts_as.expression,
            Expression::TSNonNullExpression(ts_non_null) => expression = &ts_non_null.expression,
            Expression::TSSatisfiesExpression(ts_satisfies) => {
                expression = &ts_satisfies.expression;
            }
            _ => return expression,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoMultipleObjectsInClass;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoMultipleObjectsInClass));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_two_object_literals() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="[{ a }, { b }]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_dynamic_class_argument() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :[class]="[{ a }, { b }]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_two_objects_with_keys() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[{ active: isActive }, { error: hasError }]"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_single_object_in_array() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[{ active: isActive }, 'static']"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_template_literal_and_single_object_in_array() {
        let linter = create_linter();
        let result = linter.lint_sfc(
            r##"<script setup lang="ts">
defineProps<{ device: string; isVertical: boolean }>();
</script>

<template>
  <div :class='[`rendered-${device}`, { "vertical-rendered": isVertical }]' />
</template>"##,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0, "got: {:?}", result.diagnostics);
    }

    #[test]
    fn reports_two_objects_alongside_template_literal() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[`a${b}`, { x: y }, { z: w }]" />"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn ignores_braces_inside_string_literals() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="['{', { a: b }]" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_objects_inside_call_arguments() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div :class="[cls({ a: 1 }), { b: 2 }]" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_plain_object_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="{ a, b }"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_array_of_identifiers() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="[foo, bar]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_nested_objects() {
        // A single top-level object whose value is an object must not be
        // miscounted as two.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[{ a: { nested: true } }]"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_other_bindings() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :style="[{ a }, { b }]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}
