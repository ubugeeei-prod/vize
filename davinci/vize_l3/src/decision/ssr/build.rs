//! Server eligibility attached to the existing shared enter/leave events.

use super::{SsrFacts, SsrPart, SsrRejection, SsrUnsupported};
use crate::decision::policy::BindingOwner;
use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId, side_table::SideTable};
use vize_l2::{
    expr::ExprRef,
    lang::js::NativeSelectedSetup,
    op::{BindingOp, ElementOp, Namespace, Op},
    walk::NodeRef,
};
mod read;

pub(in crate::decision) struct SsrBuilder<'facts, 'owner, 'arena> {
    facts: SsrFacts<'owner, 'arena>,
    frames: Vec<(NodeId, Option<&'owner ElementOp<'arena>>, bool)>,
    roots: usize,
    non_comments: usize,
    root_element: Option<NodeId>,
    setup: Option<&'facts NativeSelectedSetup<'owner, 'arena>>,
    previous_root_text: Option<bool>,
    root_interpolation_seen: bool,
}

impl<'facts, 'owner, 'arena> SsrBuilder<'facts, 'owner, 'arena> {
    pub fn new() -> Self {
        Self {
            facts: SsrFacts {
                parts: Vec::new(),
                unsupported: Vec::new(),
                inherit_attrs: None,
                fragment: false,
                expressions: SideTable::new(),
            },
            frames: Vec::new(),
            roots: 0,
            non_comments: 0,
            root_element: None,
            setup: None,
            previous_root_text: None,
            root_interpolation_seen: false,
        }
    }

    pub fn new_setup(setup: &'facts NativeSelectedSetup<'owner, 'arena>) -> Self {
        Self {
            setup: Some(setup),
            ..Self::new()
        }
    }

    pub fn enter(&mut self, node: NodeId, op: &'owner Op<'arena>, root: bool) {
        if root {
            if self.setup.is_some() {
                let current = match op {
                    Op::Text(_) => Some(false),
                    Op::Interpolation(_) => Some(true),
                    _ => None,
                };
                if matches!((self.previous_root_text, current), (Some(previous), Some(current))
                    if previous || current)
                {
                    self.reject(
                        node,
                        NodeRef::Op(op).span(),
                        SsrUnsupported::RootTextGrouping,
                    );
                }
                self.previous_root_text = current;
                if matches!(op, Op::Interpolation(_)) {
                    if self.root_interpolation_seen {
                        self.reject(node, NodeRef::Op(op).span(), SsrUnsupported::Operation);
                    }
                    self.root_interpolation_seen = true;
                }
            }
            self.roots += 1;
            if !matches!(op, Op::Comment(_)) {
                self.non_comments += 1;
                if matches!(op, Op::Element(_)) {
                    self.root_element = Some(node);
                }
            }
        }
        let span = NodeRef::Op(op).span();
        let (element, void) = match op {
            Op::Element(element) => {
                let void = is_void(element.tag);
                self.element(node, element, void);
                self.facts.parts.push(SsrPart::Open {
                    node,
                    element,
                    void,
                });
                (Some(element.as_ref()), void)
            }
            Op::Text(text) => {
                self.facts.parts.push(SsrPart::Text { node, text });
                (None, false)
            }
            Op::Comment(comment) => {
                let content = comment.content;
                if content.starts_with('>')
                    || content.starts_with("->")
                    || content.ends_with("<!-")
                    || ["<!--", "-->", "--!>"]
                        .iter()
                        .any(|token| content.contains(token))
                {
                    self.reject(node, span, SsrUnsupported::UnsafeComment);
                }
                self.facts.parts.push(SsrPart::Comment { node, comment });
                (None, false)
            }
            Op::Interpolation(interpolation) if root && self.setup.is_some() => {
                let expression = match (self.setup, interpolation.expression) {
                    (Some(setup), ExprRef::Js(expression)) => {
                        read::classify(setup, node, expression)
                    }
                    _ => Err(SsrUnsupported::Expression),
                };
                match expression {
                    Ok(expression) => {
                        self.facts.expressions.insert(node, expression);
                        self.facts.parts.push(SsrPart::Interpolation {
                            node,
                            interpolation,
                        });
                    }
                    Err(reason) => self.reject(node, span, reason),
                }
                (None, false)
            }
            _ => {
                self.reject(node, span, SsrUnsupported::Operation);
                (None, false)
            }
        };
        self.frames.push((node, element, void));
    }

    fn element(&mut self, node: NodeId, element: &ElementOp<'arena>, void: bool) {
        if element.namespace != Namespace::Html {
            self.reject(node, element.span, SsrUnsupported::Namespace);
        }
        let mut tag = element.tag.bytes();
        if !tag.next().is_some_and(|byte| byte.is_ascii_lowercase())
            || !tag.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            self.reject(node, element.span, SsrUnsupported::ElementName);
        }
        if matches!(
            element.tag,
            "script" | "style" | "textarea" | "template" | "slot" | "pre" | "search"
        ) {
            self.reject(node, element.span, SsrUnsupported::ElementSemantics);
        }
        if void && !element.children.ops.is_empty() {
            self.reject(node, element.span, SsrUnsupported::VoidChildren);
        }
        let mut names = Vec::new();
        for attribute in &element.attributes {
            let name = attribute.name;
            if name.is_empty()
                || !name.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_' | b':' | b'.')
                })
            {
                self.reject(node, attribute.span, SsrUnsupported::AttributeName);
            }
            if matches!(
                name,
                "class"
                    | "style"
                    | "key"
                    | "ref"
                    | "is"
                    | "slot"
                    | "value"
                    | "__proto__"
                    | "true-value"
                    | "false-value"
            ) || name.starts_with("on")
                || name.starts_with("v-")
                || name.starts_with(':')
                || name.starts_with('.')
            {
                self.reject(node, attribute.span, SsrUnsupported::AttributeSemantics);
            }
            if names.contains(&name) {
                self.reject(node, attribute.span, SsrUnsupported::DuplicateAttribute);
            }
            names.push(name);
        }
    }

    pub fn binding(&mut self, node: NodeId, binding: &BindingOp<'arena>, owner: BindingOwner) {
        // SSR has no event listeners on native HTML; cloak is compile-time only.
        if owner == BindingOwner::Element
            && matches!(binding, BindingOp::On(_) | BindingOp::VueCloak(_))
        {
            return;
        }
        self.reject(
            node,
            NodeRef::Binding(binding).span(),
            SsrUnsupported::Binding,
        );
    }

    pub fn leave(&mut self, _node: NodeId) {
        if let Some((node, Some(element), false)) = self.frames.pop() {
            self.facts.parts.push(SsrPart::Close { node, element });
        }
    }

    fn reject(&mut self, node: NodeId, span: Span, reason: SsrUnsupported) {
        self.facts
            .unsupported
            .push(SsrRejection { node, span, reason });
    }

    pub fn finish(mut self) -> SsrFacts<'owner, 'arena> {
        self.facts.fragment = self.roots > 1;
        self.facts.inherit_attrs = (self.non_comments == 1)
            .then_some(self.root_element)
            .flatten();
        self.facts
    }
}

fn is_void(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
