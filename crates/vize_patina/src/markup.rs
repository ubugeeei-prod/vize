//! Zero-copy, rule-facing markup IR shared by Vue-template and JSX/TSX rules.
//!
//! # Why
//!
//! Patina rules historically receive concrete `vize_relief` template nodes
//! ([`ElementNode`], [`DirectiveNode`], `ForNode`, `IfNode`, …). Those types
//! only exist for Vue templates, so a rule written against them cannot run over
//! JSX/TSX, where bindings are OXC [`JSXAttribute`]s, events are `onClick`-style
//! props, and `v-for` / `v-if` show up structurally as `items.map(...)` and
//! `cond && <x/>` rather than as directives.
//!
//! This module lifts the rule API onto a small, typed, **borrow-based** facade
//! that can be projected from *both* backends without materializing a synthetic
//! AST. Every wrapper is `Copy` and holds either a shared reference into the
//! `vize_relief` arena or a raw pointer into the OXC program (kept alive for the
//! lint pass — see the `*_ref` SAFETY notes). Names and values are returned as
//! `&str` slices that borrow the original source; nothing allocates unless a
//! rule explicitly asks for normalized owned data (e.g.
//! [`MarkupElement::direct_text_content`]).
//!
//! # Shape
//!
//! - [`MarkupDocument`] — document-level entry point over a template root or a
//!   JSX/TSX program, optionally carrying a [`Croquis`](vize_croquis::Croquis)
//!   for semantic / type-aware rules.
//! - [`MarkupElement`] — element / component / fragment / template / slot node.
//! - [`MarkupAttribute`] — a *written* attribute (static or dynamic).
//! - [`MarkupDirective`] — a Vue directive **or** a directive-like JSX attribute
//!   (so `walk_directives` is meaningful on JSX too).
//! - [`MarkupBinding`] — the normalized binding view ([`MarkupBindingKind`]:
//!   plain attribute, `v-bind`, event (`v-on`), `v-model`, custom directive)
//!   that lets one rule reason about bindings across both syntaxes, including
//!   event/model **modifiers**.
//! - [`MarkupConditional`] / [`MarkupList`] — conditional (`v-if`) and list
//!   (`v-for`) scopes.
//! - [`MarkupNode`] — a child node (element, text, interpolation/expression,
//!   conditional/list scope, comment).
//!
//! # Driving rules
//!
//! Implement [`MarkupRule`] (default-empty `enter_*` hooks) and run it with
//! [`MarkupDocumentVisitor`], which projects the hooks from either backend. The
//! visitor threads a [`MarkupContext`] wrapping the existing [`LintContext`], so
//! rules keep using the same diagnostic/fix APIs and all source ranges map back
//! to the original syntax.

use crate::context::LintContext;
use crate::ir::{ByteRange, TemplateSyntax};
use oxc_ast::ast::{
    JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
    JSXElementName, JSXExpression, JSXFragment, JSXText, Program,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_jsx_element, walk_jsx_fragment, walk_program},
};
use oxc_span::Span;
use std::marker::PhantomData;
use vize_carton::String;
use vize_carton::profile;
use vize_croquis::Croquis;
use vize_relief::ast::{
    AttributeNode, DirectiveNode, ElementNode, ElementType, ExpressionNode, ForNode, IfNode,
    PropNode, RootNode, SourceLocation, TemplateChildNode, TextNode,
};

/// High-level classification for a markup element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupElementKind {
    /// Plain HTML element.
    Element,
    /// Framework component.
    Component,
    /// Slot outlet.
    Slot,
    /// Template wrapper.
    Template,
}

#[derive(Clone, Copy)]
enum MarkupDocumentInner<'a> {
    Relief(&'a RootNode<'a>),
    Jsx {
        program: &'a Program<'a>,
        offset: u32,
    },
}

/// Document-level markup view.
///
/// Borrows either a `vize_relief` template root or an OXC JSX/TSX program and
/// optionally a [`Croquis`] for semantic / type-aware rules. `Copy` so it can be
/// passed by value into the [`MarkupDocumentVisitor`].
#[derive(Clone, Copy)]
pub struct MarkupDocument<'a> {
    inner: MarkupDocumentInner<'a>,
    syntax: TemplateSyntax,
    analysis: Option<&'a Croquis>,
}

impl<'a> MarkupDocument<'a> {
    /// Create a markup document from a parsed template root.
    pub const fn new(root: &'a RootNode<'a>, syntax: TemplateSyntax) -> Self {
        Self {
            inner: MarkupDocumentInner::Relief(root),
            syntax,
            analysis: None,
        }
    }

    /// Create a markup document from a parsed JSX/TSX program.
    pub const fn from_jsx(program: &'a Program<'a>, syntax: TemplateSyntax, offset: u32) -> Self {
        Self {
            inner: MarkupDocumentInner::Jsx { program, offset },
            syntax,
            analysis: None,
        }
    }

    /// Attach optional [`Croquis`] semantic analysis to this document.
    ///
    /// Carried through to [`MarkupContext::analysis`] so type-aware and
    /// semantic rules can reach the same analysis the rest of the pipeline saw,
    /// without re-deriving it.
    pub const fn with_analysis(mut self, analysis: &'a Croquis) -> Self {
        self.analysis = Some(analysis);
        self
    }

