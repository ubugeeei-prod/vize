//! vue/valid-v-for
//!
//! Enforce valid `v-for` directives.
//!
//! This rule checks the following:
//! - `v-for` directive has an expression
//! - `v-for` directive's expression is valid (contains "in" or "of")
//! - `v-for` directive doesn't have invalid modifiers
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-for></div>
//! <div v-for=""></div>
//! <div v-for.stop="item in items"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-for="item in items" :key="item.id"></div>
//! <div v-for="(item, index) in items" :key="index"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::visitor::{has_free_reference, has_free_reference_retained, parse_v_for_variables};
use oxc_ast::ast::BindingPattern;
use oxc_span::GetSpan;
use vize_croquis::drawer::retain_v_for_binding_parameters;
use vize_l0::{Allocator, CompactString, ensure_sufficient_stack};
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, JsExpression, PropNode, TemplateChildNode,
};

use super::require_v_for_key::relief_element_is_custom_component;

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-for",
    description: "Enforce valid `v-for` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Enforce valid v-for directives
pub struct ValidVFor;

impl Rule for ValidVFor {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        // Only check v-for directives
        if directive.name != "for" {
            return;
        }

        if !ctx.is_petite_vue() {
            check_iteration_keys(ctx, element, directive.exp.as_ref(), false);
        }

        // Check for modifiers (v-for should not have modifiers)
        if !directive.modifiers.is_empty() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-for.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-for.help"),
            );
            return;
        }

        // Check for argument (v-for should not have an argument like v-for:something)
        if directive.arg.is_some() {
            ctx.error_with_help(
                ctx.t("vue/valid-v-for.missing_expression"),
                &directive.loc,
                ctx.t("vue/valid-v-for.help"),
            );
            return;
        }

        // Check for expression
        match &directive.exp {
            None => {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-for.missing_expression"),
                    &directive.loc,
                    ctx.t("vue/valid-v-for.help"),
                );
            }
            Some(exp) => {
                // Validate the expression format
                let content = match exp {
                    ExpressionNode::Simple(s) => s.content,
                    ExpressionNode::Compound(_) => return, // Complex expressions are harder to validate
                };

                if content.trim().is_empty() {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.missing_expression"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                    return;
                }

                // Vue splits on any whitespace around `in`/`of`, line breaks included.
                if !aliases_are_valid(content) {
                    ctx.error_with_help(
                        ctx.t("vue/valid-v-for.invalid_syntax"),
                        &directive.loc,
                        ctx.t("vue/valid-v-for.help"),
                    );
                }
            }
        }
    }
}

fn aliases_are_valid(expression: &str) -> bool {
    let allocator = Allocator::default();
    let Some(bindings) = retain_v_for_binding_parameters(&allocator, expression) else {
        return false;
    };
    let parameters = &bindings.parameters;
    if parameters.items.is_empty()
        || parameters.items.len() > 3
        || parameters.rest.is_some()
        || has_empty_alias_position(bindings.raw, parameters)
    {
        return false;
    }
    parameters.items.iter().skip(1).all(|parameter| {
        matches!(parameter.pattern, BindingPattern::BindingIdentifier(_))
            && parameter.initializer.is_none()
    })
}

fn has_empty_alias_position(raw: &str, parameters: &oxc_ast::ast::FormalParameters<'_>) -> bool {
    let Some(first) = parameters.items.first() else {
        return true;
    };
    if separator_count(raw.get(..first.span().start as usize).unwrap_or_default()) > 0 {
        return true;
    }
    for pair in parameters.items.windows(2) {
        let [left_item, right_item] = pair else {
            continue;
        };
        let left = left_item.span().end as usize;
        let right = right_item.span().start as usize;
        if separator_count(raw.get(left..right).unwrap_or_default()) > 1 {
            return true;
        }
    }
    separator_count(
        raw.get(
            parameters
                .items
                .last()
                .map_or(0, |item| item.span().end as usize)..,
        )
        .unwrap_or_default(),
    ) > 1
}

fn separator_count(gap: &str) -> usize {
    let bytes = gap.as_bytes();
    let mut count = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index..index + 2) == Some(b"//") {
            index += 2;
            while bytes
                .get(index)
                .is_some_and(|byte| !matches!(*byte, b'\n' | b'\r'))
            {
                index += 1;
            }
        } else if bytes.get(index..index + 2) == Some(b"/*") {
            index += 2;
            while index + 1 < bytes.len() && bytes.get(index..index + 2) != Some(b"*/") {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
        } else {
            count += usize::from(bytes.get(index) == Some(&b','));
            index += 1;
        }
    }
    count
}

