//! Elements and their template strings. Numbering follows upstream (as pinned
//! by the published fixture corpus): component and outlet children are
//! numbered before their parent element, which is numbered before its other
//! dynamic descendants; those follow in document order.

use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{Vec, ensure_sufficient_stack};

use super::super::Content;
use super::spans::{tag_offset, value_offset};
use super::{Emitter, escaped};
use crate::ir::{BlockIRNode, InsertNodeIRNode, OperationNode};

impl<'a> Emitter<'a, '_> {
    /// An element that starts a template.
    pub(super) fn element(&mut self, index: usize, block: &mut BlockIRNode<'a>) {
        let reserved = self.reserve(index);
        let id = self.id();
        let mut template = EmitDocument::new(self.source.is_some());
        self.node(index, Some(id), reserved, &mut template, block);
        self.register(id, &template);
        block.returns.push(id);
    }

    /// Ids for the element's component and outlet children, in order.
    fn reserve(&mut self, index: usize) -> std::vec::Vec<usize> {
        let count = self.artifact.nodes[index]
            .children
            .iter()
            .filter(|child| {
                matches!(
                    self.artifact.nodes[**child].content,
                    Content::Component { .. } | Content::Outlet { .. }
                )
            })
            .count();
        (0..count).map(|_| self.id()).collect()
    }

    fn node(
        &mut self,
        index: usize,
        id: Option<usize>,
        reserved: std::vec::Vec<usize>,
        template: &mut EmitDocument,
        block: &mut BlockIRNode<'a>,
    ) {
        ensure_sufficient_stack(|| {
            let Content::Element {
                tag,
                tag_span,
                ref attributes,
            } = self.artifact.nodes[index].content
            else {
                unreachable!("templates start at elements")
            };
            template.push_char('<');
            // S3 keeps element and attribute spans; the tokens inside them
            // are located by the HTML syntax of that authored text (P3-9).
            let tag_token = self.token(tag_span, |raw| tag_offset(raw, tag));
            self.link(template, tag, tag_token, tag.len());
            if let Some(scope_id) = self.scope_id {
                template.push_char(' ');
                template.push_str(scope_id);
            }
            for &(name, value, span) in attributes {
                template.push_char(' ');
                let name_token = self.token(span, |raw| raw.starts_with(name).then_some(0));
                self.link(template, name, name_token, name.len());
                if let Some(value) = value {
                    template.push_str("=\"");
                    let value_token = self.token(span, |raw| value_offset(raw, name, value));
                    self.link(template, &escaped(value), value_token, value.len());
                    template.push_char('"');
                }
            }
            template.push_char('>');
            if let Some(id) = id {
                self.bindings(index, id, block);
            }
            self.children(index, id, reserved, template, block);
            if !vize_carton::is_void_tag(tag) {
                template.push_str("</");
                template.push_str(tag);
                template.push_char('>');
            }
        });
    }

    fn children(
        &mut self,
        index: usize,
        parent: Option<usize>,
        reserved: std::vec::Vec<usize>,
        template: &mut EmitDocument,
        block: &mut BlockIRNode<'a>,
    ) {
        let children = self.artifact.nodes[index].children.clone();
        // Upstream's fast-remove flag clears the whole parent, so it is sound
        // only when a loop is the parent's sole child.
        let only_child = children.len() == 1;
        let mut reserved = reserved.into_iter();
        let mut cursor = 0;
        let mut offset = 0;
        while cursor < children.len() {
            let child = children[cursor];
            cursor += 1;
            match self.artifact.nodes[child].content {
                Content::Text { .. } => {
                    cursor = self.text_run(&children, cursor - 1, parent, offset, template, block);
                }
                Content::Element { .. } => {
                    let (id, nested) = if self.dynamic[child] {
                        let parent = parent.expect("dynamic ancestry is materialized");
                        let id = self.child(parent, offset, block);
                        (Some(id), self.reserve(child))
                    } else {
                        (None, std::vec::Vec::new())
                    };
                    self.node(child, id, nested, template, block);
                }
                Content::If { .. } | Content::For(_) => {
                    // The placeholder is the authored insertion position.
                    template.push_str("<!---->");
                    let parent = parent.expect("control-flow parent is materialized");
                    let anchor = self.child(parent, offset, block);
                    self.control(child, Some((parent, anchor, only_child)), block);
                }
                Content::Component { .. } => {
                    template.push_str("<!---->");
                    let parent = parent.expect("component parent is materialized");
                    let id = reserved.next().expect("component numbered by its parent");
                    let anchor = self.child(parent, offset, block);
                    self.component(child, Some(id), Some((parent, anchor)), block);
                }
                Content::Outlet { .. } => {
                    template.push_str("<!---->");
                    let parent = parent.expect("outlet parent is materialized");
                    let id = reserved.next().expect("outlet numbered by its parent");
                    let anchor = self.child(parent, offset, block);
                    self.outlet(child, id, block);
                    let mut elements = Vec::new_in(&self.allocator);
                    elements.push(id);
                    block
                        .operation
                        .push(OperationNode::InsertNode(InsertNodeIRNode {
                            elements,
                            parent,
                            anchor: Some(anchor),
                        }));
                }
            }
            offset += 1;
        }
    }
}
