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
//! Inside an expression the scan still collects identifier-shaped tokens, so a
//! string such as `'msg'` counts. These lookalikes are not references:
//!
//! * A member name, except `this.name` and `$props.name`.
//! * A `v-for` alias or `v-slot` binding. It shadows the prop on that element,
//!   including its `:key` and children. The loop source and a pattern default
//!   stay outside that alias.
//! * A static directive argument (`:id`, `:aria-label`). A dynamic argument is
//!   an expression and is scanned.
//! * Script references come from the JavaScript/TypeScript AST, minus the
//!   `defineProps` call itself. Comments, strings and import paths cannot be
//!   mistaken for references.
//!
//! A name that appears only in an HTML comment, a text node or a plain
//! attribute is genuinely not a reference, and Vue never compiles a `v-pre`
//! region.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, IdentifierReference, StaticMemberExpression};
use oxc_ast_visit::{Visit, walk::walk_static_member_expression};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_croquis::drawer::{extract_slot_props, parse_v_for_scope_expression};
use vize_l0::{CompactString, FxHashSet};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

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
    collect_children(
        &root.children,
        &mut names,
        root.source,
        &FxHashSet::default(),
    );
    names
}

fn collect_children(
    children: &[TemplateChildNode<'_>],
    names: &mut FxHashSet<CompactString>,
    source: &str,
    shadowed: &FxHashSet<CompactString>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let mut local = shadowed.clone();
                for prop in element.props.iter() {
                    let PropNode::Directive(directive) = prop else {
                        continue;
                    };
                    let Some(exp) = directive.exp.as_ref() else {
                        continue;
                    };
                    let content = expression_source(exp, source);
                    if directive.name == "for" {
                        if let Some(parsed) = parse_v_for_scope_expression(content) {
                            push_identifier_tokens(parsed.source.as_str(), names, shadowed);
                            scan_binding_defaults(content, names, shadowed);
                            for alias in parsed.value_bindings {
                                local.insert(alias);
                            }
                            if let Some(alias) = parsed.key_alias {
                                local.insert(alias);
                            }
                            if let Some(alias) = parsed.index_alias {
                                local.insert(alias);
                            }
                        } else {
                            push_identifier_tokens(content, names, shadowed);
                        }
                    } else if directive.name == "slot" {
                        for alias in extract_slot_props(content) {
                            local.insert(alias);
                        }
                        scan_binding_defaults(content, names, shadowed);
                    }
                }
                for prop in element.props.iter() {
                    let PropNode::Directive(directive) = prop else {
                        continue;
                    };
                    if let Some(argument) = directive.arg.as_ref()
                        && !is_static_expression(argument)
                    {
                        push_identifier_tokens(expression_source(argument, source), names, &local);
                    }
                    if matches!(directive.name, "for" | "slot") {
                        continue;
                    }
                    if let Some(exp) = directive.exp.as_ref() {
                        push_identifier_tokens(expression_source(exp, source), names, &local);
                    }
                }
                collect_children(&element.children, names, source, &local);
            }
            TemplateChildNode::Interpolation(interpolation) => {
                push_identifier_tokens(
                    expression_source(&interpolation.content, source),
                    names,
                    shadowed,
                );
            }
            // `v-if` / `v-for` are still plain directives in the parse this
            // reads, so these arms only matter if a transformed AST is ever
            // passed in. Walking them keeps that case correct.
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    if let Some(condition) = branch.condition.as_ref() {
                        push_identifier_tokens(
                            expression_source(condition, source),
                            names,
                            shadowed,
                        );
                    }
                    collect_children(&branch.children, names, source, shadowed);
                }
            }
            TemplateChildNode::For(for_node) => {
                push_identifier_tokens(
                    expression_source(&for_node.source, source),
                    names,
                    shadowed,
                );
                let mut local = shadowed.clone();
                for alias in [
                    for_node.value_alias.as_ref(),
                    for_node.key_alias.as_ref(),
                    for_node.object_index_alias.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    push_identifier_tokens(
                        expression_source(alias, source),
                        &mut local,
                        &FxHashSet::default(),
                    );
                }
                collect_children(&for_node.children, names, source, &local);
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
            push_identifier_tokens(before, names, &FxHashSet::default());
        }
        if let Some(after) = script.get(end as usize..) {
            push_identifier_tokens(after, names, &FxHashSet::default());
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
        // `count.value` names a member of another object. Only `this.msg` and
        // `$props.msg` read a prop.
        if self.outside_excluded(it.property.span.start, it.property.span.end)
            && is_prop_member_object(&it.object)
        {
            self.names
                .insert(CompactString::new(it.property.name.as_str()));
        }
        walk_static_member_expression(self, it);
    }
}