    /// Semantic analysis attached to this document, if any.
    pub const fn analysis(&self) -> Option<&'a Croquis> {
        self.analysis
    }

    /// Whether this document was projected from a Vue template (`vize_relief`)
    /// rather than from JSX/TSX. Lets rules apply directive-only semantics
    /// (e.g. `<template v-for>` exemptions) only where they make sense.
    pub const fn is_template(&self) -> bool {
        matches!(self.inner, MarkupDocumentInner::Relief(_))
    }

    /// Whether this document was projected from JSX/TSX.
    pub const fn is_jsx(&self) -> bool {
        matches!(self.inner, MarkupDocumentInner::Jsx { .. })
    }

    /// Template syntax used by this document.
    pub const fn syntax(&self) -> TemplateSyntax {
        self.syntax
    }

    /// Walk all concrete elements in tree order.
    pub fn walk_elements(&self, visitor: &mut impl FnMut(MarkupElement<'a>)) {
        self.walk_tree(visitor, &mut |_| {});
    }

    /// Walk the full element tree, calling enter/exit callbacks.
    pub fn walk_tree(
        &self,
        enter: &mut impl FnMut(MarkupElement<'a>),
        exit: &mut impl FnMut(MarkupElement<'a>),
    ) {
        match self.inner {
            MarkupDocumentInner::Relief(root) => walk_relief_children(&root.children, enter, exit),
            MarkupDocumentInner::Jsx { program, offset } => {
                walk_jsx_program(program, offset, enter, exit)
            }
        }
    }

    /// Drive a [`MarkupRule`] over this document through a [`MarkupContext`].
    ///
    /// This is the zero-copy projection visitor: it walks the underlying
    /// `vize_relief` tree or OXC program in source order and fires the rule's
    /// `enter_*` hooks for elements, attributes, directives, bindings,
    /// conditional / list scopes, text, and interpolation/expression nodes —
    /// without ever allocating a synthetic template AST.
    pub fn visit_with<R: MarkupRule + ?Sized>(&self, rule: &R, ctx: &mut MarkupContext<'_, 'a>) {
        MarkupDocumentVisitor::new(rule, ctx).run(self);
    }
}

#[derive(Clone, Copy)]
enum MarkupElementInner<'a> {
    Relief(&'a ElementNode<'a>),
    JsxElement {
        node: *const JSXElement<'a>,
        offset: u32,
    },
    JsxFragment {
        node: *const JSXFragment<'a>,
        offset: u32,
    },
}

/// Wrapper around a concrete element node.
#[derive(Clone, Copy)]
pub struct MarkupElement<'a> {
    inner: MarkupElementInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupElement<'a> {
    /// Create a markup element wrapper from a Vue template node.
    pub const fn new(node: &'a ElementNode<'a>) -> Self {
        Self {
            inner: MarkupElementInner::Relief(node),
            _marker: PhantomData,
        }
    }

    const fn from_jsx_element(node: *const JSXElement<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupElementInner::JsxElement { node, offset },
            _marker: PhantomData,
        }
    }

    const fn from_jsx_fragment(node: *const JSXFragment<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupElementInner::JsxFragment { node, offset },
            _marker: PhantomData,
        }
    }

    /// Tag name.
    pub fn tag(&self) -> &str {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag.as_str(),
            MarkupElementInner::JsxElement { node, .. } => {
                jsx_element_name(&jsx_element_ref(node).opening_element.name)
            }
            MarkupElementInner::JsxFragment { .. } => "",
        }
    }

    /// Element classification.
    pub fn kind(&self) -> MarkupElementKind {
        match self.inner {
            MarkupElementInner::Relief(node) => match node.tag_type {
                ElementType::Element => MarkupElementKind::Element,
                ElementType::Component => MarkupElementKind::Component,
                ElementType::Slot => MarkupElementKind::Slot,
                ElementType::Template => MarkupElementKind::Template,
            },
            MarkupElementInner::JsxElement { node, .. } => {
                jsx_element_kind(&jsx_element_ref(node).opening_element.name)
            }
            MarkupElementInner::JsxFragment { .. } => MarkupElementKind::Template,
        }
    }

    /// Whether this node is a framework component.
    pub fn is_component(&self) -> bool {
        matches!(self.kind(), MarkupElementKind::Component)
    }

    /// Byte range in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupElementInner::Relief(node) => loc_to_range(&node.loc),
            MarkupElementInner::JsxElement { node, offset } => {
                span_to_range(jsx_element_ref(node).span, offset)
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                span_to_range(jsx_fragment_ref(node).span, offset)
            }
        }
    }

    /// Visit direct child nodes.
    pub fn walk_children(&self, visitor: &mut impl FnMut(MarkupNode<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for child in &node.children {
                    visitor(MarkupNode::from_relief_child(child));
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for child in jsx_children(jsx_element_ref(node), offset) {
                    visitor(child);
                }
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                for child in jsx_children(jsx_fragment_ref(node), offset) {
                    visitor(child);
                }
            }
        }
    }

    /// Visit static attributes on this element.
    pub fn walk_attributes(&self, visitor: &mut impl FnMut(MarkupAttribute<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    if let vize_relief::ast::PropNode::Attribute(attr) = prop {
                        visitor(MarkupAttribute::from_relief(attr));
                    }
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = attribute {
                        visitor(MarkupAttribute::from_jsx(&**attr as *const _, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
        }
    }

    /// Visit directives on this element.
    ///
    /// For Vue templates this yields the explicit `v-*` directives. For JSX,
    /// directive-like attributes — events (`onClick`) and dynamic bindings
    /// (`class={…}`) — are projected as directives too, so a rule that reasons
    /// over [`MarkupDirective`] behaves consistently across both backends. Plain
    /// static JSX attributes (`id="x"`) are *not* directives; use
    /// [`Self::walk_attributes`] or [`Self::walk_bindings`] for those.
    pub fn walk_directives(&self, visitor: &mut impl FnMut(MarkupDirective<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    if let PropNode::Directive(dir) = prop {
                        visitor(MarkupDirective::from_relief(dir));
                    }
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = attribute {
                        let attr_ref: &JSXAttribute<'a> = attr;
                        if jsx_attribute_directive_kind(attr_ref).is_some() {
                            visitor(MarkupDirective::from_jsx(attr_ref as *const _, offset));
                        }
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
        }
    }

    /// Visit every *binding* on this element in source order.
    ///
    /// A [`MarkupBinding`] is the normalized, backend-neutral view of anything
    /// written on the opening tag: plain attributes, `v-bind` (including
    /// `:key` shorthand), events (`v-on` / `onClick`), `v-model`, and custom
    /// directives. This is the projection most rules should target, because the
    /// same closure then runs unchanged over Vue templates and JSX/TSX.
    pub fn walk_bindings(&self, visitor: &mut impl FnMut(MarkupBinding<'a>)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    match prop {
                        PropNode::Attribute(attr) => {
                            visitor(MarkupBinding::from_relief_attribute(attr));
                        }
                        PropNode::Directive(dir) => {
                            visitor(MarkupBinding::from_relief_directive(dir));
                        }
                    }
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    if let JSXAttributeItem::Attribute(attr) = attribute {
                        visitor(MarkupBinding::from_jsx(&**attr as *const _, offset));
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
        }
    }

    /// Find the first binding matching a [`MarkupBindingKind`] and argument
    /// name, e.g. a `v-bind:key` / `:key` (`Bind` + `"key"`) or a `@click` /
    /// `onClick` (`On` + `"click"`).
    pub fn binding(&self, kind: MarkupBindingKind, arg: &str) -> Option<MarkupBinding<'a>> {
        let mut found = None;
        self.walk_bindings(&mut |binding| {
            if found.is_none() && binding.kind() == kind && binding.arg_name_eq(arg) {
                found = Some(binding);
            }
        });
        found
    }

    /// Whether this element has a `key` binding (`:key` / `key` / `key={…}`),
    /// the cross-backend version of the `v-for` key check.
    pub fn has_key_binding(&self) -> bool {
        let mut found = false;
        self.walk_bindings(&mut |binding| {
            if binding.is_key() {
                found = true;
            }
        });
        found
    }

    /// Check whether this element matches the given tag name.
    pub fn is_tag(&self, expected: &str) -> bool {
        self.tag().eq_ignore_ascii_case(expected)
    }

    /// Get a static attribute by name.
    pub fn static_attribute(&self, name: &str) -> Option<MarkupAttribute<'a>> {
        let mut matched = None;
        self.walk_attributes(&mut |attr| {
            if matched.is_none() && attr.name_eq(name) && !attr.is_dynamic() {
                matched = Some(attr);
            }
        });
        matched
    }

    /// Check if a named static attribute exists.
    pub fn has_static_attribute(&self, name: &str) -> bool {
        self.static_attribute(name).is_some()
    }

    /// Check if a directive with the given normalized name exists.
    pub fn has_directive(&self, name: &str) -> bool {
        let mut found = false;
        self.walk_directives(&mut |directive| {
            if directive.name_eq(name) {
                found = true;
            }
        });
        found
    }

    /// Check if this element contains a bound attribute for the given arg name.
    pub fn has_bound_attribute(&self, name: &str) -> bool {
        match self.inner {
            MarkupElementInner::Relief(_) => {
                let mut found = false;
                self.walk_directives(&mut |directive| {
                    if directive.name_eq("bind") && directive.arg_name_eq(name) {
                        found = true;
                    }
                });
                found
            }
            MarkupElementInner::JsxElement { .. } => {
                let mut found = false;
                self.walk_attributes(&mut |attr| {
                    if attr.name_eq(name) && attr.is_dynamic() {
                        found = true;
                    }
                });
                found
            }
            MarkupElementInner::JsxFragment { .. } => false,
        }
    }

    /// Concatenate direct text child nodes.
    pub fn direct_text_content(&self) -> String {
        let mut text = String::default();
        self.walk_children(&mut |child| {
            if let MarkupNode::Text(text_node) = child {
                text.push_str(text_node.content());
            }
        });
        text
    }
}

#[derive(Clone, Copy)]
enum MarkupAttributeInner<'a> {
    Relief(&'a AttributeNode),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
}

/// Static attribute view.
#[derive(Clone, Copy)]
pub struct MarkupAttribute<'a> {
    inner: MarkupAttributeInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupAttribute<'a> {
    const fn from_relief(node: &'a AttributeNode) -> Self {
        Self {
            inner: MarkupAttributeInner::Relief(node),
            _marker: PhantomData,
        }
    }

    const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupAttributeInner::Jsx { node, offset },
            _marker: PhantomData,
        }
    }

    /// Attribute name as written in source.
    pub fn name(&self) -> &str {
        match self.inner {
            MarkupAttributeInner::Relief(node) => node.name.as_str(),
            MarkupAttributeInner::Jsx { node, .. } => {
                jsx_attribute_name(&jsx_attribute_ref(node).name)
            }
        }
    }

    /// Whether this attribute matches a normalized HTML attribute name.
    pub fn name_eq(&self, expected: &str) -> bool {
        self.name().eq_ignore_ascii_case(expected)
    }

    /// Attribute value when statically present.
    pub fn value(&self) -> Option<&'a str> {
        match self.inner {
            MarkupAttributeInner::Relief(node) => {
                node.value.as_ref().map(|value| value.content.as_str())
            }
            MarkupAttributeInner::Jsx { node, .. } => {
                match jsx_attribute_ref(node).value.as_ref() {
                    Some(JSXAttributeValue::StringLiteral(value)) => Some(value.value.as_str()),
                    _ => None,
                }
            }
        }
    }

    /// Whether the attribute value is dynamic.
    pub fn is_dynamic(&self) -> bool {
        match self.inner {
            MarkupAttributeInner::Relief(_) => false,
            MarkupAttributeInner::Jsx { node, .. } => matches!(
                jsx_attribute_ref(node).value.as_ref(),
                Some(
                    JSXAttributeValue::ExpressionContainer(_)
                        | JSXAttributeValue::Element(_)
                        | JSXAttributeValue::Fragment(_)
                )
            ),
        }
    }

    /// Attribute byte range.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupAttributeInner::Relief(node) => loc_to_range(&node.loc),
            MarkupAttributeInner::Jsx { node, offset } => {
                span_to_range(jsx_attribute_ref(node).span, offset)
            }
        }
    }
}

