//! Text in the retained lane's shapes. Inside an element, adjacent parts form
//! one run with one DOM address. At a block root, a pure mixed run becomes one
//! standalone text node; otherwise each part is its own node.

use vize_atelier_core::SimpleExpressionNode;
use vize_carton::{Box, String, Vec};

use super::super::{Content, TextPart};
use super::{Emitter, escape};
use crate::ir::{BlockIRNode, OperationNode, SetTextIRNode};

type Values<'a> = Vec<'a, Box<'a, SimpleExpressionNode<'a>>>;

impl<'a> Emitter<'a, '_> {
    fn parts(&self, index: usize) -> &[TextPart<'a>] {
        let Content::Text { ref parts, .. } = self.artifact.nodes[index].content else {
            unreachable!("text payload checked by the caller")
        };
        parts
    }

    /// A block of only text with at least two parts and one expression is
    /// one text node; each further part still consumes an id.
    pub(super) fn combined_text(
        &mut self,
        children: &[usize],
        block: &mut BlockIRNode<'a>,
    ) -> bool {
        if !children
            .iter()
            .all(|child| matches!(self.artifact.nodes[*child].content, Content::Text { .. }))
        {
            return false;
        }
        let parts: std::vec::Vec<TextPart<'a>> = children
            .iter()
            .flat_map(|child| self.parts(*child).iter().copied())
            .collect();
        if parts.len() < 2 || !parts.iter().any(|part| part.dynamic) {
            return false;
        }
        let id = self.id();
        for _ in 1..parts.len() {
            self.id();
        }
        self.register(id, " ");
        self.ir.standalone_text_elements.insert(id);
        let mut values = Vec::new_in(&self.allocator);
        for part in &parts {
            values.push(self.expression(part.value, !part.dynamic));
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
        let parts: std::vec::Vec<TextPart<'a>> = self.parts(index).to_vec();
        for part in parts {
            let id = self.id();
            if part.dynamic {
                self.register(id, " ");
                self.ir.standalone_text_elements.insert(id);
                let values = self.values(part.value);
                self.effect(
                    OperationNode::SetText(SetTextIRNode {
                        element: id,
                        is_element: false,
                        values,
                    }),
                    block,
                );
            } else {
                self.register(id, part.value.text);
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
        template: &mut String,
        block: &mut BlockIRNode<'a>,
    ) -> usize {
        let end = children[start..]
            .iter()
            .position(|child| !matches!(self.artifact.nodes[*child].content, Content::Text { .. }))
            .map_or(children.len(), |length| start + length);
        let parts: std::vec::Vec<TextPart<'a>> = children[start..end]
            .iter()
            .flat_map(|child| self.parts(*child).iter().copied())
            .collect();
        let dynamic = parts.iter().any(|part| part.dynamic);
        let mut values: Values<'a> = Vec::new_in(&self.allocator);
        for part in &parts {
            if dynamic {
                values.push(self.expression(part.value, !part.dynamic));
            }
            if part.dynamic {
                template.push(' ');
            } else {
                escape(template, part.value.text);
            }
        }
        if dynamic {
            let parent = parent.expect("dynamic ancestry is materialized");
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
}
