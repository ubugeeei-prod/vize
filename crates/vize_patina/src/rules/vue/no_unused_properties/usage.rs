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
//! * Script references are read from the JavaScript/TypeScript AST, minus the
//!   `defineProps` call itself. Comments, strings and import paths cannot be
//!   mistaken for references.
//!
//! The one place precision is used is the template *AST*: a name that appears
//! only in an HTML comment, a text node or a plain attribute is genuinely not a
//! reference, and Vue never compiles a `v-pre` region. Every position Vue does
//! compile is scanned — a directive's expression *and* its argument, an
//! interpolation's content, and a `v-for`'s source and aliases.

use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk::walk_static_member_expression};
use oxc_parser::Parser;
use oxc_span::SourceType;
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
        Some(b'=') => match prefix
            .strip_suffix('=')
            .unwrap_or_default()
            .trim_end()
            .as_bytes()
            .last()
        {
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

/// Collect actual script references while excluding the prop declaration call.
/// If the script cannot be parsed, keep the older conservative token scan so a
/// syntax error cannot create an unrelated unused-prop diagnostic.
pub(super) fn push_script_references(
    script: &str,
    excluded: Option<(u32, u32)>,
    lang: Option<&str>,
    names: &mut FxHashSet<CompactString>,
) {
    let path = match lang {
        Some("tsx") => "component.tsx",
        Some("jsx") => "component.jsx",
        Some("ts") => "component.ts",
        _ => "component.js",
    };
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        let (start, end) = excluded.unwrap_or((script.len() as u32, script.len() as u32));
        if let Some(before) = script.get(..start as usize) {
            push_identifier_tokens(before, names);
        }
        if let Some(after) = script.get(end as usize..) {
            push_identifier_tokens(after, names);
        }
        return;
    }

    ScriptReferenceVisitor { excluded, names }.visit_program(&parsed.program);
}

struct ScriptReferenceVisitor<'a> {
    excluded: Option<(u32, u32)>,
    names: &'a mut FxHashSet<CompactString>,
}

impl ScriptReferenceVisitor<'_> {
    fn outside_excluded(&self, start: u32, end: u32) -> bool {
        self.excluded.is_none_or(|(excluded_start, excluded_end)| {
            end <= excluded_start || start >= excluded_end
        })
    }
}

impl<'a> Visit<'a> for ScriptReferenceVisitor<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.outside_excluded(it.span.start, it.span.end) {
            self.names.insert(CompactString::new(it.name.as_str()));
        }
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        // `this.msg` in a sibling Options API block can read a prop. Treat
        // static members conservatively, just as the previous token scan did.
        if self.outside_excluded(it.property.span.start, it.property.span.end) {
            self.names
                .insert(CompactString::new(it.property.name.as_str()));
        }
        walk_static_member_expression(self, it);
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
        if let Some(name) = source.get(offset..end) {
            names.insert(CompactString::new(name));
        }
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