fn is_prop_member_object(object: &Expression<'_>) -> bool {
    match object {
        Expression::ThisExpression(_) => true,
        Expression::Identifier(identifier) => identifier.name == "$props",
        Expression::StaticMemberExpression(member) => {
            member.property.name == "$props"
                && matches!(member.object, Expression::ThisExpression(_))
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            is_prop_member_object(&parenthesized.expression)
        }
        _ => false,
    }
}

/// Push identifier tokens from `source`, skipping `shadowed` names and member
/// names other than `this.name` / `$props.name`.
pub(super) fn push_identifier_tokens(
    source: &str,
    names: &mut FxHashSet<CompactString>,
    shadowed: &FxHashSet<CompactString>,
) {
    let mut chars = source.char_indices().peekable();
    let mut previous: Option<(&str, usize)> = None;
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
        let Some(name) = source.get(offset..end) else {
            continue;
        };
        let member_of = previous.and_then(|(object, object_end)| {
            let between = source.get(object_end..offset)?.trim();
            ((between == "." || between == "?.") && (object == "this" || object == "$props"))
                .then_some(object)
        });
        if member_of.is_some()
            || (!is_member_name(source, previous, offset) && !shadowed.contains(name))
        {
            names.insert(CompactString::new(name));
        }
        previous = Some((name, end));
    }
}

fn is_member_name(source: &str, previous: Option<(&str, usize)>, offset: usize) -> bool {
    let Some((_, object_end)) = previous else {
        return false;
    };
    let Some(between) = source.get(object_end..offset) else {
        return false;
    };
    let between = between.trim();
    between == "." || between == "?."
}

/// Identifiers on the right of `=` inside a `v-for` or `v-slot` pattern are
/// evaluated outside the alias. `==`, `===` and `=>` are not defaults.
fn scan_binding_defaults(
    pattern: &str,
    names: &mut FxHashSet<CompactString>,
    shadowed: &FxHashSet<CompactString>,
) {
    let bytes = pattern.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let Some(byte) = bytes.get(index).copied() else {
            break;
        };
        match byte {
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'=' => {
                let next = bytes.get(index + 1).copied();
                let operator = next == Some(b'=')
                    || next == Some(b'>')
                    || (index > 0
                        && bytes
                            .get(index - 1)
                            .copied()
                            .is_some_and(|byte| matches!(byte, b'!' | b'<' | b'>')));
                if operator {
                    index += if next == Some(b'=') { 2 } else { 1 };
                    if next == Some(b'=') && bytes.get(index).copied() == Some(b'=') {
                        index += 1;
                    }
                    continue;
                }
                let start = index + 1;
                let end = end_of_default(bytes, start);
                if let Some(value) = pattern.get(start..end) {
                    push_identifier_tokens(value, names, shadowed);
                }
                index = end;
            }
            _ => index += 1,
        }
    }
}

fn end_of_default(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    let mut depth = 0i32;
    while index < bytes.len() {
        let Some(byte) = bytes.get(index).copied() else {
            break;
        };
        match byte {
            b'\'' | b'"' | b'`' => index = skip_quoted(bytes, index),
            b'(' | b'[' | b'{' => {
                depth += 1;
                index += 1;
            }
            b')' | b']' | b'}' if depth == 0 => return index,
            b')' | b']' | b'}' => {
                depth -= 1;
                index += 1;
            }
            b',' if depth == 0 => return index,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn skip_quoted(bytes: &[u8], start: usize) -> usize {
    let Some(quote) = bytes.get(start).copied() else {
        return bytes.len();
    };
    let mut index = start + 1;
    while index < bytes.len() {
        let Some(byte) = bytes.get(index).copied() else {
            break;
        };
        if byte == b'\\' {
            index += 2;
            continue;
        }
        if byte == quote {
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

fn is_static_expression(exp: &ExpressionNode<'_>) -> bool {
    matches!(exp, ExpressionNode::Simple(simple) if simple.is_static)
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
