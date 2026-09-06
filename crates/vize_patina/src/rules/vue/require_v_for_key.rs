//! vue/require-v-for-key
//!
//! Require `v-bind:key` with `v-for` directives.
//!
//! This rule reports elements using `v-for` without a `:key` attribute.
//! The key attribute is essential for Vue's virtual DOM diffing algorithm
//! to efficiently update the DOM when the list changes.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <li v-for="item in items">{{ item }}</li>
//! ```
//!
//! ### Valid
//! ```vue
//! <li v-for="item in items" :key="item.id">{{ item }}</li>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupList, MarkupNode, MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/require-v-for-key",
    description: "Require `v-bind:key` with `v-for` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Require v-bind:key with v-for directives
pub struct RequireVForKey;

impl RequireVForKey {
    /// Report when `element` (the repeated node of a `v-for`) lacks a key.
    fn check_keyed_element<'a>(ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        // petite-vue does not require a `:key` on `v-for`.
        if ctx.lint().is_petite_vue() {
            return;
        }
        if element.is_tag("slot") {
            return;
        }
        if element.is_tag("template") {
            if has_markup_template_v_for_key(element) {
                return;
            }
        } else if has_markup_key(element) {
            return;
        }

        let tag = element.tag();
        let message = ctx
            .lint()
            .t_fmt("vue/require-v-for-key.message", &[("tag", tag)]);
        let help = ctx.lint().t("vue/require-v-for-key.help");
        ctx.lint()
            .error_at_with_help(message, element.range(), help);
    }
}

fn has_markup_key(element: &MarkupElement<'_>) -> bool {
    element.has_key_binding() || has_object_bound_key(element)
}

fn has_markup_template_v_for_key(element: &MarkupElement<'_>) -> bool {
    if element.has_directive("slot") {
        return true;
    }
    if has_markup_key(element) {
        return true;
    }

    let mut found = false;
    element.walk_children(&mut |child| {
        if found {
            return;
        }
        if let MarkupNode::Element(child) = child {
            found = child.binding(MarkupBindingKind::Bind, "key").is_some();
        }
    });
    found
}

fn has_object_bound_key(element: &MarkupElement<'_>) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        if !found
            && binding.kind() == MarkupBindingKind::Bind
            && binding.arg_name().is_none()
            && binding.expression().is_some_and(object_has_static_key)
        {
            found = true;
        }
    });
    found
}

fn object_has_static_key(source: &str) -> bool {
    let source = source.trim();
    if !matches!(source.as_bytes().first(), Some(b'{') | Some(b'(')) {
        return false;
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_else(|_| SourceType::ts());
    let Ok(expression) = Parser::new(&allocator, source, source_type).parse_expression() else {
        return false;
    };
    if expression.span().end as usize != source.len() {
        return false;
    }

    let Some(root) = object_expression(&expression) else {
        return false;
    };
    let mut objects = vec![root];
    while let Some(object) = objects.pop() {
        for property in &object.properties {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    if is_static_key(&property.key) {
                        return true;
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    if let Some(object) = object_expression(&spread.argument) {
                        objects.push(object);
                    }
                }
            }
        }
    }
    false
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(parenthesized) => {
            object_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(ts_as) => object_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(satisfies) => object_expression(&satisfies.expression),
        Expression::TSNonNullExpression(non_null) => object_expression(&non_null.expression),
        _ => None,
    }
}

fn is_static_key(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name == "key",
        PropertyKey::StringLiteral(literal) => literal.value == "key",
        _ => false,
    }
}

fn relief_element_has_key(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == "key",
        PropNode::Directive(dir) => {
            if relief_directive_is_bound_key(dir) {
                return true;
            }
            dir.name == "bind"
                && dir.arg.is_none()
                && matches!(&dir.exp, Some(ExpressionNode::Simple(expression)) if object_has_static_key(expression.content))
        }
    })
}

fn relief_directive_is_bound_key(directive: &DirectiveNode<'_>) -> bool {
    directive.name == "bind"
        && matches!(
            directive.arg.as_ref(),
            Some(ExpressionNode::Simple(arg)) if arg.content == "key"
        )
}

fn relief_template_v_for_has_key(element: &ElementNode<'_>) -> bool {
    relief_template_has_slot_directive(element)
        || relief_element_has_key(element)
        || element.children.iter().any(|child| {
            matches!(
                child,
                TemplateChildNode::Element(child) if child
                    .props
                    .iter()
                    .any(|prop| matches!(prop, PropNode::Directive(dir) if relief_directive_is_bound_key(dir)))
            )
        })
}

fn relief_template_has_slot_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(
            prop,
            PropNode::Directive(directive) if directive.name == "slot"
        )
    })
}

/// Markup-IR entry point for `vue/require-v-for-key`.
///
/// Demonstrates the unified rule IR: the same logic runs over a Vue template
/// **and** over JSX/TSX. `v-for` has two shapes the facade normalizes over:
///
/// - *Pre-transform* (a freshly parsed Vue template): the `v-for` is a
///   directive on the repeated element — handled in [`Self::enter_element`].
/// - *Post-transform* (lowered JSX `items.map((i) => <li/>)`, or a transformed
///   template): the repeated element is wrapped by a list scope — handled in
///   [`Self::enter_list`].
///
/// Either way the rule only asks "does this element have a key binding?", and
/// `key={…}` lowers to the very same `:key` (`bind` directive, arg `key`).
impl MarkupRule for RequireVForKey {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        // Pre-transform shape: the element itself carries the `v-for` directive.
        if element.has_directive("for") {
            Self::check_keyed_element(ctx, element);
        }
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        // Post-transform shape: the list scope wraps the repeated element(s).
        list.walk_elements(&mut |element| {
            Self::check_keyed_element(ctx, &element);
        });
    }
}

impl Rule for RequireVForKey {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // `v-for`'s JSX form is `items.map(…)`, which is a JS expression with no
        // list scope until lowering. The `enter_list` hook only fires over the
        // lowered relief AST, so route this rule there for JSX.
        true
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

        // petite-vue does not require a :key on v-for, so this Vue-3-only rule
        // must not fire on petite-vue documents.
        if ctx.is_petite_vue() {
            return;
        }

        if element.tag == "template" {
            if relief_template_v_for_has_key(element) {
                return;
            }
        } else if element.tag == "slot" || relief_element_has_key(element) {
            return;
        }

        ctx.error_with_help(
            ctx.t_fmt("vue/require-v-for-key.message", &[("tag", element.tag)]),
            &directive.loc,
            ctx.t("vue/require-v-for-key.help"),
        );
    }
}

#[cfg(test)]
mod tests;