/// The normalized class of a [`MarkupBinding`] (and the directive a JSX
/// attribute projects to).
///
/// This is the cross-backend vocabulary rules reason in:
///
/// | Vue template      | JSX/TSX                | kind        |
/// |-------------------|------------------------|-------------|
/// | `id="x"`          | `id="x"`               | [`Attribute`](Self::Attribute) |
/// | `:key`, `v-bind`  | `key={…}`, `class={…}` | [`Bind`](Self::Bind) |
/// | `@click`, `v-on`  | `onClick={…}`          | [`On`](Self::On) |
/// | `v-model`         | (no JSX equivalent)    | [`Model`](Self::Model) |
/// | `v-show`, `v-foo` | (no JSX equivalent)    | [`Custom`](Self::Custom) |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupBindingKind {
    /// A plain static attribute (`id="x"`).
    Attribute,
    /// A `v-bind` / `:` attribute binding, including the `:key` shorthand and
    /// JSX expression-valued attributes.
    Bind,
    /// An event handler (`v-on` / `@` / JSX `onX`).
    On,
    /// A `v-model` two-way binding.
    Model,
    /// Any other Vue directive (`v-show`, `v-html`, custom directives, …).
    Custom,
}

#[derive(Clone, Copy)]
enum MarkupDirectiveInner<'a> {
    Relief(&'a DirectiveNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
}

/// Directive view: a Vue `v-*` directive or a directive-like JSX attribute.
#[derive(Clone, Copy)]
pub struct MarkupDirective<'a> {
    inner: MarkupDirectiveInner<'a>,
}

impl<'a> MarkupDirective<'a> {
    const fn from_relief(node: &'a DirectiveNode<'a>) -> Self {
        Self {
            inner: MarkupDirectiveInner::Relief(node),
        }
    }

