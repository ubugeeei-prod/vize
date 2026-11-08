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
use crate::markup::{
    MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use oxc_span::GetSpan;
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
                let Some(expression) = binding.retained_expression() else {
                    return;
                };
                match static_style_findings(expression.ast) {
                    StyleFindings::None => {}
                    StyleFindings::Whole => Self::report(ctx, binding.range()),
                    StyleFindings::Properties(properties) => {
                        for property in properties {
                            let Some(range) = expression.authored_range(property) else {
                                continue;
                            };
                            Self::report(ctx, range);
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

fn static_style_findings(expression: &Expression<'_>) -> StyleFindings {
    match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => findings_for_objects([&**object], true),
        Expression::ArrayExpression(array) => {
            let mut objects = Vec::new();
            let mut complete = true;
            for element in &array.elements {
                let Some(expression) = element.as_expression() else {
                    complete = false;
                    break;
                };
                let Expression::ObjectExpression(object) = expression.get_inner_expression() else {
                    complete = false;
                    break;
                };
                objects.push(&**object);
            }
            findings_for_objects(objects, complete)
        }
        _ => StyleFindings::None,
    }
}

fn findings_for_objects<'a>(
    objects: impl IntoIterator<Item = &'a oxc_ast::ast::ObjectExpression<'a>>,
    complete: bool,
) -> StyleFindings {
    let mut properties = Vec::new();
    for object in objects {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return findings_from_properties(properties);
            };
            if property.computed {
                return findings_from_properties(properties);
            }
            if matches!(
                property.key,
                PropertyKey::StaticIdentifier(_)
                    | PropertyKey::StringLiteral(_)
                    | PropertyKey::NumericLiteral(_)
            ) && value_is_static(&property.value)
            {
                properties.push(property.span);
            } else {
                // Upstream finishes this object, collecting later static
                // properties, but does not inspect following array entries.
                let remaining = object
                    .properties
                    .iter()
                    .skip_while(|candidate| candidate.span() != property.span)
                    .skip(1);
                for candidate in remaining {
                    let ObjectPropertyKind::ObjectProperty(candidate) = candidate else {
                        break;
                    };
                    if candidate.computed {
                        break;
                    }
                    if matches!(
                        candidate.key,
                        PropertyKey::StaticIdentifier(_)
                            | PropertyKey::StringLiteral(_)
                            | PropertyKey::NumericLiteral(_)
                    ) && value_is_static(&candidate.value)
                    {
                        properties.push(candidate.span);
                    }
                }
                return findings_from_properties(properties);
            }
        }
    }
    if complete {
        StyleFindings::Whole
    } else {
        findings_from_properties(properties)
    }
}

fn findings_from_properties(properties: Vec<oxc_span::Span>) -> StyleFindings {
    if properties.is_empty() {
        StyleFindings::None
    } else {
        StyleFindings::Properties(properties)
    }
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

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Bindings
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

    #[test]
    fn stops_before_static_properties_after_a_computed_key() {
        let result = create_linter().lint_template(
            r#"<div :style="{ [wide ? 'width' : 'minWidth']: width, outline: 'none' }" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_static_jsx_style_binding_from_the_retained_ast() {
        let source = r#"const view = <div style={{ color: 'red', width: 10 }}>text</div>;"#;
        let result = create_linter().lint_jsx(source, "test.tsx", vize_atelier_jsx::JsxLang::Tsx);
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
        assert_eq!(
            result.diagnostics[0].rule_name,
            "vue/no-static-inline-styles"
        );
    }
}
