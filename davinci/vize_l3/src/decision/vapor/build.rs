//! Vapor's bounded eligibility joins the existing shared event stream.

use super::{VaporFacts, VaporPart, VaporRejection, VaporRoot, VaporUnsupported};
use alloc::vec::Vec;
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    op::{BindingOp, ElementOp, Namespace, Op},
    walk::NodeRef,
};

pub(in crate::decision) struct VaporBuilder<'owner, 'arena> {
    facts: VaporFacts<'owner, 'arena>,
    frames: Vec<(NodeId, Option<&'owner ElementOp<'arena>>, bool)>,
    non_comments: usize,
    root_element: Option<NodeId>,
}

impl<'owner, 'arena> VaporBuilder<'owner, 'arena> {
    pub fn new() -> Self {
        Self {
            facts: VaporFacts {
                parts: Vec::new(),
                roots: Vec::new(),
                unsupported: Vec::new(),
                inherit_attrs: None,
            },
            frames: Vec::new(),
            non_comments: 0,
            root_element: None,
        }
    }

    pub fn enter(&mut self, node: NodeId, op: &'owner Op<'arena>, root: bool) {
        let span = NodeRef::Op(op).span();
        if root {
            self.facts.roots.push(VaporRoot {
                ordinal: self.facts.roots.len(),
                node,
                span,
                parts: self.facts.parts.len()..self.facts.parts.len(),
            });
            self.non_comments += usize::from(!matches!(op, Op::Comment(_)));
            if matches!(op, Op::Element(_)) {
                self.root_element = Some(node);
            }
        }
        let (element, void) = match op {
            Op::Element(element) => {
                let void = matches!(element.tag, "br" | "hr" | "img");
                self.element(node, element, void);
                self.facts.parts.push(VaporPart::Open {
                    node,
                    element,
                    void,
                });
                (Some(element.as_ref()), void)
            }
            Op::Text(text) => {
                // These decoded bytes are stable under both Vue's default
                // condense policy and the current native preserve policy.
                if text.content.is_empty()
                    || text.content.starts_with(' ')
                    || text.content.ends_with(' ')
                    || text.content.contains("  ")
                    || text
                        .content
                        .chars()
                        .any(|ch| matches!(ch, '\0' | '\t' | '\n' | '\r' | '\u{c}'))
                {
                    self.reject(node, span, VaporUnsupported::TextNormalization);
                }
                self.facts.parts.push(VaporPart::Text { node, text });
                (None, false)
            }
            Op::Comment(comment) => {
                let content = comment.content;
                if content.contains(['\0', '\r'])
                    || content.starts_with('>')
                    || content.starts_with("->")
                    || content.ends_with("<!-")
                    || ["<!--", "-->", "--!>"]
                        .iter()
                        .any(|token| content.contains(token))
                {
                    self.reject(node, span, VaporUnsupported::UnsafeComment);
                }
                self.facts.parts.push(VaporPart::Comment { node, comment });
                (None, false)
            }
            _ => {
                self.reject(node, span, VaporUnsupported::Operation);
                (None, false)
            }
        };
        self.frames.push((node, element, void));
    }

    fn element(&mut self, node: NodeId, element: &ElementOp<'arena>, void: bool) {
        if element.namespace != Namespace::Html {
            self.reject(node, element.span, VaporUnsupported::Namespace);
        }
        // No raw-text, RCDATA, formatting adoption, table/select/p paragraph
        // recovery, custom-element, component or foreign-tree semantics.
        if !matches!(
            element.tag,
            "div"
                | "span"
                | "section"
                | "article"
                | "main"
                | "aside"
                | "header"
                | "footer"
                | "nav"
                | "small"
                | "br"
                | "hr"
                | "img"
        ) {
            self.reject(node, element.span, VaporUnsupported::ElementSemantics);
        }
        if void && !element.children.ops.is_empty() {
            self.reject(node, element.span, VaporUnsupported::VoidChildren);
        }
        let mut names = Vec::new();
        for attribute in &element.attributes {
            let name = attribute.name;
            let generic = matches!(name, "id" | "title" | "role" | "dir" | "lang")
                || (name.starts_with("data-") || name.starts_with("aria-")) && name.len() > 5;
            if !generic
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                || attribute
                    .value
                    .is_some_and(|value| value.contains(['\0', '\r']))
            {
                self.reject(node, attribute.span, VaporUnsupported::AttributeSemantics);
            }
            if names.contains(&name) {
                self.reject(node, attribute.span, VaporUnsupported::DuplicateAttribute);
            }
            names.push(name);
        }
    }

    pub fn binding(&mut self, node: NodeId, binding: &BindingOp<'arena>) {
        self.reject(
            node,
            NodeRef::Binding(binding).span(),
            VaporUnsupported::Binding,
        );
    }

    pub fn leave(&mut self, _node: NodeId) {
        if let Some((node, Some(element), false)) = self.frames.pop() {
            self.facts.parts.push(VaporPart::Close { node, element });
        }
        if self.frames.is_empty() {
            if let Some(root) = self.facts.roots.last_mut() {
                root.parts.end = self.facts.parts.len();
            }
        }
    }

    fn reject(&mut self, node: NodeId, span: Span, reason: VaporUnsupported) {
        self.facts
            .unsupported
            .push(VaporRejection { node, span, reason });
    }

    pub fn finish(mut self) -> VaporFacts<'owner, 'arena> {
        if self.non_comments == 1 {
            self.facts.inherit_attrs = self.root_element;
        }
        self.facts
    }
}