    const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupDirectiveInner::Jsx { node, offset },
        }
    }

    /// Normalized directive name without the `v-` prefix (`bind`, `on`, `for`,
    /// `model`, …). For a JSX `onClick` this is `on`; for a dynamic JSX
    /// attribute such as `class={…}` it is `bind`.
    pub fn name(&self) -> &str {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => node.name.as_str(),
            MarkupDirectiveInner::Jsx { node, .. } => {
                match jsx_attribute_directive_kind(jsx_attribute_ref(node)) {
                    Some(MarkupBindingKind::On) => "on",
                    _ => "bind",
                }
            }
        }
    }

    /// Whether the directive name matches.
    pub fn name_eq(&self, expected: &str) -> bool {
        self.name() == expected
    }

    /// The normalized [`MarkupBindingKind`] this directive represents.
    pub fn kind(&self) -> MarkupBindingKind {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => relief_directive_kind(node),
            MarkupDirectiveInner::Jsx { node, .. } => {
                jsx_attribute_directive_kind(jsx_attribute_ref(node))
                    .unwrap_or(MarkupBindingKind::Bind)
            }
        }
    }

    /// Static argument name when available, e.g. `key` for `:key` / `onKey`,
    /// `click` for `@click` / `onClick`.
    pub fn arg_name(&self) -> Option<&'a str> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => match node.arg.as_ref() {
                Some(ExpressionNode::Simple(simple)) => Some(simple.content.as_str()),
                _ => None,
            },
            MarkupDirectiveInner::Jsx { node, .. } => {
                jsx_attribute_arg_name(jsx_attribute_ref(node))
            }
        }
    }

    /// Whether the directive argument matches.
    pub fn arg_name_eq(&self, expected: &str) -> bool {
        self.arg_name()
            .is_some_and(|arg| arg.eq_ignore_ascii_case(expected))
    }

    /// Visit the directive's modifiers (`stop`/`prevent` for `@click.stop`,
    /// `trim` for `v-model.trim`). JSX has no modifier syntax, so this is empty
    /// for JSX-backed directives.
    pub fn walk_modifiers(&self, visitor: &mut impl FnMut(&'a str)) {
        if let MarkupDirectiveInner::Relief(node) = self.inner {
            for modifier in node.modifiers.iter() {
                visitor(modifier.content.as_str());
            }
        }
    }

    /// Whether a modifier with the given name is present.
    pub fn has_modifier(&self, name: &str) -> bool {
        let mut found = false;
        self.walk_modifiers(&mut |modifier| {
            if modifier == name {
                found = true;
            }
        });
        found
    }

    /// Directive byte range in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => loc_to_range(&node.loc),
            MarkupDirectiveInner::Jsx { node, offset } => {
                span_to_range(jsx_attribute_ref(node).span, offset)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum MarkupBindingInner<'a> {
    ReliefAttribute(&'a AttributeNode),
    ReliefDirective(&'a DirectiveNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
}

/// Normalized binding view — anything written on an opening tag.
///
/// Unlike [`MarkupAttribute`] (only *written* attributes) and
/// [`MarkupDirective`] (only directive-like things), a `MarkupBinding`
/// represents **every** prop uniformly and classifies it via
/// [`MarkupBindingKind`]. It is the recommended projection for rules that need
/// to behave identically on Vue templates and JSX/TSX (key checks, event
/// inspection, model inspection, …).
#[derive(Clone, Copy)]
pub struct MarkupBinding<'a> {
    inner: MarkupBindingInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupBinding<'a> {
    const fn from_relief_attribute(node: &'a AttributeNode) -> Self {
        Self {
            inner: MarkupBindingInner::ReliefAttribute(node),
            _marker: PhantomData,
        }
    }

    const fn from_relief_directive(node: &'a DirectiveNode<'a>) -> Self {
        Self {
            inner: MarkupBindingInner::ReliefDirective(node),
            _marker: PhantomData,
        }
    }

    const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupBindingInner::Jsx { node, offset },
            _marker: PhantomData,
        }
    }

    /// The normalized class of this binding.
    pub fn kind(&self) -> MarkupBindingKind {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_) => MarkupBindingKind::Attribute,
            MarkupBindingInner::ReliefDirective(node) => relief_directive_kind(node),
            MarkupBindingInner::Jsx { node, .. } => {
                jsx_attribute_binding_kind(jsx_attribute_ref(node))
            }
        }
    }

    /// The binding's *argument* name — the part a rule usually keys off:
    ///
    /// - `Attribute`: the attribute name (`id`).
    /// - `Bind`: the bound attribute name (`key` for `:key`).
    /// - `On`: the event name (`click` for `@click` / `onClick`).
    /// - `Model`: the model name (`modelValue` default, or `foo` for
    ///   `v-model:foo`).
    /// - `Custom`: the directive name (`show` for `v-show`).
    pub fn arg_name(&self) -> Option<&'a str> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => Some(node.name.as_str()),
            MarkupBindingInner::ReliefDirective(node) => match relief_directive_kind(node) {
                MarkupBindingKind::Custom => Some(node.name.as_str()),
                _ => match node.arg.as_ref() {
                    Some(ExpressionNode::Simple(simple)) => Some(simple.content.as_str()),
                    _ => None,
                },
            },
            MarkupBindingInner::Jsx { node, .. } => {
                let attr = jsx_attribute_ref(node);
                match jsx_attribute_binding_kind(attr) {
                    MarkupBindingKind::On => jsx_attribute_arg_name(attr),
                    _ => Some(jsx_attribute_name(&attr.name)),
                }
            }
        }
    }

    /// Whether the argument name matches (ASCII case-insensitive).
    pub fn arg_name_eq(&self, expected: &str) -> bool {
        self.arg_name()
            .is_some_and(|arg| arg.eq_ignore_ascii_case(expected))
    }

    /// Whether this binding is a `key` (`key="…"`, `:key`, or JSX `key={…}`).
    pub fn is_key(&self) -> bool {
        matches!(
            self.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) && self.arg_name_eq("key")
    }

    /// Whether the binding's value is dynamic (an expression rather than a
    /// static string). Directives and JSX expression containers are dynamic;
    /// plain attributes are static.
    pub fn is_dynamic(&self) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_) => false,
            MarkupBindingInner::ReliefDirective(_) => true,
            MarkupBindingInner::Jsx { node, .. } => matches!(
                jsx_attribute_ref(node).value.as_ref(),
                Some(
                    JSXAttributeValue::ExpressionContainer(_)
                        | JSXAttributeValue::Element(_)
                        | JSXAttributeValue::Fragment(_)
                )
            ),
        }
    }

    /// Static value when present (`alt="x"` → `Some("x")`). Dynamic bindings
    /// return `None`.
    pub fn static_value(&self) -> Option<&'a str> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => {
                node.value.as_ref().map(|value| value.content.as_str())
            }
            MarkupBindingInner::ReliefDirective(_) => None,
            MarkupBindingInner::Jsx { node, .. } => match jsx_attribute_ref(node).value.as_ref() {
                Some(JSXAttributeValue::StringLiteral(value)) => Some(value.value.as_str()),
                _ => None,
            },
        }
    }

    /// The bound *expression* source for a dynamic binding, as written.
    ///
    /// For a `vize_relief` directive (a Vue template, or JSX/TSX lowered to the
    /// shared AST) `:class="'a'"` / `class={'a'}` yields `'a'` — the JS
    /// expression text, quotes included. For a binding projected *directly* from
    /// the OXC AG ([`MarkupDocument::from_jsx`]) this returns `None`, because the
    /// raw expression slice requires the source string the OXC-direct facade
    /// does not carry; expression-shaped rules consume the lowered AST instead.
    /// Plain static attributes and compound expressions also return `None`.
    pub fn expression(&self) -> Option<&'a str> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_) => None,
            MarkupBindingInner::ReliefDirective(node) => simple_expression_text(node.exp.as_ref()),
            MarkupBindingInner::Jsx { .. } => None,
        }
    }

    /// Visit modifiers on this binding (`stop` for `@click.stop`, `trim` for
    /// `v-model.trim`). Empty for plain attributes and for JSX (no modifier
    /// syntax).
    pub fn walk_modifiers(&self, visitor: &mut impl FnMut(&'a str)) {
        if let MarkupBindingInner::ReliefDirective(node) = self.inner {
            for modifier in node.modifiers.iter() {
                visitor(modifier.content.as_str());
            }
        }
    }

    /// Whether a modifier with the given name is present.
    pub fn has_modifier(&self, name: &str) -> bool {
        let mut found = false;
        self.walk_modifiers(&mut |modifier| {
            if modifier == name {
                found = true;
            }
        });
        found
    }

    /// The byte range of this binding in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => loc_to_range(&node.loc),
            MarkupBindingInner::ReliefDirective(node) => loc_to_range(&node.loc),
            MarkupBindingInner::Jsx { node, offset } => {
                span_to_range(jsx_attribute_ref(node).span, offset)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum MarkupTextInner<'a> {
    Relief(&'a TextNode),
    Jsx {
        node: *const JSXText<'a>,
        offset: u32,
    },
}

/// Text node view.
#[derive(Clone, Copy)]
pub struct MarkupText<'a> {
    inner: MarkupTextInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupText<'a> {
    const fn from_relief(node: &'a TextNode) -> Self {
        Self {
            inner: MarkupTextInner::Relief(node),
            _marker: PhantomData,
        }
    }

    const fn from_jsx(node: *const JSXText<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupTextInner::Jsx { node, offset },
            _marker: PhantomData,
        }
    }

    /// Raw text content.
    pub fn content(&self) -> &'a str {
        match self.inner {
            MarkupTextInner::Relief(node) => node.content.as_str(),
            MarkupTextInner::Jsx { node, .. } => jsx_text_ref(node).value.as_str(),
        }
    }

    /// Byte range.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupTextInner::Relief(node) => loc_to_range(&node.loc),
            MarkupTextInner::Jsx { node, offset } => span_to_range(jsx_text_ref(node).span, offset),
        }
    }

    /// Whether the text contains any non-whitespace content.
    pub fn is_significant(&self) -> bool {
        !self.content().trim().is_empty()
    }
}

/// Direct child node view.
#[derive(Clone, Copy)]
pub enum MarkupNode<'a> {
    /// Concrete element.
    Element(MarkupElement<'a>),
    /// Text node.
    Text(MarkupText<'a>),
    /// Comment node.
    Comment(ByteRange),
    /// Interpolation node.
    Interpolation(ByteRange),
    /// `v-if` / branch control-flow node.
    If(ByteRange),
    /// `v-for` control-flow node.
    For(ByteRange),
    /// Any other node that is currently not projected.
    Other(ByteRange),
}

impl<'a> MarkupNode<'a> {
    fn from_relief_child(child: &'a TemplateChildNode<'a>) -> Self {
        match child {
            TemplateChildNode::Element(element) => Self::Element(MarkupElement::new(element)),
            TemplateChildNode::Text(text) => Self::Text(MarkupText::from_relief(text)),
            TemplateChildNode::Comment(comment) => Self::Comment(loc_to_range(&comment.loc)),
            TemplateChildNode::Interpolation(interpolation) => {
                Self::Interpolation(loc_to_range(&interpolation.loc))
            }
            TemplateChildNode::If(if_node) => Self::If(loc_to_range(&if_node.loc)),
            TemplateChildNode::For(for_node) => Self::For(loc_to_range(&for_node.loc)),
            other => Self::Other(loc_to_range(other.loc())),
        }
    }

    fn from_jsx_child(child: &'a JSXChild<'a>, offset: u32) -> Self {
        match child {
            JSXChild::Text(text) => Self::Text(MarkupText::from_jsx(&**text as *const _, offset)),
            JSXChild::Element(element) => Self::Element(MarkupElement::from_jsx_element(
                &**element as *const _,
                offset,
            )),
            JSXChild::Fragment(fragment) => Self::Element(MarkupElement::from_jsx_fragment(
                &**fragment as *const _,
                offset,
            )),
            JSXChild::ExpressionContainer(container) => match &container.expression {
                JSXExpression::EmptyExpression(_) => {
                    Self::Comment(span_to_range(container.span, offset))
                }
                _ => Self::Interpolation(span_to_range(container.span, offset)),
            },
            JSXChild::Spread(spread) => Self::Interpolation(span_to_range(spread.span, offset)),
        }
    }
}

fn walk_relief_children<'a>(
    children: &'a [TemplateChildNode<'a>],
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                let element = MarkupElement::new(element);
                enter(element);
                walk_relief_children(element_children_relief(element), enter, exit);
                exit(element);
            }
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    walk_relief_children(&branch.children, enter, exit);
                }
            }
            TemplateChildNode::For(for_node) => {
                walk_relief_children(&for_node.children, enter, exit);
            }
            _ => {}
        }
    }
}