fn check_iteration_keys<'a>(
    ctx: &mut LintContext<'a>,
    element: &ElementNode<'a>,
    v_for_exp: Option<&ExpressionNode<'a>>,
    inherited_iteration: bool,
) {
    // A template that combines v-for with v-slot creates one slot function per
    // iteration. Its children are slot bodies rather than repeated VNodes, so
    // they do not need keys. Keep validating the v-for expression itself, but
    // do not forward the iteration's key requirement into the slot body.
    if element.tag == "template"
        && element
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
    {
        return;
    }

    if element.tag == "template" && bound_key_directive(element).is_none() {
        let variables = v_for_exp.map(parse_v_for_variables).unwrap_or_default();
        for child in &element.children {
            if let TemplateChildNode::Element(child) = child {
                // A nested loop over the parent's alias is keyed by its own iteration.
                let depends_on_parent = child.props.iter().any(|prop| {
                    let PropNode::Directive(directive) = prop else {
                        return false;
                    };
                    if directive.name != "for" {
                        return false;
                    }
                    let Some(ExpressionNode::Simple(expression)) = &directive.exp else {
                        return false;
                    };
                    let Some(retained) =
                        ctx.compatibility_v_for(expression.loc.span, expression.content)
                    else {
                        return false;
                    };
                    let span = retained.source.span();
                    let start = retained.source_offset as usize + span.start as usize;
                    let end = retained.source_offset as usize + span.end as usize;
                    let Some(source) = expression.content.get(start..end) else {
                        return false;
                    };
                    has_free_reference_retained(
                        source,
                        &JsExpression {
                            ast: retained.source,
                            raw: source,
                        },
                        |reference| variables.iter().any(|name| name.as_str() == reference),
                    )
                });
                if !depends_on_parent {
                    ensure_sufficient_stack(|| check_iteration_keys(ctx, child, v_for_exp, true));
                }
            }
        }
        return;
    }
    if relief_element_is_custom_component(element)
        && element.tag != "template"
        // An authored <slot v-for> does not create a component vnode that
        // needs a key. A <slot> repeated by a parent <template v-for> is the
        // iteration body only when it carries fallback VNodes; an empty outlet
        // is still exempt.
        && (element.tag != "slot"
            || (inherited_iteration && slot_has_unkeyed_fallback(element)))
        && bound_key_directive(element).is_none()
    {
        ctx.error_with_help(
            ctx.t("vue/valid-v-for.require_key"),
            &element.loc,
            ctx.t("vue/valid-v-for.help"),
        );
    }
    if let Some(expression) = v_for_exp {
        check_key_uses_v_for_variables(ctx, element, expression);
    }
}

fn slot_has_unkeyed_fallback(element: &ElementNode<'_>) -> bool {
    element.children.iter().any(|child| {
        let TemplateChildNode::Element(child) = child else {
            return false;
        };
        if bound_key_directive(child).is_some()
            || child.props.iter().any(
                |prop| matches!(prop, PropNode::Directive(directive) if directive.name == "for"),
            )
        {
            return false;
        }
        if matches!(child.tag, "template" | "slot") {
            return slot_has_unkeyed_fallback(child);
        }
        true
    })
}

fn check_key_uses_v_for_variables(
    ctx: &mut LintContext<'_>,
    element: &ElementNode<'_>,
    v_for_exp: &ExpressionNode<'_>,
) {
    let Some(key_directive) = bound_key_directive(element) else {
        return;
    };
    let Some(ExpressionNode::Simple(key_expression)) = &key_directive.exp else {
        return;
    };

    let vars = parse_v_for_variables(v_for_exp);
    if vars.is_empty() {
        return;
    }
    if expression_references_any(key_expression, &vars) {
        return;
    }

    ctx.error_with_help(
        ctx.t("vue/valid-v-for.key_uses_variables"),
        &key_directive.loc,
        ctx.t("vue/valid-v-for.help"),
    );
}

fn bound_key_directive<'a>(element: &'a ElementNode<'a>) -> Option<&'a DirectiveNode<'a>> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir)
            if dir.name == "bind"
                && matches!(
                    dir.arg.as_ref(),
                    Some(ExpressionNode::Simple(arg)) if arg.content == "key"
                ) =>
        {
            Some(dir.as_ref())
        }
        _ => None,
    })
}

fn expression_references_any(
    expression: &vize_relief::SimpleExpressionNode<'_>,
    names: &[CompactString],
) -> bool {
    has_free_reference(expression, |reference| {
        names.iter().any(|name| name.as_str() == reference)
    })
}

#[cfg(test)]
mod key_tests;

#[cfg(test)]
mod tests;
