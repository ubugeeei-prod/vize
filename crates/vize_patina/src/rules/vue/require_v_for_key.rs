//! vue/require-v-for-key
//!
//! Require `v-bind:key` with `v-for` directives.
//!
//! This rule reports native elements using `v-for` without a `:key` attribute.
//! The key attribute is essential for Vue's virtual DOM diffing algorithm
//! to efficiently update the DOM when the list changes.
//!
//! Like eslint-plugin-vue, a finding is reported at the element's opening tag,
//! and an unkeyed `<template v-for>` or `<slot v-for>` forwards the check to
//! each child element, so each unkeyed native child is reported.
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
use crate::ir::ByteRange;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupList, MarkupNode, MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::ensure_sufficient_stack;
use vize_relief::{DirectiveNode, ElementNode};

mod components;
mod relief;

use components::markup_element_is_custom_component;
pub(super) use components::relief_element_is_custom_component;
use relief::relief_check;

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
    /// Check the repeated node of a `v-for` like eslint-plugin-vue: a keyed node
    /// is fine; an unkeyed `<template>` / `<slot>` forwards the check to each
    /// child element; an unkeyed native element is reported at its opening tag.
    fn check_keyed_element<'a>(
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        range: ByteRange,
        template_directive: bool,
    ) {
        // petite-vue does not require a `:key` on `v-for`.
        if ctx.lint().is_petite_vue() {
            return;
        }
        let exact = ctx.is_template();
        if is_transparent(element, exact) {
            if !element.has_directive("slot") && !has_markup_key(element, exact) {
                Self::check_children(ctx, element, exact);
            }
            return;
        }
        if has_markup_key(element, exact)
            || (template_directive && markup_element_is_custom_component(element))
        {
            return;
        }
        let range = if template_directive {
            element.open_tag_range(ctx.lint().source)
        } else {
            range
        };
        Self::report(ctx, element, range);
    }

    fn check_children<'a>(
        ctx: &mut MarkupContext<'_, 'a>,
        parent: &MarkupElement<'a>,
        exact: bool,
    ) {
        parent.walk_children(&mut |child| {
            let MarkupNode::Element(child) = child else {
                return;
            };
            // A child with its own loop is checked by that loop.
            let mut own_loop = false;
            child.walk_authored_directive_ranges("for", &mut |_| own_loop = true);
            // Like the reference rule, only a bound `:key` satisfies a child;
            // the root's static and object-spread key exemptions stay at the root.
            if own_loop || has_markup_bound_key(&child, exact) {
                return;
            }
            if is_transparent(&child, exact) {
                ensure_sufficient_stack(|| Self::check_children(ctx, &child, exact));
            } else if !markup_element_is_custom_component(&child) {
                let range = child.open_tag_range(ctx.lint().source);
                Self::report(ctx, &child, range);
            }
        });
    }

    fn report(ctx: &mut MarkupContext<'_, '_>, element: &MarkupElement<'_>, range: ByteRange) {
        let message = ctx
            .lint()
            .t_fmt("vue/require-v-for-key.message", &[("tag", element.tag())]);
        let help = ctx.lint().t("vue/require-v-for-key.help");
        ctx.lint().error_at_with_help(message, range, help);
    }
}

fn is_transparent(element: &MarkupElement<'_>, exact: bool) -> bool {
    matches!(element.tag(), "template" | "slot")
        || (!exact && (element.is_tag("template") || element.is_tag("slot")))
}

fn has_markup_bound_key(element: &MarkupElement<'_>, exact: bool) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= binding.kind() == MarkupBindingKind::Bind
            && if exact {
                binding.is_unqualified_arg_exact("key")
            } else {
                binding.arg_name_eq("key")
            };
    });
    found
}

fn has_markup_key(element: &MarkupElement<'_>, exact: bool) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) && if exact {
            binding.is_unqualified_arg_exact("key")
        } else {
            binding.arg_name_eq("key")
        };
    });
    found || has_object_bound_key(element)
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

pub(super) fn object_has_static_key(source: &str) -> bool {
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

/// Markup-IR entry point for `vue/require-v-for-key`.
///
/// Demonstrates the unified rule IR: the same logic runs over a Vue template
/// **and** over JSX/TSX. `v-for` has two shapes the facade normalizes over:
///
/// - *Pre-transform* (a freshly parsed Vue template): the `v-for` is a
///   directive on the repeated element — handled in [`Self::enter_attributes`].
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

    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let template_directive = ctx.is_template();
        element.walk_authored_directive_ranges("for", &mut |range| {
            Self::check_keyed_element(ctx, element, range, template_directive);
        });
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        if list.has_authored_directive() {
            return;
        }
        // Transformed scopes carry no authored directive attribute.
        list.walk_elements(&mut |element| {
            Self::check_keyed_element(ctx, &element, element.range(), false);
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

        relief_check(ctx, element, true);
    }
}

#[cfg(test)]
mod tests;