fn walk_jsx_program<'a>(
    program: &'a Program<'a>,
    offset: u32,
    enter: &mut impl FnMut(MarkupElement<'a>),
    exit: &mut impl FnMut(MarkupElement<'a>),
) {
    struct JsxMarkupWalker<'enter, 'exit, FEnter, FExit> {
        offset: u32,
        enter: &'enter mut FEnter,
        exit: &'exit mut FExit,
    }

    impl<'ast, FEnter, FExit> Visit<'ast> for JsxMarkupWalker<'_, '_, FEnter, FExit>
    where
        FEnter: FnMut(MarkupElement<'ast>),
        FExit: FnMut(MarkupElement<'ast>),
    {
        fn visit_jsx_element(&mut self, it: &JSXElement<'ast>) {
            let element = MarkupElement::from_jsx_element(it as *const _, self.offset);
            (self.enter)(element);
            walk_jsx_element(self, it);
            (self.exit)(element);
        }

        fn visit_jsx_fragment(&mut self, it: &JSXFragment<'ast>) {
            let element = MarkupElement::from_jsx_fragment(it as *const _, self.offset);
            (self.enter)(element);
            walk_jsx_fragment(self, it);
            (self.exit)(element);
        }
    }

    let mut walker = JsxMarkupWalker {
        offset,
        enter,
        exit,
    };
    walk_program(&mut walker, program);
}

fn element_children_relief<'a>(element: MarkupElement<'a>) -> &'a [TemplateChildNode<'a>] {
    match element.inner {
        MarkupElementInner::Relief(node) => &node.children,
        _ => &[],
    }
}

#[inline]
fn jsx_element_ref<'a>(node: *const JSXElement<'a>) -> &'a JSXElement<'a> {
    // SAFETY: the pointer is captured while walking an `oxc_ast::Program`
    // borrowed for the same `'a` lifetime used by the returned markup facade.
    // Markup wrappers are never stored beyond the lint pass that owns the OXC
    // allocator, and the pass is single-threaded, so the pointed JSX node cannot
    // move, be freed, or be mutably aliased while this shared reference exists.
    unsafe { &*node }
}

#[inline]
fn jsx_fragment_ref<'a>(node: *const JSXFragment<'a>) -> &'a JSXFragment<'a> {
    // SAFETY: same OXC-program lifetime invariant as `jsx_element_ref`. Fragment
    // pointers originate from visitor callbacks and are dereferenced only while
    // the source program and allocator are still alive for the current lint pass.
    unsafe { &*node }
}

#[inline]
fn jsx_attribute_ref<'a>(node: *const JSXAttribute<'a>) -> &'a JSXAttribute<'a> {
    // SAFETY: attribute pointers are copied from JSX element attributes during
    // traversal and keep the OXC AST lifetime. The wrapper is read-only and does
    // not outlive the program, so dereferencing avoids a clone without changing
    // aliasing semantics.
    unsafe { &*node }
}

#[inline]
fn jsx_text_ref<'a>(node: *const JSXText<'a>) -> &'a JSXText<'a> {
    // SAFETY: text pointers are borrowed from immutable JSX children owned by the
    // OXC allocator for the current program. The markup adapter only exposes
    // shared reads, and all adapters are dropped before the program is dropped.
    unsafe { &*node }
}

fn jsx_children<'a>(element: &'a impl JsxChildContainer<'a>, offset: u32) -> Vec<MarkupNode<'a>> {
    element
        .jsx_children()
        .iter()
        .map(|child| MarkupNode::from_jsx_child(child, offset))
        .collect()
}

trait JsxChildContainer<'a> {
    fn jsx_children(&self) -> &oxc_allocator::Vec<'a, JSXChild<'a>>;
}

impl<'a> JsxChildContainer<'a> for JSXElement<'a> {
    fn jsx_children(&self) -> &oxc_allocator::Vec<'a, JSXChild<'a>> {
        &self.children
    }
}

impl<'a> JsxChildContainer<'a> for JSXFragment<'a> {
    fn jsx_children(&self) -> &oxc_allocator::Vec<'a, JSXChild<'a>> {
        &self.children
    }
}

#[inline]
fn jsx_element_kind(name: &JSXElementName<'_>) -> MarkupElementKind {
    if jsx_name_is_component(name) {
        MarkupElementKind::Component
    } else {
        MarkupElementKind::Element
    }
}

#[inline]
fn jsx_name_is_component(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(identifier) => !is_intrinsic_html_name(identifier.name.as_str()),
        JSXElementName::IdentifierReference(reference) => {
            !is_intrinsic_html_name(reference.name.as_str())
        }
        JSXElementName::NamespacedName(name) => !is_intrinsic_html_name(name.name.name.as_str()),
        JSXElementName::MemberExpression(_) | JSXElementName::ThisExpression(_) => true,
    }
}

#[inline]
fn is_intrinsic_html_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_lowercase())
}

#[inline]
fn jsx_element_name<'a>(name: &'a JSXElementName<'a>) -> &'a str {
    match name {
        JSXElementName::Identifier(identifier) => identifier.name.as_str(),
        JSXElementName::IdentifierReference(reference) => reference.name.as_str(),
        JSXElementName::NamespacedName(name) => name.name.name.as_str(),
        JSXElementName::MemberExpression(expression) => expression.property.name.as_str(),
        JSXElementName::ThisExpression(_) => "this",
    }
}

#[inline]
fn jsx_attribute_name<'a>(name: &'a JSXAttributeName<'a>) -> &'a str {
    match name {
        JSXAttributeName::Identifier(identifier) => identifier.name.as_str(),
        JSXAttributeName::NamespacedName(name) => name.name.name.as_str(),
    }
}

/// Classify a `vize_relief` directive into a normalized [`MarkupBindingKind`].
#[inline]
fn relief_directive_kind(node: &DirectiveNode<'_>) -> MarkupBindingKind {
    match node.name.as_str() {
        "bind" => MarkupBindingKind::Bind,
        "on" => MarkupBindingKind::On,
        "model" => MarkupBindingKind::Model,
        _ => MarkupBindingKind::Custom,
    }
}

/// The [`MarkupBindingKind`] a JSX attribute projects to.
///
/// JSX has no directive syntax, so the classification is name-driven: a
/// React-style `onClick` is an event ([`MarkupBindingKind::On`]); an
/// expression-valued attribute (`class={…}`) is a [`MarkupBindingKind::Bind`];
/// a plain string attribute (`id="x"`) is a [`MarkupBindingKind::Attribute`].
#[inline]
fn jsx_attribute_binding_kind(attr: &JSXAttribute<'_>) -> MarkupBindingKind {
    let name = jsx_attribute_name(&attr.name);
    if is_jsx_event_handler_name(name) {
        MarkupBindingKind::On
    } else if matches!(
        attr.value.as_ref(),
        Some(
            JSXAttributeValue::ExpressionContainer(_)
                | JSXAttributeValue::Element(_)
                | JSXAttributeValue::Fragment(_)
        )
    ) {
        MarkupBindingKind::Bind
    } else {
        MarkupBindingKind::Attribute
    }
}

/// The directive-like kind for a JSX attribute, or `None` when the attribute is
/// a plain static attribute (and therefore not directive-like).
#[inline]
fn jsx_attribute_directive_kind(attr: &JSXAttribute<'_>) -> Option<MarkupBindingKind> {
    match jsx_attribute_binding_kind(attr) {
        MarkupBindingKind::Attribute => None,
        kind => Some(kind),
    }
}

/// The event name carried by a JSX `onX` handler (`onClick` → `click`).
#[inline]
fn jsx_attribute_arg_name<'a>(attr: &'a JSXAttribute<'a>) -> Option<&'a str> {
    let name = jsx_attribute_name(&attr.name);
    name.strip_prefix("on").filter(|rest| {
        rest.chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
    })
}

