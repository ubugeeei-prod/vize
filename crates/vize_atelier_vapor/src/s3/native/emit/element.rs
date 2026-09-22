//! Elements and their template strings. Numbering follows upstream (as pinned
//! by the published fixture corpus): component and outlet children are
//! numbered before their parent element, which is numbered before its other
//! dynamic descendants; those follow in document order.

use oxc_allocator::StringBuilder;
use vize_carton::{Vec, ensure_sufficient_stack};

use super::super::{BindingKind, Content};
use super::{Emitter, escape, take};
use crate::ir::{BlockIRNode, InsertNodeIRNode, OperationNode};

impl<'a> Emitter<'a, '_> {
    /// An element that starts a template.
    pub(super) fn element(&mut self, index: usize, block: &mut BlockIRNode<'a>) {
        let reserved = self.reserve(index);
        let id = self.id();
        let mut template = StringBuilder::with_capacity_in(64, self.allocator.as_oxc());
        self.node(index, Some(id), reserved, &mut template, block);
        self.register(id, template.into_str());
        block.returns.push(id);
    }

    /// Ids for the element's component and outlet children, in order: a
    /// consecutive run minted before the element's own id.
    fn reserve(&mut self, index: usize) -> std::ops::Range<usize> {
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
        let first = self.next_id;
        self.next_id += count;
        first..self.next_id
    }

    fn node(
        &mut self,
        index: usize,
        id: Option<usize>,
        reserved: std::ops::Range<usize>,
        template: &mut StringBuilder<'a>,
        block: &mut BlockIRNode<'a>,
    ) {
        ensure_sufficient_stack(|| {
            let Content::Element {
                tag,
                ref attributes,
            } = self.artifact.nodes[index].content
            else {
                unreachable!("templates start at elements")
            };
            template.push('<');
            template.push_str(tag);
            if let Some(scope_id) = self.scope_id {
                template.push(' ');
                template.push_str(scope_id);
            }
            // A `v-bind` object merges the static attributes at runtime.
            let merged = (self.artifact.nodes[index].bindings.iter())
                .any(|binding| binding.kind == BindingKind::Spread);
            for (name, value, _) in attributes.iter().filter(|_| !merged) {
                template.push(' ');
                template.push_str(name);
                if let Some(value) = value {
                    template.push_str("=\"");
                    escape(template, value);
                    template.push('"');
                }
            }
            template.push('>');
            if let Some(id) = id {
                self.bindings(index, id, block);
            }
            self.children(index, id, reserved, template, block);
            if !super::super::validate::admitted_void(tag) {
                template.push_str("</");
                template.push_str(tag);
                template.push('>');
            }
        });
    }

    fn children(
        &mut self,
        index: usize,
        parent: Option<usize>,
        mut reserved: std::ops::Range<usize>,
        template: &mut StringBuilder<'a>,
        block: &mut BlockIRNode<'a>,
    ) {
        // Children are read in place: nothing below reads its parent's list,
        // and the list is restored once every child is emitted.
        let children = take(self.allocator, &mut self.artifact.nodes[index].children);
        // Upstream's fast-remove flag clears the whole parent, so it is sound
        // only when a loop is the parent's sole child.
        let only_child = children.len() == 1;
        let mut cursor = 0;
        let mut offset = 0;
        // The last referenced element child and its offset: a later element
        // sibling is reached from it, as the retained lane and upstream do.
        let mut previous: Option<(usize, usize)> = None;
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
                        let id = match previous {
                            Some((prev_id, prev_offset)) => {
                                self.next(prev_id, offset - prev_offset, block)
                            }
                            None => self.child(parent, offset, block),
                        };
                        previous = Some((id, offset));
                        (Some(id), self.reserve(child))
                    } else {
                        (None, 0..0)
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
        self.artifact.nodes[index].children = children;
    }
}
