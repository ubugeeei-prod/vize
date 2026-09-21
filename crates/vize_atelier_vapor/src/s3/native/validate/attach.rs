//! Bindings attach to their native owner once the tree is known: each
//! name/family binds at most once, a loop body's `:key` moves to its loop, a
//! static `class` merges into its `:class` exactly as the retained lane does,
//! and component/outlet bindings become props in authored order.

use vize_carton::FxHashSet;
use vize_s3::op::{OpId, RegionId};

use super::super::{Binding, BindingKind, Content, Node, Prop};
use super::{Result, Slots, component::component_prop};
use crate::s3::{AdmissionFailure, LegacyReason};

type Pending<'a> = (OpId, RegionId, u32, Binding<'a>);

pub(super) fn bindings<'a>(
    nodes: &mut [Node<'a>],
    slots: &Slots,
    parents: &[Option<usize>],
    bindings: std::vec::Vec<Pending<'a>>,
) -> Result<()> {
    // Names already bound per native node: static attributes and props first.
    let mut names = FxHashSet::default();
    for (index, node) in nodes.iter().enumerate() {
        match &node.content {
            Content::Element { attributes, .. } => {
                names.extend((attributes.iter()).map(|(name, _)| (index, BindingKind::Prop, *name)))
            }
            Content::Component { props, .. } | Content::Outlet { props, .. } => {
                names.extend(
                    props
                        .iter()
                        .map(|prop| (index, BindingKind::Prop, prop.key)),
                );
            }
            _ => {}
        }
    }
    for (target, region, position, mut binding) in bindings {
        let Some(&Some((index, target_region))) = slots.get(target.index() as usize) else {
            return Err(LegacyReason::Structure.into());
        };
        if region != target_region {
            return Err(AdmissionFailure::Invalid(
                "binding target is outside its native region",
            ));
        }
        let fresh = names.insert((index, binding.kind, binding.name));
        match &mut nodes[index].content {
            Content::Element { .. } => {}
            Content::Component { props, .. } => {
                prop(props, binding, position, fresh, true)?;
                continue;
            }
            Content::Outlet { props, .. } => {
                prop(props, binding, position, fresh, false)?;
                continue;
            }
            _ => {
                return Err(AdmissionFailure::Invalid(
                    "binding target is outside its native region",
                ));
            }
        }
        if !fresh {
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
    for node in nodes.iter_mut() {
        if let Content::Component { props, .. } | Content::Outlet { props, .. } = &mut node.content
        {
            props.sort_by_key(|prop| prop.position);
        }
    }
    Ok(())
}

/// A component `:prop` or `@event`, or an outlet `:prop`. Repeated names merge
/// only for `class`/`style`, which the shared generator normalizes together.
fn prop<'a>(
    props: &mut std::vec::Vec<Prop<'a>>,
    binding: Binding<'a>,
    position: u32,
    fresh: bool,
    component: bool,
) -> Result<()> {
    let handler = match binding.kind {
        BindingKind::Prop => false,
        BindingKind::Event if component && binding.modifiers.is_empty() => true,
        _ => return Err(LegacyReason::Component.into()),
    };
    if !handler && !component_prop(binding.name)
        || !fresh && !matches!(binding.name, "class" | "style")
    {
        return Err(LegacyReason::Component.into());
    }
    props.push(Prop {
        key: binding.name,
        value: Some(binding.value),
        dynamic: true,
        handler,
        position,
    });
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
