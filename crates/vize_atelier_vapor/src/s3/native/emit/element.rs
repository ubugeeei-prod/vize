//! Elements and their template strings. Numbering follows upstream (as pinned
//! by the published fixture corpus): component and outlet children are
//! numbered before their parent element, which is numbered before its other
//! dynamic descendants; those follow in document order.

use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{Vec, ensure_sufficient_stack};

use super::super::{BindingKind, Content};
use super::spans::{tag_offset, value_offset};
use super::{Emitter, escaped, take};
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

    /// Ids for the element's component and outlet children, in order: a
    /// consecutive run minted before the element's own id.
    fn reserve(&mut self, index: usize) -> std::ops::Range<usize> {
        let Some(node) = self.artifact.nodes.get(index) else {
            self.invariant_broken();
            return self.next_id..self.next_id;
        };
        let count = (node.children.iter())
            .filter(|child| {
                self.artifact.nodes.get(**child).is_some_and(|child| {
                    matches!(
                        child.content,
                        Content::Component { .. } | Content::Outlet { .. }
                    )
                })
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
        template: &mut EmitDocument,
        block: &mut BlockIRNode<'a>,
    ) {
        ensure_sufficient_stack(|| {
            // Linking reborrows the emitter, so the open tag is copied out first.
            let open = self.artifact.nodes.get(index).and_then(|node| {
                // Templates start at elements.
                let Content::Element {
                    tag,
                    tag_span,
                    ref attributes,
                } = node.content
                else {
                    return None;
                };
                // A `v-bind` object merges the static attributes at runtime.
                let merged =
                    (node.bindings.iter()).any(|binding| binding.kind == BindingKind::Spread);
                let attributes = if merged {
                    Vec::new_in(&self.allocator)
                } else {
                    Vec::from_iter_in(attributes.iter().copied(), &self.allocator)
                };
                Some((tag, tag_span, attributes))
            });
            let Some((tag, tag_span, attributes)) = open else {
                return self.invariant_broken();
            };
            let once = self.artifact.nodes.get(index).is_some_and(|node| {
                node.bindings
                    .iter()
                    .any(|binding| binding.kind == BindingKind::Once)
            });
            if once {
                self.non_reactive_depth += 1;
            }
            template.push_char('<');
            // S3 keeps element and attribute spans; the tokens inside them
            // are located by the HTML syntax of that authored text (P3-9).
            let tag_token = self.token(tag_span, |raw| tag_offset(raw, tag));
            self.link(template, tag, tag_token, tag.len());
            if let Some(scope_id) = self.scope_id {
                template.push_char(' ');
                template.push_str(scope_id);
            }
            for (name, value, span) in attributes {
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
            if !super::super::validate::admitted_void(tag) {
                template.push_str("</");
                template.push_str(tag);
                template.push_char('>');
            }
            if once {
                self.non_reactive_depth -= 1;
            }
        });
    }

    fn children(
        &mut self,
        index: usize,
        parent: Option<usize>,
        mut reserved: std::ops::Range<usize>,
        template: &mut EmitDocument,
        block: &mut BlockIRNode<'a>,
    ) {
        // Children are read in place: nothing below reads its parent's list,
        // and the list is restored once every child is emitted.
        let Some(node) = self.artifact.nodes.get_mut(index) else {
            return self.invariant_broken();
        };
        let children = take(self.allocator, &mut node.children);
        // Upstream's fast-remove flag clears the whole parent, so it is sound
        // only when a loop is the parent's sole child.
        let only_child = children.len() == 1;
        let mut cursor = 0;
        let mut offset = 0;
        // The last referenced element child and its offset: a later element
        // sibling is reached from it, as the retained lane and upstream do.
        let mut previous: Option<(usize, usize)> = None;
        while let Some(&child) = children.get(cursor) {
            cursor += 1;
            let Some(node) = self.artifact.nodes.get(child) else {
                self.invariant_broken();
                continue;
            };
            match node.content {
                Content::Text { .. } => {
                    cursor = self.text_run(&children, cursor - 1, parent, offset, template, block);
                }
                Content::Element { .. } => {
                    let (id, nested) = if self.dynamic.get(child) == Some(&true) {
                        // Dynamic ancestry is materialized.
                        let Some(parent) = parent else {
                            self.invariant_broken();
                            continue;
                        };
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
                    let Some(parent) = parent else {
                        self.invariant_broken();
                        continue;
                    };
                    let anchor = self.child(parent, offset, block);
                    self.control(child, Some((parent, anchor, only_child)), block);
                }
                Content::Component { .. } => {
                    template.push_str("<!---->");
                    let Some(parent) = parent else {
                        self.invariant_broken();
                        continue;
                    };
                    // The parent numbered it in `reserve`.
                    let Some(id) = reserved.next() else {
                        self.invariant_broken();
                        continue;
                    };
                    let anchor = self.child(parent, offset, block);
                    self.component(child, Some(id), Some((parent, anchor)), block);
                }
                Content::Outlet { .. } => {
                    template.push_str("<!---->");
                    let Some(parent) = parent else {
                        self.invariant_broken();
                        continue;
                    };
                    // The parent numbered it in `reserve`.
                    let Some(id) = reserved.next() else {
                        self.invariant_broken();
                        continue;
                    };
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
        if let Some(node) = self.artifact.nodes.get_mut(index) {
            node.children = children;
        }
    }
}