/// Whether a JSX attribute name is a React-style event handler (`onX`, where
/// `X` starts uppercase — so `online` is not mistaken for an event).
#[inline]
fn is_jsx_event_handler_name(name: &str) -> bool {
    name.strip_prefix("on").is_some_and(|rest| {
        rest.chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
    })
}

#[inline]
fn span_to_range(span: Span, offset: u32) -> ByteRange {
    ByteRange::new(offset + span.start, offset + span.end)
}

#[inline]
fn loc_to_range(loc: &SourceLocation) -> ByteRange {
    ByteRange::new(loc.start.offset, loc.end.offset)
}

// ===========================================================================
// Scope nodes: conditional (`v-if`) and list (`v-for`).
// ===========================================================================

/// A conditional scope: a Vue `v-if` / `v-else-if` / `v-else` chain.
///
/// JSX conditionals (`cond && <x/>`) are expressions, not markup nodes, so the
/// projection visitor only surfaces this for the template backend; the element
/// it guards is still visited as a normal [`MarkupElement`].
#[derive(Clone, Copy)]
pub struct MarkupConditional<'a> {
    node: &'a IfNode<'a>,
}

impl<'a> MarkupConditional<'a> {
    const fn from_relief(node: &'a IfNode<'a>) -> Self {
        Self { node }
    }

    /// Number of branches in the chain (`v-if` + each `v-else-if` + optional
    /// `v-else`).
    pub fn branch_count(&self) -> usize {
        self.node.branches.len()
    }

    /// Whether the chain has a terminal `v-else` branch (one whose condition is
    /// absent).
    pub fn has_else(&self) -> bool {
        self.node
            .branches
            .iter()
            .any(|branch| branch.condition.is_none())
    }

    /// The byte range of the whole conditional in the original source.
    pub fn range(&self) -> ByteRange {
        loc_to_range(&self.node.loc)
    }
}

/// A list scope: a Vue `v-for`.
///
/// JSX lists (`items.map(...)`) lower structurally rather than as a directive,
/// so the projection visitor surfaces this for the template backend; the
/// repeated element is still visited as a normal [`MarkupElement`], and JSX
/// rules can assert on its [`MarkupElement::has_key_binding`] directly.
#[derive(Clone, Copy)]
pub struct MarkupList<'a> {
    node: &'a ForNode<'a>,
}

impl<'a> MarkupList<'a> {
    const fn from_relief(node: &'a ForNode<'a>) -> Self {
        Self { node }
    }

    /// The source iterable expression text (`items` in `item in items`).
    pub fn source_expression(&self) -> Option<&'a str> {
        match &self.node.source {
            ExpressionNode::Simple(simple) => Some(simple.content.as_str()),
            ExpressionNode::Compound(_) => None,
        }
    }

    /// The value-alias expression text (`item` in `item in items`).
    pub fn value_alias(&self) -> Option<&'a str> {
        simple_expression_text(self.node.value_alias.as_ref())
    }

    /// Visit the direct element children repeated by this list — the elements a
    /// `:key` requirement applies to. Used for the post-transform `v-for` shape
    /// (both lowered JSX `.map()` and a transformed Vue template), where the
    /// repeated element is wrapped by the `ForNode` rather than carrying the
    /// `v-for` directive itself.
    pub fn walk_elements(&self, visitor: &mut impl FnMut(MarkupElement<'a>)) {
        for child in &self.node.children {
            if let TemplateChildNode::Element(element) = child {
                visitor(MarkupElement::new(element));
            }
        }
    }

    /// The byte range of the whole `v-for` node in the original source.
    pub fn range(&self) -> ByteRange {
        loc_to_range(&self.node.loc)
    }
}

#[inline]
fn simple_expression_text<'a>(exp: Option<&'a ExpressionNode<'a>>) -> Option<&'a str> {
    match exp {
        Some(ExpressionNode::Simple(simple)) => Some(simple.content.as_str()),
        _ => None,
    }
}

/// Lifetime-erased accessor for a `vize_relief` directive backing a
/// [`MarkupDirective`], used by the projection visitor to also fire the legacy
/// per-directive hook with the concrete node when one exists.
impl<'a> MarkupDirective<'a> {
    /// The backing `vize_relief` directive, when this directive was projected
    /// from a Vue template (rather than from a JSX attribute).
    pub fn as_relief(&self) -> Option<&'a DirectiveNode<'a>> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => Some(node),
            MarkupDirectiveInner::Jsx { .. } => None,
        }
    }
}

impl<'a> MarkupElement<'a> {
    /// The backing `vize_relief` element, when this element was projected from a
    /// Vue template. Lets a migrating rule fall back to concrete-node helpers
    /// for template-only cases while still sharing one entry point.
    pub fn as_relief(&self) -> Option<&'a ElementNode<'a>> {
        match self.inner {
            MarkupElementInner::Relief(node) => Some(node),
            _ => None,
        }
    }
}

// ===========================================================================
// Rule-facing trait + driving visitor.
// ===========================================================================

/// Context handed to [`MarkupRule`] callbacks.
///
/// Thin wrapper over the existing [`LintContext`] (so rules keep the full
/// diagnostic / fix / semantic API and `ByteRange`-based reporting via
/// [`LintContext::error_at`] and friends) plus the document-level metadata a
/// markup rule typically needs: source syntax, whether the input is a template
/// or JSX, and the optional [`Croquis`].
pub struct MarkupContext<'ctx, 'a> {
    lint: &'ctx mut LintContext<'a>,
    syntax: TemplateSyntax,
    is_template: bool,
    analysis: Option<&'a Croquis>,
}

impl<'ctx, 'a> MarkupContext<'ctx, 'a> {
    /// Build a markup context from a [`LintContext`] and the document being
    /// linted.
    ///
    /// Semantic analysis is taken from the document's [`MarkupDocument::analysis`]
    /// when present; callers that only have analysis on the [`LintContext`]
    /// should attach it to the document with [`MarkupDocument::with_analysis`].
    pub fn new(lint: &'ctx mut LintContext<'a>, document: &MarkupDocument<'a>) -> Self {
        Self {
            syntax: document.syntax(),
            is_template: document.is_template(),
            analysis: document.analysis(),
            lint,
        }
    }

    /// Mutable access to the underlying [`LintContext`], for reporting
    /// diagnostics and reaching the full semantic API.
    #[inline]
    pub fn lint(&mut self) -> &mut LintContext<'a> {
        &mut *self.lint
    }

    /// The document's template syntax.
    #[inline]
    pub fn syntax(&self) -> TemplateSyntax {
        self.syntax
    }

    /// Whether the document is a Vue template (rather than JSX/TSX). Useful for
    /// directive-only semantics that have no JSX analogue.
    #[inline]
    pub fn is_template(&self) -> bool {
        self.is_template
    }

    /// Whether the document is JSX/TSX.
    #[inline]
    pub fn is_jsx(&self) -> bool {
        !self.is_template
    }

    /// The optional [`Croquis`] semantic analysis for this document.
    #[inline]
    pub fn analysis(&self) -> Option<&'a Croquis> {
        self.analysis
    }
}

/// A lint rule expressed against the zero-copy markup IR.
///
/// This is the parallel of [`crate::rule::Rule`] for the unified IR: the same
/// rule object can run over Vue templates *and* JSX/TSX. All hooks default to
/// empty, so a rule overrides only what it needs. Drive a rule with
/// [`MarkupDocument::visit_with`] / [`MarkupDocumentVisitor`].
///
/// Hooks fire in source order during a single depth-first traversal. Reporting
/// uses [`MarkupContext::lint`] + the `*_at` [`LintContext`] helpers so all
/// diagnostics and fixes map back to original syntax via [`ByteRange`].
pub trait MarkupRule {
    /// The rule name, used to set [`LintContext::current_rule`] before each
    /// callback so diagnostics are attributed and rule-level suppression works.
    fn name(&self) -> &'static str;

    /// Called once before traversal begins.
    #[allow(unused_variables)]
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {}

    /// Called on entering each element / component / fragment / template / slot.
    #[allow(unused_variables)]
    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {}

    /// Called on exiting each element.
    #[allow(unused_variables)]
    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {}

