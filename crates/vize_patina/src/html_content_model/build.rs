//! Build a render skeleton from a [`MarkupDocument`] — the one projection
//! Patina's rules read, whichever syntax backs it (Vue template, lowered JSX,
//! and the S2 facade P4-7 introduces).

use std::cell::RefCell;

use vize_armature::{Parser, ParserOptions, TemplateSyntaxMode};
use vize_s0::{Allocator, CompactString, Span};

use super::build_helpers::{
    compiler_ns, component_kind, element_node, intrinsic_member_tag, is_dynamic_is, name_span,
    slot_template_name, static_name,
};
use super::build_props::PropRecorder;
use super::facts::Ns;
use super::skeleton::{BoundaryKind, Element, NodeKind, PropFacts, Skeleton};
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupDocument, MarkupElement, MarkupElementKind, MarkupNode};

/// Parse a Vue template **as authored** and build its skeleton.
///
/// The default (`Standard`) template syntax repairs part of the HTML tree
/// construction while parsing (`<p><div>` becomes siblings, table content is
/// foster-parented) and reports the repair as a parse diagnostic; the checker
/// must see the structure the author wrote — the one Vue's compiler renders
/// and the browser then re-parses — so it reads the Vue-compatible `Quirks`
/// parse, which performs no repair.
pub fn authored_skeleton(allocator: &Allocator, source: &str) -> Skeleton {
    authored(allocator, source, false)
}

/// [`authored_skeleton`] with the [`PropFacts`](super::skeleton::PropFacts)
/// composition prunes with.
pub fn composable_skeleton(allocator: &Allocator, source: &str) -> Skeleton {
    authored(allocator, source, true)
}

fn authored(allocator: &Allocator, source: &str, props: bool) -> Skeleton {
    let parser = Parser::with_options_and_template_syntax(
        allocator,
        source,
        ParserOptions::default(),
        TemplateSyntaxMode::Quirks,
    );
    let (root, _errors) = parser.parse();
    build(&MarkupDocument::new(&root, TemplateSyntax::Vue), props)
}

/// Build the skeleton of a markup document.
pub fn skeleton(document: &MarkupDocument<'_>) -> Skeleton {
    build(document, false)
}

fn build(document: &MarkupDocument<'_>, props: bool) -> Skeleton {
    // `walk_tree` takes two callbacks; the builder is shared between them.
    // Capacities sized for a typical template: the builder runs on every
    // linted template, and regrowing these is a measurable share of it.
    let builder = RefCell::new(Builder {
        skeleton: Skeleton {
            nodes: Vec::with_capacity(128),
            props: PropFacts::default(),
        },
        open: Vec::with_capacity(32),
        exits: Vec::with_capacity(32),
        namespaces: Vec::with_capacity(32),
        props: props.then(PropRecorder::default),
    });
    document.walk_tree(
        &mut |element| {
            let mut builder = builder.borrow_mut();
            let first = builder.skeleton.nodes.len() as u32;
            let opened = builder.enter(element);
            let builder = &mut *builder;
            if let Some(props) = builder.props.as_mut() {
                props.enter(&element, first, opened, &builder.skeleton);
            }
            builder.exits.push(opened);
        },
        &mut |_| {
            let mut builder = builder.borrow_mut();
            if let Some(props) = builder.props.as_mut() {
                props.exit();
            }
            builder.namespaces.pop();
            let opened = builder.exits.pop().unwrap_or(0);
            for _ in 0..opened {
                if let Some(index) = builder.open.pop() {
                    builder.skeleton.close(index);
                }
            }
        },
    );
    let builder = builder.into_inner();
    let mut skeleton = builder.skeleton;
    if let Some(props) = builder.props {
        skeleton.props = props.facts;
    }
    skeleton
}

struct Builder {
    skeleton: Skeleton,
    /// Currently open skeleton nodes, innermost last.
    open: Vec<u32>,
    /// Per entered element, how many nodes it opened (the innermost
    /// entries of `open`, closed on exit).
    exits: Vec<u8>,
    /// Per entered element: its compiler namespace, tag, and whether it is an
    /// `annotation-xml` with an HTML encoding.
    namespaces: Vec<(Ns, CompactString, bool)>,
    /// The prop-fact recorder of a composable skeleton.
    props: Option<PropRecorder>,
}

impl Builder {
    /// Open a node; returns the count it adds to the element's `opened`.
    fn push_open(&mut self, kind: NodeKind, span: Span) -> u8 {
        let index = self.skeleton.open(kind, span);
        self.open.push(index);
        1
    }

    fn in_component(&self) -> bool {
        self.open.last().is_some_and(|index| {
            matches!(self.skeleton.node(*index).kind, NodeKind::Component { .. })
        })
    }

