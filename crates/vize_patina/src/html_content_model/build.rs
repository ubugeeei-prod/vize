//! Build a render skeleton from a [`MarkupDocument`] — the one projection
//! Patina's rules read, whichever syntax backs it (Vue template, lowered JSX,
//! and the S2 facade P4-7 introduces).

use std::cell::RefCell;

use vize_armature::{Parser, ParserOptions, TemplateSyntaxMode};
use vize_s0::{Allocator, CompactString, Span};

use super::facts::Attr;
use super::skeleton::{BoundaryKind, Element, NodeKind, Skeleton};
use crate::ir::TemplateSyntax;
use crate::markup::{
    MarkupBindingKind, MarkupDocument, MarkupElement, MarkupElementKind, MarkupNode,
};

/// Parse a Vue template **as authored** and build its skeleton.
///
/// The default (`Standard`) template syntax repairs part of the HTML tree
/// construction while parsing (`<p><div>` becomes siblings, table content is
/// foster-parented) and reports the repair as a parse diagnostic; the checker
/// must see the structure the author wrote — the one Vue's compiler renders
/// and the browser then re-parses — so it reads the Vue-compatible `Quirks`
/// parse, which performs no repair.
pub fn authored_skeleton(allocator: &Allocator, source: &str) -> Skeleton {
    let parser = Parser::with_options_and_template_syntax(
        allocator,
        source,
        ParserOptions::default(),
        TemplateSyntaxMode::Quirks,
    );
    let (root, _errors) = parser.parse();
    skeleton(&MarkupDocument::new(&root, TemplateSyntax::Vue))
}

/// Build the skeleton of a markup document.
pub fn skeleton(document: &MarkupDocument<'_>) -> Skeleton {
    // `walk_tree` takes two callbacks; the builder is shared between them.
    let builder = RefCell::new(Builder {
        skeleton: Skeleton::default(),
        open: Vec::new(),
        exits: Vec::new(),
    });
    document.walk_tree(
        &mut |element| {
            let mut builder = builder.borrow_mut();
            let opened = builder.enter(element);
            builder.exits.push(opened);
        },
        &mut |_| {
            let mut builder = builder.borrow_mut();
            if let Some(opened) = builder.exits.pop() {
                for index in opened.into_iter().rev() {
                    builder.skeleton.close(index);
                    builder.open.pop();
                }
            }
        },
    );
    builder.into_inner().skeleton
}

struct Builder {
    skeleton: Skeleton,
    /// Currently open skeleton nodes, innermost last.
    open: Vec<u32>,
    /// Per entered element, the nodes it opened (closed on exit).
    exits: Vec<Vec<u32>>,
}

/// Render-in-place built-ins: their children render where they stand.
const TRANSPARENT_BUILTINS: &[&str] = &[
    "Transition",
    "transition",
    "BaseTransition",
    "base-transition",
    "KeepAlive",
    "keep-alive",
    "Suspense",
    "suspense",
];

impl Builder {
    fn push_open(&mut self, kind: NodeKind, span: Span) -> u32 {
        let index = self.skeleton.open(kind, span);
        self.open.push(index);
        index
    }

    fn in_component(&self) -> bool {
        self.open.last().is_some_and(|index| {
            matches!(self.skeleton.node(*index).kind, NodeKind::Component { .. })
        })
    }

