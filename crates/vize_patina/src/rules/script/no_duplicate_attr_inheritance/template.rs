//! The template signal: `v-bind="$attrs"` on the component's root element.
//!
//! # Why the AST, and why the *root* element specifically
//!
//! A recovered `v-bind="$attrs"` *creates* a finding, so an over-match is a
//! false positive. Taking the evidence from the template AST excludes the cases
//! a raw scan over the template text cannot:
//!
//! * `<!-- v-bind="$attrs" -->` is a comment node;
//! * `<p>v-bind="$attrs"</p>` is a text node;
//! * `@click="log('$attrs')"` is a `v-on` expression, not a `v-bind`, and the
//!   occurrence is inside a string literal;
//! * inside a `v-pre` region the parser rewrites every directive to a plain
//!   attribute, so nothing there is a `DirectiveNode` at all.
//!
//! The trap is **which element**. Upstream reports the duplication on the
//! *root* element's `v-bind="$attrs"`, because that is where the fallthrough
//! attributes are applied a second time. `v-bind="$attrs"` on a nested element
//! is the documented way to forward attributes to an inner node, is idiomatic,
//! and is normally paired with `inheritAttrs: false`; reporting it would be a
//! false positive on correct code. So only the single root element counts, and
//! a multi-root (fragment) template — which has no single fallthrough target —
//! records nothing.
//!
//! # Direction of error
//!
//! * **Over-match** would be a false positive, so the directive must be a
//!   `bind` with **no argument** whose expression is exactly `$attrs`. `:x="$attrs"`
//!   carries an argument and binds one prop; `v-bind="$attrsExtra"` is a
//!   different identifier.
//! * **Under-match** loses a report: an expression that only *contains* `$attrs`
//!   (`v-bind="{ ...$attrs }"`) is not matched, and neither is a root
//!   `<template v-if>` wrapper, whose real root is decided at runtime.

use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode};

/// The location of a root `v-bind="$attrs"`, relative to the template block.
pub(super) struct AttrsSpread {
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Find `v-bind="$attrs"` on the template's single root element.
///
/// Returns `None` when the template has no single root element, when that root
/// is a `<template>` wrapper (its rendered root is not statically known), or
/// when the root does not spread `$attrs`.
pub(super) fn root_attrs_spread(root: &RootNode<'_>) -> Option<AttrsSpread> {
    let element = single_root_element(root)?;
    // A root `<template v-if>` / `<template v-for>` renders its children, so
    // the element that receives the fallthrough attributes is not this node.
    if element.tag == "template" {
        return None;
    }
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(directive) if is_attrs_spread(directive, root.source) => {
            Some(AttrsSpread {
                start: directive.loc.span.start,
                end: directive.loc.span.end,
            })
        }
        _ => None,
    })
}

/// The template's single root element, if it has exactly one.
///
/// Whitespace-only text and comments are not rendered content, so they do not
/// make a template multi-root. Anything else that renders — a second element,
/// an interpolation, real text — does, and a fragment has no single element to
/// receive the fallthrough attributes.
fn single_root_element<'a, 'ast>(root: &'a RootNode<'ast>) -> Option<&'a ElementNode<'ast>> {
    let mut found = None;
    for child in root.children.iter() {
        match child {
            vize_relief::TemplateChildNode::Comment(_) => {}
            vize_relief::TemplateChildNode::Text(text) if text.content.trim().is_empty() => {}
            vize_relief::TemplateChildNode::Element(element) if found.is_none() => {
                found = Some(&**element);
            }
            _ => return None,
        }
    }
    found
}

/// Whether the directive is exactly `v-bind="$attrs"`.
///
/// An argument (`:id="$attrs"`) binds a single prop rather than spreading the
/// fallthrough attributes, so it does not count. The expression is compared as
/// a whole — trimmed — rather than searched: the construct *is* the entire
/// expression, so equality is exact, and anything richer
/// (`v-bind="{ ...$attrs }"`) is deliberately not matched.
fn is_attrs_spread(directive: &DirectiveNode<'_>, source: &str) -> bool {
    if directive.name != "bind" || directive.arg.is_some() {
        return false;
    }
    directive
        .exp
        .as_ref()
        .is_some_and(|exp| expression_source(exp, source).trim() == "$attrs")
}

fn expression_source<'a>(exp: &'a ExpressionNode<'a>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}
