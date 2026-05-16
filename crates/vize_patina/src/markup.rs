//! Zero-copy markup view used by HTML-oriented lint rules.
//!
//! This keeps parser-specific ASTs internal while exposing a smaller API
//! that can be projected from Vue templates and JSX/TSX.

use crate::ir::{ByteRange, TemplateSyntax};
use oxc_ast::ast::{
    JSXAttribute, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild, JSXElement,
    JSXElementName, JSXExpression, JSXFragment, JSXText, Program,
};
use oxc_ast_visit::{
    walk::{walk_jsx_element, walk_jsx_fragment, walk_program},
    Visit,
};
use oxc_span::Span;
use std::marker::PhantomData;
use vize_carton::String;
use vize_relief::ast::{
    AttributeNode, DirectiveNode, ElementNode, ElementType, ExpressionNode, RootNode,
    SourceLocation, TemplateChildNode, TextNode,
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
#[derive(Clone, Copy)]
pub struct MarkupDocument<'a> {
    inner: MarkupDocumentInner<'a>,
    syntax: TemplateSyntax,
}

impl<'a> MarkupDocument<'a> {
    /// Create a markup document from a parsed template root.
    pub const fn new(root: &'a RootNode<'a>, syntax: TemplateSyntax) -> Self {
        Self {
            inner: MarkupDocumentInner::Relief(root),
            syntax,
        }
    }

    /// Create a markup document from a parsed JSX/TSX program.
    pub const fn from_jsx(program: &'a Program<'a>, syntax: TemplateSyntax, offset: u32) -> Self {
        Self {
            inner: MarkupDocumentInner::Jsx { program, offset },
            syntax,
        }
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
    pub fn walk_directives(&self, visitor: &mut impl FnMut(MarkupDirective<'a>)) {
        if let MarkupElementInner::Relief(node) = self.inner {
            for prop in &node.props {
                if let vize_relief::ast::PropNode::Directive(dir) = prop {
                    visitor(MarkupDirective::from_relief(dir));
                }
            }
        }
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

#[derive(Clone, Copy)]
enum MarkupDirectiveInner<'a> {
    Relief(&'a DirectiveNode<'a>),
}

/// Directive view.
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

    /// Normalized directive name.
    pub fn name(&self) -> &str {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => node.name.as_str(),
        }
    }

    /// Whether the directive name matches.
    pub fn name_eq(&self, expected: &str) -> bool {
        self.name() == expected
    }

    /// Static argument name when available.
    pub fn arg_name(&self) -> Option<&'a str> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => match node.arg.as_ref() {
                Some(ExpressionNode::Simple(simple)) => Some(simple.content.as_str()),
                _ => None,
            },
        }
    }

    /// Whether the directive argument matches.
    pub fn arg_name_eq(&self, expected: &str) -> bool {
        self.arg_name()
            .is_some_and(|arg| arg.eq_ignore_ascii_case(expected))
    }

    /// Directive byte range.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => loc_to_range(&node.loc),
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
                walk_relief_children(&element_children_relief(element), enter, exit);
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
    // SAFETY: pointers are created from nodes borrowed from a parsed OXC program,
    // and the owning program outlives all markup wrappers constructed for it.
    unsafe { &*node }
}

#[inline]
fn jsx_fragment_ref<'a>(node: *const JSXFragment<'a>) -> &'a JSXFragment<'a> {
    // SAFETY: pointers are created from nodes borrowed from a parsed OXC program,
    // and the owning program outlives all markup wrappers constructed for it.
    unsafe { &*node }
}

#[inline]
fn jsx_attribute_ref<'a>(node: *const JSXAttribute<'a>) -> &'a JSXAttribute<'a> {
    // SAFETY: pointers are created from nodes borrowed from a parsed OXC program,
    // and the owning program outlives all markup wrappers constructed for it.
    unsafe { &*node }
}

#[inline]
fn jsx_text_ref<'a>(node: *const JSXText<'a>) -> &'a JSXText<'a> {
    // SAFETY: pointers are created from nodes borrowed from a parsed OXC program,
    // and the owning program outlives all markup wrappers constructed for it.
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

#[inline]
fn span_to_range(span: Span, offset: u32) -> ByteRange {
    ByteRange::new(offset + span.start, offset + span.end)
}

#[inline]
fn loc_to_range(loc: &SourceLocation) -> ByteRange {
    ByteRange::new(loc.start.offset, loc.end.offset)
}