    /// Called for every binding on an element (attribute / bind / event / model
    /// / custom directive), in source order.
    #[allow(unused_variables)]
    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
    }

    /// Called for every directive-like binding on an element (Vue `v-*` or a
    /// directive-like JSX attribute). A strict subset of [`Self::enter_binding`]
    /// for rules that only care about directives.
    #[allow(unused_variables)]
    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
    }

    /// Called on entering a conditional (`v-if`) scope (template backend only).
    #[allow(unused_variables)]
    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
    }

    /// Called on entering a list (`v-for`) scope (template backend only).
    #[allow(unused_variables)]
    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {}

    /// Called for each text node.
    #[allow(unused_variables)]
    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {}

    /// Called for each interpolation / expression node (`{{ … }}` or a JSX
    /// `{expr}` child). The range addresses the original source.
    #[allow(unused_variables)]
    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {}
}

/// Projection visitor that drives a [`MarkupRule`] from either backend.
///
/// Walks the underlying `vize_relief` tree or OXC program in source order,
/// firing the rule's hooks. Crucially it never materializes a synthetic
/// template AST: the hooks receive borrow-based facades over the live parser
/// nodes. Profiling spans (`patina.markup.*`) mirror the template visitor so the
/// adapter's overhead stays visible in Patina benchmarks.
pub struct MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R: ?Sized> {
    rule: &'rule R,
    ctx: &'ctx mut MarkupContext<'mc, 'a>,
}

impl<'rule, 'ctx, 'mc, 'a, R: MarkupRule + ?Sized> MarkupDocumentVisitor<'rule, 'ctx, 'mc, 'a, R> {
    fn new(rule: &'rule R, ctx: &'ctx mut MarkupContext<'mc, 'a>) -> Self {
        Self { rule, ctx }
    }

    #[inline]
    fn set_rule(&mut self) {
        self.ctx.lint.current_rule = self.rule.name();
    }

    fn run(&mut self, document: &MarkupDocument<'a>) {
        self.set_rule();
        self.rule.enter_document(self.ctx, document);

        profile!("patina.markup.visit", {
            match document.inner {
                MarkupDocumentInner::Relief(root) => self.visit_relief_children(&root.children),
                MarkupDocumentInner::Jsx { program, offset } => {
                    self.visit_jsx_program(program, offset)
                }
            }
        });
    }

    fn visit_relief_children(&mut self, children: &'a [TemplateChildNode<'a>]) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    self.visit_element(MarkupElement::new(element));
                }
                TemplateChildNode::Text(text) => {
                    self.set_rule();
                    self.rule
                        .enter_text(self.ctx, &MarkupText::from_relief(text));
                }
                TemplateChildNode::Interpolation(interpolation) => {
                    self.set_rule();
                    self.rule
                        .enter_interpolation(self.ctx, loc_to_range(&interpolation.loc));
                }
                TemplateChildNode::If(if_node) => {
                    self.set_rule();
                    self.rule
                        .enter_conditional(self.ctx, &MarkupConditional::from_relief(if_node));
                    for branch in if_node.branches.iter() {
                        self.visit_relief_children(&branch.children);
                    }
                }
                TemplateChildNode::For(for_node) => {
                    self.set_rule();
                    self.rule
                        .enter_list(self.ctx, &MarkupList::from_relief(for_node));
                    self.visit_relief_children(&for_node.children);
                }
                _ => {}
            }
        }
    }

    fn visit_element(&mut self, element: MarkupElement<'a>) {
        self.set_rule();
        self.rule.enter_element(self.ctx, &element);

        element.walk_bindings(&mut |binding| {
            self.ctx.lint.current_rule = self.rule.name();
            self.rule.enter_binding(self.ctx, &element, &binding);
        });
        element.walk_directives(&mut |directive| {
            self.ctx.lint.current_rule = self.rule.name();
            self.rule.enter_directive(self.ctx, &element, &directive);
        });

        // Children. For the template backend `walk_children` already yields the
        // projected scopes; for JSX it yields elements/text/interpolations.
        match element.inner {
            MarkupElementInner::Relief(node) => self.visit_relief_children(&node.children),
            MarkupElementInner::JsxElement { node, offset } => {
                self.visit_jsx_children(jsx_element_ref(node).jsx_children(), offset)
            }
            MarkupElementInner::JsxFragment { node, offset } => {
                self.visit_jsx_children(jsx_fragment_ref(node).jsx_children(), offset)
            }
        }

        self.set_rule();
        self.rule.exit_element(self.ctx, &element);
    }

    fn visit_jsx_program(&mut self, program: &'a Program<'a>, offset: u32) {
        // OXC's `Visit` borrows `&mut self`, which would conflict with the
        // `&mut MarkupContext` we already hold. Collect the top-level JSX roots
        // first (cheap: pointers only, no AST copy), then drive our own
        // depth-first walk so element enter/exit nest correctly.
        let roots = collect_jsx_roots(program, offset);
        for root in roots {
            self.visit_element(root);
        }
    }

    fn visit_jsx_children(&mut self, children: &'a [JSXChild<'a>], offset: u32) {
        for child in children {
            match MarkupNode::from_jsx_child(child, offset) {
                MarkupNode::Element(element) => self.visit_element(element),
                MarkupNode::Text(text) => {
                    self.set_rule();
                    self.rule.enter_text(self.ctx, &text);
                }
                MarkupNode::Interpolation(range) => {
                    self.set_rule();
                    self.rule.enter_interpolation(self.ctx, range);
                }
                _ => {}
            }
        }
    }
}

/// Collect the outermost JSX elements/fragments in a program in source order,
/// as markup elements. Nested JSX is visited by our own recursion, so this only
/// needs the roots (an `expr && <x/>` guard or a `.map(item => <li/>)` callback
/// still yields its `<x/>` / `<li/>` here as a root we then recurse into).
fn collect_jsx_roots<'a>(program: &'a Program<'a>, offset: u32) -> Vec<MarkupElement<'a>> {
    struct RootCollector<'a> {
        offset: u32,
        roots: Vec<MarkupElement<'a>>,
    }

    impl<'a> Visit<'a> for RootCollector<'a> {
        fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
            self.roots
                .push(MarkupElement::from_jsx_element(it as *const _, self.offset));
            // Do not descend: nested elements are visited by the markup walker.
        }

        fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
            self.roots.push(MarkupElement::from_jsx_fragment(
                it as *const _,
                self.offset,
            ));
        }
    }

    let mut collector = RootCollector {
        offset,
        roots: Vec::new(),
    };
    walk_program(&mut collector, program);
    collector.roots
}

#[cfg(test)]
mod tests {
    //! Cross-backend verification for the rule IR.
    //!
    //! Each test drives a [`MarkupRule`] over a Vue template fixture **and** a
    //! JSX fixture and asserts the diagnostic count, proving one rule body runs
    //! over both backends through the zero-copy facade.

    use super::*;
    use crate::context::LintContext;
    use crate::rules::a11y::ImgAlt;
    use crate::rules::vapor::{NoVueLifecycleEvents, PreferStaticClass};
    use crate::rules::vue::RequireVForKey;
    use vize_atelier_jsx::JsxLang;
    use vize_carton::Allocator;

    /// Run a markup rule over a Vue template and return the diagnostic count.
    fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> usize {
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let parser = vize_armature::Parser::new(allocator.as_bump(), source);
        let (root, _errors) = parser.parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

        let mut lint = LintContext::new(&allocator, source, "test.vue");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        lint.diagnostics().len()
    }

    /// Run a markup rule over JSX/TSX **lowered to the shared relief AST**, the
    /// path directive-shaped rules use (so `.map()`/`key={…}` surface as
    /// `v-for`/`:key`). Returns the diagnostic count.
    fn run_over_jsx_lowered<R: MarkupRule>(rule: &R, source: &str) -> usize {
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let lowered = vize_atelier_jsx::lower_source(allocator.as_bump(), source, JsxLang::Jsx);

        let mut total = 0;
        for lowered_root in &lowered.roots {
            let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
            let mut lint = LintContext::new(&allocator, source, "test.jsx");
            let mut ctx = MarkupContext::new(&mut lint, &document);
            document.visit_with(rule, &mut ctx);
            total += lint.diagnostics().len();
        }
        total
    }

