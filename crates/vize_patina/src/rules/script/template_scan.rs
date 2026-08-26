//! Calls a `<template>` performs, recovered exactly enough to *create* a
//! finding.
//!
//! # Why the AST plus a real parse
//!
//! A call recovered from the template creates a diagnostic, so an over-match is
//! a false positive — the direction #3223 calls dangerous, and the opposite of
//! [`crate::rules::script::props_emits::template_emits`], whose raw scan can
//! only ever suppress an "unused" report. A substring sweep for `name(` over
//! the raw template would fire on all of these, none of which call anything:
//!
//! * `<!-- total() -->` — an HTML comment.
//! * `<p>total()</p>` — a text node.
//! * `<div title="total()">` — a plain attribute, not an expression.
//! * `@click="log('total()')"` — the call is inside a string literal.
//! * `@click="subtotal()"` — a longer identifier ending in the name.
//! * `<pre v-pre>{{ total() }}</pre>` — a region Vue never compiles.
//! * `<Child v-slot="{ total }">{{ total() }}</Child>` — the name is a slot
//!   variable, not the component's own binding.
//!
//! So the evidence is taken from the **template AST** — only a directive
//! carrying an expression and an interpolation's content, which structurally
//! excludes comments, text nodes, plain attributes and `v-pre` regions (the
//! parser rewrites a `v-pre` region's directives to attributes and its
//! interpolations to text) — and each expression is then **parsed with oxc**,
//! so only a genuine `CallExpression` with a plain identifier callee counts. An
//! occurrence inside a string literal parses to a `StringLiteral` and can never
//! be a callee, and `subtotal` is a different `IdentifierReference`.
//!
//! # Direction of error
//!
//! * **Over-match** would be a false positive, so every step above is exact.
//!   The one deliberate imprecision is shadowing (a `v-for` alias or a slot
//!   variable reusing a name), and it errs towards *not* reporting.
//! * **Under-match** loses a report: an expression oxc cannot parse is skipped,
//!   as is a member callee (`bus.emit('x')`, deliberately — it dispatches on
//!   another object) and a dynamic directive argument (`:[key]="…"`).

mod calls;

use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};
use vize_s0::String;

pub(super) use calls::TemplateCall;

/// The template-visible emit helper. Available in every template expression and
/// dispatching this component's own events, unlike a member call such as
/// `child.$emit('x')`.
pub(super) const DOLLAR_EMIT: &str = "$emit";

/// Visit every call with a plain identifier callee that the template performs.
///
/// Calls whose callee is shadowed by an enclosing `v-for` alias or slot
/// variable are skipped: inside that subtree the name is the iteration /slot
/// binding, not the component's own.
pub(super) fn for_each_template_call(root: &RootNode<'_>, mut visit: impl FnMut(TemplateCall<'_>)) {
    let mut walker = Walker {
        source: root.source,
        shadowed: Vec::new(),
    };
    walker.walk(&root.children, &mut visit);
}

struct Walker<'s> {
    /// The template source node-loc spans index into (`RootNode::source`).
    source: &'s str,
    /// Names bound by an enclosing `v-for` alias or slot variable.
    ///
    /// Collected as bare identifier tokens out of the alias / slot expression
    /// rather than by parsing the binding pattern, so a destructuring pattern
    /// contributes every name it mentions — including a renamed source key
    /// (`v-slot="{ total: sum }"` shadows both `total` and `sum`). That
    /// over-collects, which only ever *suppresses* a call, the safe direction
    /// for evidence that creates findings.
    shadowed: Vec<String>,
}

impl Walker<'_> {
    fn walk(
        &mut self,
        children: &[TemplateChildNode<'_>],
        visit: &mut impl FnMut(TemplateCall<'_>),
    ) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => self.walk_element(element, visit),
                TemplateChildNode::Interpolation(interpolation) => {
                    self.scan(&interpolation.content, visit);
                }
                // `v-if` / `v-for` are still plain directives in the parse this
                // reads, so these arms only matter if a transformed AST is ever
                // passed in. Walking them keeps that case correct.
                TemplateChildNode::If(if_node) => {
                    for branch in if_node.branches.iter() {
                        self.walk(&branch.children, visit);
                    }
                }
                TemplateChildNode::For(for_node) => self.walk(&for_node.children, visit),
                _ => {}
            }
        }
    }

    fn walk_element(
        &mut self,
        element: &ElementNode<'_>,
        visit: &mut impl FnMut(TemplateCall<'_>),
    ) {
        let depth = self.shadowed.len();

        // A `v-for` alias is in scope for the element's *own* bindings as well
        // as its children (`v-for="total in rows" @click="total()"` calls the
        // iteration variable), so it is collected before any directive here is
        // scanned.
        for prop in element.props.iter() {
            if let PropNode::Directive(directive) = prop
                && directive.name == "for"
            {
                push_for_aliases(directive, &mut self.shadowed, self.source);
            }
        }

        for prop in element.props.iter() {
            if let PropNode::Directive(directive) = prop
                && let Some(exp) = directive.exp.as_ref()
                // `v-for`'s expression is `item in items`, not JavaScript, and
                // `v-slot`'s is a binding pattern rather than a reference.
                && !matches!(directive.name, "for" | "slot")
            {
                self.scan(exp, visit);
            }
        }

        // A slot variable, by contrast, scopes the slot *content*, so it is
        // collected only after this element's own directives are scanned.
        for prop in element.props.iter() {
            if let PropNode::Directive(directive) = prop
                && directive.name == "slot"
                && let Some(exp) = directive.exp.as_ref()
            {
                push_identifier_tokens(expression_source(exp, self.source), &mut self.shadowed);
            }
        }

        self.walk(&element.children, visit);
        self.shadowed.truncate(depth);
    }

    fn scan(&self, exp: &ExpressionNode<'_>, visit: &mut impl FnMut(TemplateCall<'_>)) {
        let shadowed = &self.shadowed;
        calls::for_each_call(
            expression_source(exp, self.source),
            exp.loc().span.start,
            &mut |call| {
                if shadowed.iter().any(|name| name.as_str() == call.callee) {
                    return;
                }
                visit(call);
            },
        );
    }
}

/// Push the value / key / index aliases of a `v-for` directive.
///
/// The parsed aliases are preferred; a `v-for` the parser could not split falls
/// back to the whole expression, whose identifier tokens include the source as
/// well as the aliases. Over-collecting only suppresses calls.
fn push_for_aliases(directive: &DirectiveNode<'_>, out: &mut Vec<String>, source: &str) {
    if let Some(parsed) = directive.for_parse_result.as_ref() {
        for alias in [
            parsed.value.as_ref(),
            parsed.key.as_ref(),
            parsed.index.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            push_identifier_tokens(expression_source(alias, source), out);
        }
        return;
    }
    if let Some(exp) = directive.exp.as_ref() {
        push_identifier_tokens(expression_source(exp, source), out);
    }
}

fn expression_source<'a>(exp: &'a ExpressionNode<'a>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}

/// Push every identifier-shaped token of `source` onto `out`.
fn push_identifier_tokens(source: &str, out: &mut Vec<String>) {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !is_identifier_start(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && is_identifier_byte(bytes[index]) {
            index += 1;
        }
        out.push(String::new(&source[start..index]));
    }
}

#[inline]
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

#[inline]
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
