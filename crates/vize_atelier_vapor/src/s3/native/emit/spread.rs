//! An element with a `v-bind` object: its static attributes, `:prop`s and the
//! object become one `setDynamicProps` effect over upstream's ordered sources.
//! Props between objects form literal groups; the runtime merges the sources
//! in authored order, so later sources win and `class`/`style` concatenate.

use vize_carton::Vec;

use super::super::{BindingKind, Content, Expr};
use super::Emitter;
use crate::ir::{BlockIRNode, IRProp, MergedPropsSource, OperationNode, SetMergedPropsIRNode};

/// A merged source in authored order: a literal prop, or the object itself.
enum Entry<'a> {
    Prop(&'a str, Expr<'a>, bool),
    Object(Expr<'a>),
}

impl<'a> Emitter<'a, '_> {
    pub(super) fn merged_props(
        &mut self,
        index: usize,
        element: usize,
        block: &mut BlockIRNode<'a>,
    ) {
        let node = &self.artifact.nodes[index];
        let Content::Element { ref attributes, .. } = node.content else {
            unreachable!("bindings attach to elements")
        };
        let mut entries: std::vec::Vec<(u32, Entry<'a>)> = (attributes.iter())
            .map(|&(name, value, position)| {
                // A valueless attribute is the empty string, as upstream binds it.
                (
                    position,
                    Entry::Prop(name, Expr::plain(value.unwrap_or("")), true),
                )
            })
            .collect();
        for binding in &node.bindings {
            let entry = match binding.kind {
                BindingKind::Spread => Entry::Object(binding.value),
                BindingKind::Prop => Entry::Prop(binding.name, binding.value, false),
                _ => unreachable!("a spread element binds only props"),
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
                Entry::Prop(name, value, is_static) => {
                    let value = self.expression(value, is_static);
                    // Only `class`/`style` repeat (checked on admission).
                    if let Some(prop) = group.iter_mut().find(|prop| prop.key.content == name) {
                        prop.values.push(value);
                        continue;
                    }
                    let mut values = Vec::new_in(&self.allocator);
                    values.push(value);
                    group.push(IRProp {
                        key: self.expression(Expr::plain(name), true),
                        values,
                        is_component: false,
                    });
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