    /// Run a markup rule over JSX projected **directly from the OXC AST** (no
    /// relief lowering), the path HTML-shaped rules use. Returns the diagnostic
    /// count.
    fn run_over_jsx_oxc<R: MarkupRule>(rule: &R, source: &str) -> usize {
        let oxc_allocator = oxc_allocator::Allocator::default();
        let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
        let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

        // The lint context still needs an arena; reuse a fresh carton allocator.
        let lint_allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let mut lint = LintContext::new(&lint_allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        lint.diagnostics().len()
    }

    // ---- vue/require-v-for-key (Vue correctness) ----------------------------

    #[test]
    fn require_v_for_key_template() {
        let rule = RequireVForKey;
        assert_eq!(
            run_over_template(
                &rule,
                r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#
            ),
            1,
            "template v-for without :key must report through the IR"
        );
        assert_eq!(
            run_over_template(
                &rule,
                r#"<ul><li v-for="item in items" :key="item.id">{{ item }}</li></ul>"#
            ),
            0,
            "template v-for with :key must be clean"
        );
    }

    #[test]
    fn require_v_for_key_jsx() {
        let rule = RequireVForKey;
        // `.map()` lowers to v-for; missing key must report.
        assert_eq!(
            run_over_jsx_lowered(
                &rule,
                "const L = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;",
            ),
            1,
            "JSX .map() without key must report through the IR"
        );
        // With a key it is clean.
        assert_eq!(
            run_over_jsx_lowered(
                &rule,
                "const L = () => <ul>{items.map((item) => <li key={item.id}>{item}</li>)}</ul>;",
            ),
            0,
            "JSX .map() with key={{…}} must be clean"
        );
    }

    // ---- a11y/img-alt (accessibility / HTML) --------------------------------

    #[test]
    fn img_alt_template() {
        let rule = ImgAlt;
        assert_eq!(
            run_over_template(&rule, r#"<img src="/photo.jpg" />"#),
            1,
            "template <img> without alt must warn through the IR"
        );
        assert_eq!(
            run_over_template(&rule, r#"<img src="/photo.jpg" alt="Team photo" />"#),
            0,
            "template <img> with alt must be clean"
        );
        assert_eq!(
            run_over_template(&rule, r#"<img :src="photo" :alt="caption" />"#),
            0,
            "template <img> with dynamic :alt must be clean"
        );
    }

    #[test]
    fn img_alt_jsx_oxc() {
        let rule = ImgAlt;
        // Projected straight from the OXC AST — no synthetic template AST.
        assert_eq!(
            run_over_jsx_oxc(&rule, "const I = () => <img src=\"/photo.jpg\" />;"),
            1,
            "JSX <img> without alt must warn through the OXC IR path"
        );
        assert_eq!(
            run_over_jsx_oxc(
                &rule,
                "const I = () => <img src=\"/photo.jpg\" alt=\"Team\" />;"
            ),
            0,
            "JSX <img> with static alt must be clean"
        );
        assert_eq!(
            run_over_jsx_oxc(&rule, "const I = () => <img src={photo} alt={caption} />;"),
            0,
            "JSX <img> with dynamic alt={{…}} must be clean"
        );
    }

    // ---- vapor/prefer-static-class (Vapor) ----------------------------------

    #[test]
    fn prefer_static_class_template() {
        let rule = PreferStaticClass;
        assert_eq!(
            run_over_template(&rule, r#"<div :class="'static'"></div>"#),
            1,
            "template :class with a string literal must warn through the IR"
        );
        assert_eq!(
            run_over_template(&rule, r#"<div :class="dynamic"></div>"#),
            0,
            "template :class with a real expression must be clean"
        );
        assert_eq!(
            run_over_template(&rule, r#"<div class="static"></div>"#),
            0,
            "template static class must be clean"
        );
    }

    #[test]
    fn prefer_static_class_jsx() {
        let rule = PreferStaticClass;
        // `class={'static'}` lowers to the same `:class="'static'"` string
        // literal a Vue template produces.
        assert_eq!(
            run_over_jsx_lowered(&rule, "const C = () => <div class={'static'} />;"),
            1,
            "JSX class={{'static'}} must warn through the IR"
        );
        assert_eq!(
            run_over_jsx_lowered(&rule, "const C = () => <div class={dynamic} />;"),
            0,
            "JSX class={{dynamic}} must be clean"
        );
    }

    // ---- vapor/no-vue-lifecycle-events (Vapor, template-native bonus) -------

    #[test]
    fn no_vue_lifecycle_events_template() {
        let rule = NoVueLifecycleEvents;
        assert_eq!(
            run_over_template(&rule, r#"<div @vue:mounted="onMounted"></div>"#),
            1,
            "template @vue:mounted must report through the IR"
        );
        assert_eq!(
            run_over_template(&rule, r#"<div @click="onClick"></div>"#),
            0,
            "template @click must be clean"
        );
    }

    // ---- Facade unit coverage ----------------------------------------------

    #[test]
    fn jsx_binding_classification() {
        // `onClick` is an event, `class={…}` is a bind, `id="x"` is a plain
        // attribute, `key={…}` is a key binding.
        let oxc_allocator = oxc_allocator::Allocator::default();
        let source = "const C = () => <li id=\"a\" class={cls} key={k} onClick={f} />;";
        let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
        let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

        let mut kinds = Vec::new();
        let mut has_key = false;
        let mut click_is_event = false;
        document.walk_elements(&mut |element| {
            if element.is_tag("li") {
                has_key = element.has_key_binding();
                element.walk_bindings(&mut |binding| {
                    kinds.push((binding.arg_name().map(str::to_owned), binding.kind()));
                    // `onClick` is an event; its argument matches `click`
                    // case-insensitively (JSX event names are PascalCase).
                    if binding.kind() == MarkupBindingKind::On && binding.arg_name_eq("click") {
                        click_is_event = true;
                    }
                });
            }
        });

        assert!(has_key, "key={{k}} must be detected as a key binding");
        assert!(
            click_is_event,
            "onClick must be an event binding with arg `click`"
        );
        assert!(kinds.contains(&(Some("id".to_owned()), MarkupBindingKind::Attribute)));
        assert!(kinds.contains(&(Some("class".to_owned()), MarkupBindingKind::Bind)));
        assert!(kinds.contains(&(Some("key".to_owned()), MarkupBindingKind::Bind)));
    }

    #[test]
    fn template_event_modifiers_are_exposed() {
        // Modifiers come through the normalized binding view for templates.
        let allocator = Allocator::with_capacity(1024);
        let source = r#"<button @click.stop.prevent="f"></button>"#;
        let parser = vize_armature::Parser::new(allocator.as_bump(), source);
        let (root, _errors) = parser.parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

        let mut modifiers = Vec::new();
        document.walk_elements(&mut |element| {
            element.walk_bindings(&mut |binding| {
                if binding.kind() == MarkupBindingKind::On {
                    binding.walk_modifiers(&mut |m| modifiers.push(m.to_owned()));
                }
            });
        });
        assert_eq!(modifiers, vec!["stop".to_owned(), "prevent".to_owned()]);
    }

    #[test]
    fn diagnostics_map_to_original_source_offsets() {
        // The reported range must fall inside the original source for both
        // backends — this is what makes fixes map back to written syntax.
        let rule = ImgAlt;
        let allocator = Allocator::with_capacity(1024);
        let source = r#"<div><img src="/p.jpg" /></div>"#;
        let parser = vize_armature::Parser::new(allocator.as_bump(), source);
        let (root, _errors) = parser.parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.vue");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(&rule, &mut ctx);

        let diagnostics = lint.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        let diag = &diagnostics[0];
        let img_start = source.find("<img").unwrap() as u32;
        assert_eq!(diag.start, img_start, "range must point at the <img> tag");
        assert!(diag.end <= source.len() as u32);
        assert_eq!(&source[diag.start as usize..diag.end as usize][..4], "<img");
    }
}
