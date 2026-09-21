//! Adjacent S3 text ops coalesce into one DOM text node, as HTML parsing does.

use vize_carton::{String, Vec};

use super::super::Content;
use super::{Emitter, escape};
use crate::ir::{BlockIRNode, OperationNode, SetTextIRNode};

impl<'a> Emitter<'a, '_> {
    /// Emit the run starting at `start`; returns the cursor after it.
    pub(super) fn text_run(
        &mut self,
        index: usize,
        start: usize,
        parent: Option<usize>,
        offset: usize,
        template: &mut String,
        block: &mut BlockIRNode<'a>,
    ) -> usize {
        let children = &self.artifact.nodes[index].children;
        let end = children[start..]
            .iter()
            .position(|child| !matches!(self.artifact.nodes[*child].content, Content::Text { .. }))
            .map_or(children.len(), |length| start + length);
        let dynamic = children[start..end].iter().any(|child| {
            matches!(
                self.artifact.nodes[*child].content,
                Content::Text { dynamic: true, .. }
            )
        });
        // HTML parsing coalesces adjacent text. One S3 text run must own one
        // DOM address, even when it contains many expressions.
        let mut values = Vec::new_in(&self.allocator);
        for position in start..end {
            let child = self.artifact.nodes[index].children[position];
            let Content::Text { ref parts, .. } = self.artifact.nodes[child].content else {
                unreachable!("text run checked above")
            };
            for part in parts {
                if dynamic {
                    values.push(self.expression(part.value, !part.dynamic));
                }
                if part.dynamic {
                    template.push(' ');
                } else {
                    escape(template, part.value);
                }
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
