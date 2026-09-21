//! Bindings attach to their native element once the tree is known: each
//! name/family binds at most once, a loop body's `:key` moves to its loop, and
//! a static `class` merges into its `:class` exactly as the retained lane does.

use vize_carton::{FxHashMap, FxHashSet};
use vize_s3::op::{OpId, RegionId};

use super::super::{Binding, BindingKind, Content, Node};
use super::Result;
use crate::s3::{AdmissionFailure, LegacyReason};

pub(super) fn bindings<'a>(
    nodes: &mut [Node<'a>],
    indexes: &FxHashMap<OpId, (usize, RegionId)>,
    parents: &[Option<usize>],
    bindings: std::vec::Vec<(OpId, RegionId, Binding<'a>)>,
) -> Result<()> {
    let mut names = FxHashSet::default();
    for (id, (index, _)) in indexes {
        if let Content::Element { attributes, .. } = &nodes[*index].content {
            names.extend(
                attributes
                    .iter()
                    .map(|(name, _)| (*id, BindingKind::Prop, *name)),
            );
        }
    }
    for (target, region, mut binding) in bindings {
        let Some(&(index, target_region)) = indexes.get(&target) else {
            return Err(LegacyReason::Structure.into());
        };
        if !matches!(nodes[index].content, Content::Element { .. }) || region != target_region {
            return Err(AdmissionFailure::Invalid(
                "binding target is outside its native region",
            ));
        }
        if !names.insert((target, binding.kind, binding.name)) {
            binding.merge = static_class(&mut nodes[index], &binding);
            if binding.merge.is_none() {
                return Err(LegacyReason::Binding.into());
            }
        }
        if binding.kind == BindingKind::Prop && binding.name == "key" {
            // Only the body element of an element-carried loop owns a key.
            let owner = parents[index].map(|parent| &mut nodes[parent].content);
            let Some(Content::For(owner)) = owner else {
                return Err(LegacyReason::Binding.into());
            };
            owner.key_prop = Some(binding.value);
            continue;
        }
        // Content directives replace the element's children at runtime.
        if matches!(binding.kind, BindingKind::Html | BindingKind::Text)
            && !nodes[index].children.is_empty()
        {
            return Err(LegacyReason::Structure.into());
        }
        nodes[index].bindings.push(binding);
    }
    Ok(())
}

/// Remove a valued static `class` for its first `:class`; the template then
/// omits it and the dynamic binding renders `[static, dynamic]`.
fn static_class<'a>(node: &mut Node<'a>, binding: &Binding<'a>) -> Option<&'a str> {
    if binding.kind != BindingKind::Prop || binding.name != "class" {
        return None;
    }
    let Content::Element { attributes, .. } = &mut node.content else {
        return None;
    };
    let position = attributes
        .iter()
        .position(|(name, value)| *name == "class" && value.is_some())?;
    attributes.remove(position).1
}