    /// Open the nodes an element contributes; returns how many.
    fn enter(&mut self, element: MarkupElement<'_>) -> u8 {
        let range = element.range();
        let span = Span::new(range.start, range.end);
        let tag = element.tag();
        let parent = self
            .namespaces
            .last()
            .map(|(ns, parent_tag, html)| (*ns, parent_tag.as_str(), *html));
        let ns = compiler_ns(tag, parent);
        let encoding_html = tag.eq_ignore_ascii_case("annotation-xml")
            && element
                .static_attribute("encoding")
                .and_then(|attr| attr.value())
                .is_some_and(|value| {
                    value.eq_ignore_ascii_case("text/html")
                        || value.eq_ignore_ascii_case("application/xhtml+xml")
                });
        self.namespaces
            .push((ns, CompactString::new(tag), encoding_html));
        let mut opened = 0u8;
        // A direct child of a component is slot content: named for
        // `<template #name>`, the default slot otherwise.
        if self.in_component() {
            let name = slot_template_name(&element).unwrap_or_else(|| Some("default".into()));
            opened += self.push_open(NodeKind::SlotContent { name }, span);
            if slot_template_name(&element).is_some() {
                self.text_children(&element);
                return opened;
            }
        }
        match element.kind() {
            MarkupElementKind::Template => {}
            MarkupElementKind::Slot => {
                let name = static_name(&element, "name").unwrap_or_else(|| Some("default".into()));
                opened += self.push_open(NodeKind::SlotOutlet { name }, span);
            }
            MarkupElementKind::Component => {
                if let Some(intrinsic) = intrinsic_member_tag(tag) {
                    let node = intrinsic_node(&element, intrinsic, range.start, ns);
                    opened += self.push_open(NodeKind::Element(node), span);
                } else if let Some(kind) = component_kind(&element) {
                    opened += self.push_open(kind, span);
                } else if matches!(tag, "TransitionGroup" | "transition-group") {
                    match static_name(&element, "tag") {
                        Some(Some(group_tag)) => {
                            let mut node =
                                Element::new(group_tag.as_str(), name_span(range.start, tag));
                            node.compiler_ns = ns;
                            opened += self.push_open(NodeKind::Element(node), span);
                        }
                        Some(None) => {
                            opened += self
                                .push_open(NodeKind::Boundary(BoundaryKind::DynamicComponent), span)
                        }
                        None => {}
                    }
                }
            }
            MarkupElementKind::Element if intrinsic_member_tag(tag).is_some() => {
                let intrinsic = intrinsic_member_tag(tag).unwrap_or(tag);
                let node = intrinsic_node(&element, intrinsic, range.start, ns);
                opened += self.push_open(NodeKind::Element(node), span);
            }
            MarkupElementKind::Element => {
                if is_dynamic_is(&element) {
                    let kind = NodeKind::Boundary(BoundaryKind::DynamicComponent);
                    opened += self.push_open(kind, span);
                } else {
                    let mut node = element_node(&element, tag, range.start);
                    node.compiler_ns = ns;
                    opened += self.push_open(NodeKind::Element(node), span);
                    let boundary = match tag {
                        "template" => Some(BoundaryKind::TemplateContents),
                        "noscript" => Some(BoundaryKind::ScriptingDependent),
                        "html" | "head" | "body" | "frameset" => {
                            Some(BoundaryKind::DocumentStructure)
                        }
                        _ => None,
                    };
                    if let Some(boundary) = boundary {
                        opened += self.push_open(NodeKind::Boundary(boundary), span);
                    }
                }
            }
        }
        self.text_children(&element);
        opened
    }
}

/// An intrinsic member component (`<motion.div>`): the element is
/// `intrinsic`; the source names it by its full tag.
fn intrinsic_node(element: &MarkupElement<'_>, intrinsic: &str, start: u32, ns: Ns) -> Element {
    let mut node = element_node(element, intrinsic, start);
    node.tag = CompactString::new(element.tag());
    node.name_span = name_span(start, element.tag());
    node.compiler_ns = ns;
    node
}

impl Builder {
    /// Record an element's non-element children (text, expressions).
    fn text_children(&mut self, element: &MarkupElement<'_>) {
        let in_component = self.in_component();
        element.walk_children(&mut |child| {
            let (kind, range) = match child {
                MarkupNode::Text(text) => (
                    NodeKind::Text {
                        whitespace_only: text
                            .content()
                            .bytes()
                            .all(|byte| matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')),
                    },
                    text.range(),
                ),
                MarkupNode::Interpolation(range)
                | MarkupNode::If(range)
                | MarkupNode::For(range)
                | MarkupNode::Other(range) => (NodeKind::DynamicText, range),
                MarkupNode::Element(_) | MarkupNode::Comment(_) => return,
            };
            let span = Span::new(range.start, range.end);
            if in_component {
                let group = self.skeleton.open(
                    NodeKind::SlotContent {
                        name: Some("default".into()),
                    },
                    span,
                );
                self.skeleton.leaf(kind, span);
                self.skeleton.close(group);
            } else {
                self.skeleton.leaf(kind, span);
            }
        });
    }
}
