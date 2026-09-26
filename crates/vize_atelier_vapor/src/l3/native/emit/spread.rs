//! An element with an object or computed key: its attributes, props and the
//! object become one `setDynamicProps` effect over upstream's ordered sources.
//! Props between objects form literal groups; the runtime merges the sources
//! in authored order, so later sources win and `class`/`style` concatenate.

use vize_carton::Vec;

use super::super::{BindingKind, Content, Expr};
use super::Emitter;
use crate::ir::{BlockIRNode, IRProp, MergedPropsSource, OperationNode, SetMergedPropsIRNode};

/// A merged source in authored order: a literal prop, or the object itself.
enum Entry<'a> {
    Prop(Expr<'a>, bool, Expr<'a>, bool),
    Object(Expr<'a>),
}

impl<'a> Emitter<'a, '_> {
    pub(super) fn merged_props(
        &mut self,
        index: usize,
        element: usize,
        block: &mut BlockIRNode<'a>,
    ) {
        // Bindings attach to elements.
        let Some(node) = self.artifact.nodes.get(index) else {
            return self.invariant_broken();
        };
        let Content::Element { ref attributes, .. } = node.content else {
            return self.invariant_broken();
        };
        let mut entries: Vec<'a, (u32, Entry<'a>)> = Vec::from_iter_in(
            attributes.iter().map(|&(name, value, span)| {
                // A valueless attribute is the empty string, as upstream binds it.
                // The value span's start is the authored position among sources.
                (
                    span.0,
                    Entry::Prop(
                        Expr::plain(name),
                        true,
                        Expr::plain(value.unwrap_or("")),
                        true,
                    ),
                )
            }),
            &self.allocator,
        );
        for binding in &node.bindings {
            let entry = match binding.kind {
                BindingKind::Spread => Entry::Object(binding.value),
                BindingKind::Prop => Entry::Prop(
                    binding
                        .dynamic_name
                        .unwrap_or_else(|| Expr::plain(binding.name)),
                    binding.dynamic_name.is_none(),
                    binding.value,
                    false,
                ),
                // Admission lets a spread element bind only props.
                _ => {
                    self.invariant_broken();
                    continue;
                }
            };
            entries.push((binding.position, entry));
        }
        entries.sort_by_key(|(position, _)| *position);
        let mut sources = Vec::new_in(&self.allocator);
        let mut group: Vec<'a, IRProp<'a>> = Vec::new_in(&self.allocator);
        for (_, entry) in entries {
            match entry {
                Entry::Object(value) => {
                    if !group.is_empty() {
                        let full = std::mem::replace(&mut group, Vec::new_in(&self.allocator));
                        sources.push(MergedPropsSource::Group(full));
                    }
                    sources.push(MergedPropsSource::Object(self.expression(value, false)));
                }
                Entry::Prop(name, static_name, value, is_static) => {
                    let value = self.expression(value, is_static);
                    // Only `class`/`style` repeat (checked on admission).
                    if static_name
                        && let Some(prop) = group
                            .iter_mut()
                            .find(|prop| prop.key.is_static && prop.key.content == name.text)
                    {
                        prop.values.push(value);
                        continue;
                    }
                    let mut values = Vec::new_in(&self.allocator);
                    values.push(value);
                    group.push(IRProp::new(
                        self.expression(name, static_name),
                        values,
                        false,
                    ));
                }
            }
        }
        if !group.is_empty() {
            sources.push(MergedPropsSource::Group(group));
        }
        self.effect(
            OperationNode::SetMergedProps(SetMergedPropsIRNode { element, sources }),
            block,
        );
    }
}
