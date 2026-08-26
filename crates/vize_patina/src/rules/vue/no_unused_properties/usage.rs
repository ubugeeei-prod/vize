//! Where a prop name can be referenced, and how `defineProps` is consumed.
//!
//! # Direction of error
//!
//! `vue/no-unused-properties` reports the **absence** of a reference, so the
//! usage scan is the mirror image of the evidence scans in
//! [`crate::rules::script::template_scan`]: here an **under-match is the false
//! positive** (a reference this misses becomes an "unused" report on a prop the
//! component does use), and an over-match only costs a missed finding.
//!
//! Every step therefore over-approximates on purpose:
//!
//! * Identifier-shaped **tokens** are collected, not resolved references. So
//!   `props.msg`, `msg.length` and even `'msg'` inside a template expression all
//!   count as a reference to `msg`.
//! * Shadowing is **not** honoured. `v-for="msg in rows"` binds an iteration
//!   variable rather than the prop, but treating it as a reference only
//!   suppresses a report.
//! * The whole script block is scanned, minus the `defineProps` call itself —
//!   which must be excluded, since the declaration always spells the name and
//!   would otherwise mark every prop used.
//!
//! The one place precision is used is the template *AST*: a name that appears
//! only in an HTML comment, a text node or a plain attribute is genuinely not a
//! reference, and Vue never compiles a `v-pre` region. Every position Vue does
//! compile is scanned — a directive's expression *and* its argument, an
//! interpolation's content, and a `v-for`'s source and aliases.

use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};
use vize_s0::{CompactString, FxHashSet};

/// How the `defineProps(...)` return value is consumed.
pub(super) enum PropsAccess {
    /// A bare `defineProps(...)` statement. The props are reachable only from
    /// the template (and from an Options API sibling block through `this`), so
    /// a name referenced nowhere is genuinely unused.
    Discarded,
    /// `const { msg } = defineProps(...)`. Each destructured name becomes a
    /// script binding this cannot follow, so those are left alone; a prop the
    /// pattern does *not* name is still only reachable from the template.
    Destructured,
    /// `const props = defineProps(...)`, or the call wrapped in another call
    /// (`withDefaults(...)`). The script holds the props object and can index it
    /// in ways no scan can see (`props[key]`), so nothing may be reported.
    Captured,
}

/// Classify how the `defineProps` call at `span` inside `script` is consumed.
///
/// Decided from the text immediately before the call, which is enough to tell
/// the three cases apart and resolves every ambiguity towards
/// [`PropsAccess::Captured`] — the outcome that reports nothing.
pub(super) fn classify_props_access(script: &str, span: (u32, u32)) -> PropsAccess {
    let Some(prefix) = script.get(..span.0 as usize) else {
        return PropsAccess::Captured;
    };
    let prefix = prefix.trim_end();
    match prefix.as_bytes().last() {
        // `const <pattern> = defineProps(...)`.
        Some(b'=') => match prefix[..prefix.len() - 1].trim_end().as_bytes().last() {
            Some(b'}') => PropsAccess::Destructured,
            _ => PropsAccess::Captured,
        },
        // Wrapped in a call, e.g. `withDefaults(defineProps(...), { ... })`.
        Some(b'(') => PropsAccess::Captured,
        // A statement of its own: nothing consumes the return value.
        _ => PropsAccess::Discarded,
    }
}

/// Every identifier-shaped token referenced by a compiled template expression.
pub(super) fn template_references(root: &RootNode<'_>) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    collect_children(&root.children, &mut names, root.source);
    names
}

fn collect_children(
    children: &[TemplateChildNode<'_>],
    names: &mut FxHashSet<CompactString>,
    source: &str,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                for prop in element.props.iter() {
                    if let PropNode::Directive(directive) = prop {
                        // A directive's argument is an expression too when it is
                        // dynamic (`:[key]="x"`); a static one is a plain name
                        // and contributes nothing harmful.
                        for exp in [directive.exp.as_ref(), directive.arg.as_ref()]
                            .into_iter()
                            .flatten()
                        {
                            push_identifier_tokens(expression_source(exp, source), names);
                        }
                    }
                }
                collect_children(&element.children, names, source);
            }
            TemplateChildNode::Interpolation(interpolation) => {
                push_identifier_tokens(expression_source(&interpolation.content, source), names);
            }
            // `v-if` / `v-for` are still plain directives in the parse this
            // reads, so these arms only matter if a transformed AST is ever
            // passed in. Walking them keeps that case correct.
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    if let Some(condition) = branch.condition.as_ref() {
                        push_identifier_tokens(expression_source(condition, source), names);
                    }
                    collect_children(&branch.children, names, source);
                }
            }
            TemplateChildNode::For(for_node) => {
                push_identifier_tokens(expression_source(&for_node.source, source), names);
                collect_children(&for_node.children, names, source);
            }
            _ => {}
        }
    }
}

fn expression_source<'a>(exp: &'a ExpressionNode<'a>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}

/// Push every identifier-shaped token of `source` onto `names`.
pub(super) fn push_identifier_tokens(source: &str, names: &mut FxHashSet<CompactString>) {
    let mut chars = source.char_indices().peekable();
    while let Some((offset, ch)) = chars.next() {
        if !is_identifier_start(ch) {
            continue;
        }
        let mut end = offset + ch.len_utf8();
        while let Some(&(next_offset, next_ch)) = chars.peek() {
            if !is_identifier_char(next_ch) {
                break;
            }
            end = next_offset + next_ch.len_utf8();
            chars.next();
        }
        names.insert(CompactString::new(&source[offset..end]));
    }
}

/// Identifiers may be non-ASCII (`ラベル`, `día`), so scan by character rather
/// than by byte; otherwise such references read as absent.
#[inline]
fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

#[inline]
fn is_identifier_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}
