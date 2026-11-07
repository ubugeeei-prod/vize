//! vue/no-static-inline-styles
//!
//! Disallow static inline `style` attributes.
//!
//! This follows eslint-plugin-vue's `vue/no-static-inline-styles` surface:
//! literal `style="..."` attributes and fully static `:style` object/array
//! bindings are reported, while bindings that depend on runtime state remain
//! valid.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-static-inline-styles",
    description: "Disallow static inline style attributes",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoStaticInlineStyles;

impl NoStaticInlineStyles {
    fn check_binding(ctx: &mut LintContext<'_>, binding: &MarkupBinding<'_>) {
        if !binding.is_unqualified_arg_exact("style") {
            return;
        }
        match binding.kind() {
            MarkupBindingKind::Attribute if binding.static_value().is_some() => {
                Self::report(ctx, binding.range());
            }
            MarkupBindingKind::Bind => {
                let Some(expression) = binding.expression() else {
                    return;
                };
                match static_style_findings(expression) {
                    StyleFindings::None => {}
                    StyleFindings::Whole => Self::report(ctx, binding.range()),
                    StyleFindings::Properties(properties) => {
                        let range = binding.range();
                        let Some(authored) =
                            ctx.source.get(range.start as usize..range.end as usize)
                        else {
                            return;
                        };
                        let Some(relative) = authored.find(expression) else {
                            return;
                        };
                        let expression_start = range.start + relative as u32;
                        for property in properties {
                            Self::report(
                                ctx,
                                ByteRange::new(
                                    expression_start + property.start,
                                    expression_start + property.end,
                                ),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn report(ctx: &mut LintContext<'_>, range: ByteRange) {
        ctx.warn_at_with_help(
            "Static inline style attributes are not allowed.",
            range,
            "Move the style to a class, CSS module, or dynamic style binding.",
        );
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        element.walk_bindings(&mut |binding| Self::check_binding(ctx, &binding));
    }
}

enum StyleFindings {
    None,
    Whole,
    Properties(Vec<oxc_span::Span>),
}

fn static_style_findings(source: &str) -> StyleFindings {
    let source = source.trim();
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, source, SourceType::ts()).parse_expression()
    else {
        return StyleFindings::None;
    };
    if source
        .get(expression.span().end as usize..)
        .is_none_or(|rest| !rest.trim().is_empty())
    {
        return StyleFindings::None;
    }
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) if object_is_fully_static(object) => {
            StyleFindings::Whole
        }
        Expression::ArrayExpression(array)
            if array.elements.iter().all(|element| {
                let Some(expression) = element.as_expression() else {
                    return false;
                };
                matches!(expression.get_inner_expression(), Expression::ObjectExpression(object) if object_is_fully_static(object))
            }) =>
        {
            StyleFindings::Whole
        }
        Expression::ObjectExpression(object) => {
            let properties = static_properties(object);
            if properties.is_empty() {
                StyleFindings::None
            } else {
                StyleFindings::Properties(properties)
            }
        }
        Expression::ArrayExpression(array) => {
            let properties = array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
                .filter_map(|expression| match expression.get_inner_expression() {
                    Expression::ObjectExpression(object) => Some(static_properties(object)),
                    _ => None,
                })
                .flatten()
                .collect::<Vec<_>>();
            if properties.is_empty() {
                StyleFindings::None
            } else {
                StyleFindings::Properties(properties)
            }
        }
        _ => StyleFindings::None,
    }
}

fn static_properties(object: &oxc_ast::ast::ObjectExpression<'_>) -> Vec<oxc_span::Span> {
    object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            (!property.computed
                && matches!(
                    property.key,
                    PropertyKey::StaticIdentifier(_)
                        | PropertyKey::StringLiteral(_)
                        | PropertyKey::NumericLiteral(_)
                )
                && value_is_static(&property.value))
            .then_some(property.span)
        })
        .collect()
}

fn object_is_fully_static(object: &oxc_ast::ast::ObjectExpression<'_>) -> bool {
    object.properties.iter().all(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        !property.computed
            && matches!(
                property.key,
                PropertyKey::StaticIdentifier(_)
                    | PropertyKey::StringLiteral(_)
                    | PropertyKey::NumericLiteral(_)
            )
            && value_is_static(&property.value)
    })
}

fn value_is_static(value: &Expression<'_>) -> bool {
    matches!(
        value.get_inner_expression(),
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

impl MarkupRule for NoStaticInlineStyles {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        Self::check_binding(ctx.lint(), binding);
    }
}

impl Rule for NoStaticInlineStyles {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoStaticInlineStyles;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoStaticInlineStyles));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_static_style_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div style="color: red">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/no-static-inline-styles"
        );
    }

    #[test]
    fn reports_static_style_binding() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :style="{ color: 'red', width: 10 }">text</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_dynamic_style_binding() {
        let result = create_linter().lint_template(
            r#"<div :style="{ color: theme.color }">text</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_v_bind_style_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-bind:style="styles">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_class_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="text-red">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_static_property_inside_otherwise_dynamic_style_binding() {
        let source = r#"<div :style="{ background: color, width: '10px' }" />"#;
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
        let property = "width: '10px'";
        let start = source.find(property).unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, start);
        assert_eq!(result.diagnostics[0].end, start + property.len() as u32);
    }
}
