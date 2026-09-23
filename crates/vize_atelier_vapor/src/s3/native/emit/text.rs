//! Text in the retained lane's shapes. Inside an element, adjacent parts form
//! one run with one DOM address. At a block root, a pure mixed run becomes one
//! standalone text node; otherwise each part is its own node.

use vize_atelier_core::{SimpleExpressionNode, codegen::document::EmitDocument};
use vize_carton::{Box, Vec};

use super::super::{AuthoredSpan, Content, TextPart};
use super::{Emitter, escaped};
use crate::ir::{BlockIRNode, OperationNode, SetTextIRNode};

type Values<'a> = Vec<'a, Box<'a, SimpleExpressionNode<'a>>>;

impl<'a> Emitter<'a, '_> {
    fn parts(&self, index: usize) -> &[TextPart<'a>] {
        // The caller checked the text payload.
        match self.artifact.nodes.get(index).map(|node| &node.content) {
            Some(Content::Text { parts, .. }) => parts,
            _ => {
                self.invariant_broken();
                &[]
            }
        }
    }

    /// A block of only text with at least two parts and one expression is
    /// one text node; each further part still consumes an id.
    pub(super) fn combined_text(
        &mut self,
        children: &[usize],
        block: &mut BlockIRNode<'a>,
    ) -> bool {
        if !children.iter().all(|child| self.is_text(*child)) {
            return false;
        }
        let parts: Vec<'a, TextPart<'a>> = Vec::from_iter_in(
            (children.iter()).flat_map(|child| self.parts(*child).iter().copied()),
            &self.allocator,
        );
        if parts.len() < 2 || !parts.iter().any(|part| part.dynamic) {
            return false;
        }
        let id = self.id();
        for _ in 1..parts.len() {
            self.id();
        }
        self.register(id, &EmitDocument::plain(" "));
        self.ir.standalone_text_elements.insert(id);
        let mut values = Vec::new_in(&self.allocator);
        for part in &parts {
            values.push(self.spanned(part.value, !part.dynamic, Some(self.text_span(part))));
        }
        self.effect(
            OperationNode::SetText(SetTextIRNode {
                element: id,
                is_element: false,
                values,
            }),
            block,
        );
        block.returns.push(id);
        true
    }

    /// Text parts at a block root: static text is its own template; each
    /// expression is a standalone updated text node.
    pub(super) fn text_roots(&mut self, index: usize, block: &mut BlockIRNode<'a>) {
        let parts: Vec<'a, TextPart<'a>> =
            Vec::from_iter_in(self.parts(index).iter().copied(), &self.allocator);
        for &part in parts.iter() {
            let id = self.id();
            if part.dynamic {
                self.register(id, &EmitDocument::plain(" "));
                self.ir.standalone_text_elements.insert(id);
                let values = self.values(part.value, Some(self.text_span(&part)));
                self.effect(
                    OperationNode::SetText(SetTextIRNode {
                        element: id,
                        is_element: false,
                        values,
                    }),
                    block,
                );
            } else {
                let text = part.value.text;
                let mut template = EmitDocument::new(self.source.is_some());
                self.link(&mut template, text, Some(part.span), text.len());
                self.register(id, &template);
            }
            block.returns.push(id);
        }
    }

    /// The text run inside an element starting at `children[start]`; returns
    /// the cursor after it. HTML parsing coalesces adjacent text, so one run
    /// owns one DOM address however many expressions it holds; a run at the
    /// first position reads the element's first child.
    pub(super) fn text_run(
        &mut self,
        children: &[usize],
        start: usize,
        parent: Option<usize>,
        offset: usize,
        template: &mut EmitDocument,
        block: &mut BlockIRNode<'a>,
    ) -> usize {
        let run = children.get(start..).unwrap_or_default();
        let length = run.iter().take_while(|child| self.is_text(**child)).count();
        let end = start + length;
        let parts: Vec<'a, TextPart<'a>> = Vec::from_iter_in(
            (run.iter().take(length)).flat_map(|child| self.parts(*child).iter().copied()),
            &self.allocator,
        );
        let dynamic = parts.iter().any(|part| part.dynamic);
        let mut values: Values<'a> = Vec::new_in(&self.allocator);
        for part in parts.iter() {
            if dynamic {
                values.push(self.spanned(part.value, !part.dynamic, Some(self.text_span(part))));
            }
            if part.dynamic {
                template.push_char(' ');
            } else {
                let text = part.value.text;
                self.link(template, &escaped(text), Some(part.span), text.len());
            }
        }
        if dynamic {
            // Dynamic ancestry is materialized.
            let Some(parent) = parent else {
                self.invariant_broken();
                return end;
            };
            let element = if offset == 0 {
                parent
            } else {
                let id = self.child(parent, offset, block);
                self.ir.standalone_text_elements.insert(id);
                id
            };
            self.effect(
                OperationNode::SetText(SetTextIRNode {
                    element,
                    is_element: false,
                    values,
                }),
                block,
            );
        }
        end
    }

    fn is_text(&self, index: usize) -> bool {
        (self.artifact.nodes.get(index))
            .is_some_and(|node| matches!(node.content, Content::Text { .. }))
    }

    /// Dynamic compound parts keep the whole `{{ expr }}`; map the expression
    /// at its inner text, which is the loc the legacy lane records.
    fn text_span(&self, part: &TextPart<'a>) -> AuthoredSpan {
        if part.dynamic {
            self.expression_anchor(part.span, part.value.text)
        } else {
            part.span
        }
    }
}