    /// Open the nodes an element contributes; returns them, outermost first.
    fn enter(&mut self, element: MarkupElement<'_>) -> Vec<u32> {
        let range = element.range();
        let span = Span::new(range.start, range.end);
        let tag = element.tag();
        let mut opened = Vec::new();
        // A direct child of a component is slot content: named for
        // `<template #name>`, the default slot otherwise.
        if self.in_component() {
            let name = slot_template_name(&element).unwrap_or_else(|| Some("default".into()));
            opened.push(self.push_open(NodeKind::SlotContent { name }, span));
            if slot_template_name(&element).is_some() {
                self.text_children(&element);
                return opened;
            }
        }
        match element.kind() {
            MarkupElementKind::Template => {}
            MarkupElementKind::Slot => {
                let name = static_name(&element, "name").unwrap_or_else(|| Some("default".into()));
                opened.push(self.push_open(NodeKind::SlotOutlet { name }, span));
            }
            MarkupElementKind::Component => {
                if let Some(kind) = component_kind(&element) {
                    opened.push(self.push_open(kind, span));
                } else if matches!(tag, "TransitionGroup" | "transition-group") {
                    match static_name(&element, "tag") {
                        Some(Some(group_tag)) => {
                            let node =
                                Element::new(group_tag.as_str(), name_span(range.start, tag));
                            opened.push(self.push_open(NodeKind::Element(node), span));
                        }
                        Some(None) => {
                            opened.push(self.push_open(
                                NodeKind::Boundary(BoundaryKind::DynamicComponent),
                                span,
                            ))
                        }
                        None => {}
                    }
                }
            }
            MarkupElementKind::Element => {
                if is_dynamic_is(&element) {
                    let kind = NodeKind::Boundary(BoundaryKind::DynamicComponent);
                    opened.push(self.push_open(kind, span));
                } else {
                    let node = element_node(&element, tag, range.start);
                    opened.push(self.push_open(NodeKind::Element(node), span));
                    let boundary = match tag {
                        "template" => Some(BoundaryKind::TemplateContents),
                        "noscript" => Some(BoundaryKind::ScriptingDependent),
                        "html" | "head" | "body" | "frameset" => {
                            Some(BoundaryKind::DocumentStructure)
                        }
                        _ => None,
                    };
                    if let Some(boundary) = boundary {
                        opened.push(self.push_open(NodeKind::Boundary(boundary), span));
                    }
                }
            }
        }
        self.text_children(&element);
        opened
    }

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

fn name_span(start: u32, tag: &str) -> Span {
    Span::new(start + 1, start + 1 + tag.len() as u32)
}

/// The node a component tag opens; `None` for render-in-place built-ins
/// (and `TransitionGroup`, handled by the caller).
fn component_kind(element: &MarkupElement<'_>) -> Option<NodeKind> {
    let tag = element.tag();
    if TRANSPARENT_BUILTINS.contains(&tag) || matches!(tag, "TransitionGroup" | "transition-group")
    {
        return None;
    }
    if matches!(tag, "Teleport" | "teleport") {
        let disabled = element
            .static_attribute("disabled")
            .is_some_and(|attr| attr.value().is_none_or(|value| value != "false"));
        return if disabled {
            None
        } else {
            Some(NodeKind::Boundary(BoundaryKind::Teleport))
        };
    }
    if matches!(tag, "component" | "Component") {
        return Some(NodeKind::Boundary(BoundaryKind::DynamicComponent));
    }
    Some(NodeKind::Component {
        name: CompactString::new(tag),
    })
}

/// `<x is="vue:…">` or a bound `is`: Vue resolves a component at runtime.
fn is_dynamic_is(element: &MarkupElement<'_>) -> bool {
    element.has_bound_attribute("is")
        || element
            .static_attribute("is")
            .and_then(|attr| attr.value())
            .is_some_and(|value| value.starts_with("vue:"))
}

/// A static attribute's value (`Some(Some(v))`), a bound one (`Some(None)`),
/// or absent (`None`).
fn static_name(element: &MarkupElement<'_>, attr: &str) -> Option<Option<CompactString>> {
    if let Some(found) = element.static_attribute(attr) {
        return Some(found.value().map(CompactString::new));
    }
    element.has_bound_attribute(attr).then_some(None)
}

/// `<template #name>` / `<template v-slot:name>`: `Some(Some(name))`,
/// `Some(None)` for a dynamic slot name, `None` when not a slot template.
fn slot_template_name(element: &MarkupElement<'_>) -> Option<Option<CompactString>> {
    if element.kind() != MarkupElementKind::Template {
        return None;
    }
    let mut found = None;
    element.walk_directives(&mut |directive| {
        if directive.name() == "slot" {
            found = Some(match directive.arg_name() {
                None => Some(CompactString::new("default")),
                Some(arg) if arg.starts_with('[') => None,
                Some(arg) => Some(CompactString::new(arg)),
            });
        }
    });
    found
}

fn element_node(element: &MarkupElement<'_>, tag: &str, start: u32) -> Element {
    let mut node = Element::new(tag, name_span(start, tag));
    element.walk_bindings(&mut |binding| {
        let kind = binding.kind();
        let arg = binding.arg_name();
        match (kind, arg) {
            (MarkupBindingKind::Attribute, Some(name)) => {
                if let Some(attr) = attr_of(name, binding.static_value()) {
                    node.attrs.set_yes(attr);
                }
            }
            (MarkupBindingKind::Bind, Some(name)) => {
                if matches!(
                    name,
                    "innerHTML" | "inner-html" | "textContent" | "text-content"
                ) {
                    node.dynamic_content = true;
                } else if let Some(attr) = bound_attr(name) {
                    node.attrs.set_maybe(attr);
                }
            }
            (MarkupBindingKind::Bind, None) => node.attrs.set_all_maybe(),
            (MarkupBindingKind::Custom, Some("html" | "text")) => node.dynamic_content = true,
            _ => {}
        }
    });
    node
}

/// The attribute fact a static attribute establishes.
fn attr_of(name: &str, value: Option<&str>) -> Option<Attr> {
    match name {
        "href" => Some(Attr::Href),
        "controls" => Some(Attr::Controls),
        "usemap" => Some(Attr::Usemap),
        "itemprop" => Some(Attr::Itemprop),
        "tabindex" => Some(Attr::Tabindex),
        "type" => value
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"))
            .then_some(Attr::TypeHidden),
        "color" | "face" | "size" => Some(Attr::FontPresentational),
        "encoding" => value
            .is_some_and(|value| {
                value.eq_ignore_ascii_case("text/html")
                    || value.eq_ignore_ascii_case("application/xhtml+xml")
            })
            .then_some(Attr::EncodingHtml),
        _ => None,
    }
}

/// The attribute fact a bound attribute makes unknown.
fn bound_attr(name: &str) -> Option<Attr> {
    match name {
        "type" => Some(Attr::TypeHidden),
        "encoding" => Some(Attr::EncodingHtml),
        other => attr_of(other, Some("hidden")),
    }
}
