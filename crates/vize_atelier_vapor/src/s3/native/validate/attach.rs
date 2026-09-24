//! Bindings attach to their native owner once the tree is known: each
//! name/family binds at most once, a loop body's `:key` moves to its loop, a
//! static `class`/`style` merge into bound values,
//! component/outlet bindings become props in authored order, and a `v-bind`
//! or `v-on` object on a component becomes a `$` source in that order.

use oxc_allocator::HashSet;
use vize_carton::{Allocator, String, Vec, cstr};
use vize_s3::op::{OpId, RegionId};

use super::super::{Binding, BindingKind, Content, Expr, Node, Prop};
use super::{Result, Slots, at, at_mut, component::component_prop};
use crate::s3::{AdmissionFailure, LegacyReason};

type Pending<'a> = (OpId, RegionId, u32, Binding<'a>);

pub(super) fn bindings<'a>(
    nodes: &mut [Node<'a>],
    slots: &Slots,
    parents: &[Option<usize>],
    bindings: Vec<'a, Pending<'a>>,
    alloc: &'a Allocator,
) -> Result<()> {
    // Names already bound per native node: static attributes and props first.
    let mut names = HashSet::with_capacity_in(bindings.len() + nodes.len(), alloc.as_oxc());
    for (index, node) in nodes.iter().enumerate() {
        match &node.content {
            Content::Element { attributes, .. } => {
                names.extend(
                    (attributes.iter()).map(|(name, ..)| (index, BindingKind::Prop, *name)),
                );
            }
            Content::Component { props, .. } | Content::Outlet { props, .. } => {
                names.extend((props.iter()).map(|prop| (index, BindingKind::Prop, prop.key)));
            }
            _ => {}
        }
    }
    // Elements whose props merge through a `v-bind` object keep every static
    // attribute and `:class`/`:style` as its own ordered source.
    let mut spreads = super::filled(alloc, false, nodes.len());
    for (target, .., binding) in bindings.iter() {
        if binding.kind == BindingKind::Spread
            && let Some(&Some((index, _))) = slots.get(target.index() as usize)
        {
            *at_mut(&mut spreads, index)? = true;
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
        // Slot content binds to its `<template>` or to its component (only
        // the default slot there); the structure is checked once attached.
        if binding.kind == BindingKind::Slot {
            let node = at_mut(nodes, index)?;
            let admitted = match node.content {
                Content::Element {
                    tag: "template", ..
                } => true,
                Content::Component { .. } => binding.name == "default",
                _ => false,
            };
            if !admitted || node.bindings.iter().any(|b| b.kind == BindingKind::Slot) {
                return Err(LegacyReason::Component.into());
            }
            node.bindings.push(binding);
            continue;
        }
        match &mut at_mut(nodes, index)?.content {
            Content::Element { .. } => {}
            // A `<component>` takes its `:is` once; everything else is a prop.
            Content::Component {
                tag: "component",
                is,
                ..
            } if binding.kind == BindingKind::Prop && binding.name == "is" => {
                if is.replace(binding.value).is_some() {
                    return Err(LegacyReason::Component.into());
                }
                continue;
            }
            Content::Component { props, .. }
                if matches!(binding.kind, BindingKind::Spread | BindingKind::Handlers) =>
            {
                if !fresh {
                    return Err(LegacyReason::Component.into());
                }
                props.push(Prop {
                    key: "$",
                    value: Some(binding.value),
                    dynamic: true,
                    handler: binding.kind == BindingKind::Handlers,
                    position,
                });
                continue;
            }
            Content::Component { tag, props, .. } if binding.kind == BindingKind::Model => {
                // Built-ins and dynamic components retain their own model
                // contracts. One ordinary component model contributes three
                // adjacent props in the directive's authored position.
                if binding.model_element != Some("component") {
                    return Err(AdmissionFailure::Invalid(
                        "model element kind disagrees with target",
                    ));
                }
                if *tag == "component" || !fresh {
                    return Err(LegacyReason::Component.into());
                }
                component_model(props, &binding, position, &mut names, index, alloc)?;
                continue;
            }
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
        if !fresh && *at(&spreads, index)? {
            // Only a `:class`/`:style` beside its static attribute repeats.
            if binding.kind != BindingKind::Prop
                || !matches!(binding.name, "class" | "style")
                || (at(nodes, index)?.bindings.iter())
                    .any(|b| b.kind == binding.kind && b.name == binding.name)
            {
                return Err(LegacyReason::Binding.into());
            }
        } else if !fresh {
            binding.merge = static_class_or_style(at_mut(nodes, index)?, &binding);
            if binding.merge.is_none() {
                return Err(LegacyReason::Binding.into());
            }
        }
        if binding.kind == BindingKind::Prop && binding.name == "is" {
            return Err(LegacyReason::Binding.into());
        }
        if binding.kind == BindingKind::Prop && binding.name == "key" {
            // Only the body element of an element-carried loop owns a key.
            let owner = match *at(parents, index)? {
                Some(parent) => Some(&mut at_mut(nodes, parent)?.content),
                None => None,
            };
            // A `<template v-for>` keys the loop on its wrapper.
            let Some(Content::For(owner)) =
                owner.filter(|owner| !matches!(owner, Content::For(looped) if looped.template))
            else {
                return Err(LegacyReason::Binding.into());
            };
            owner.key_prop = Some(binding.value);
            let [_, value_span] = binding.spans;
            owner.spans.key_prop = Some(value_span);
            continue;
        }
        // Content directives replace the element's children at runtime.
        let node = at_mut(nodes, index)?;
        if matches!(binding.kind, BindingKind::Html | BindingKind::Text)
            && !node.children.is_empty()
        {
            return Err(LegacyReason::Structure.into());
        }
        node.bindings.push(binding);
    }
    for node in nodes.iter_mut() {
        if let Content::Component {
            tag: "component",
            is: None,
            ..
        } = node.content
        {
            return Err(LegacyReason::Component.into());
        }
        if let Content::Component { props, .. } | Content::Outlet { props, .. } = &mut node.content
        {
            props.sort_by_key(|prop| prop.position);
        }
    }
    Ok(())
}

/// Expand a checked component model into the same getter, update listener,
/// and optional modifier props as the retained component transform.
fn component_model<'a>(
    props: &mut Vec<'a, Prop<'a>>,
    binding: &Binding<'a>,
    position: u32,
    names: &mut HashSet<'_, (usize, BindingKind, &'a str)>,
    index: usize,
    alloc: &'a Allocator,
) -> Result<()> {
    let prop = binding.name;
    let event = alloc.alloc_str(&cstr!("update:{prop}"));
    let modifiers = if binding.modifiers.is_empty() {
        None
    } else if matches!(prop, "modelValue" | "model-value") {
        Some("modelModifiers")
    } else {
        Some(alloc.alloc_str(&cstr!("{prop}Modifiers")))
    };
    if !names.insert((index, BindingKind::Prop, prop))
        || !names.insert((index, BindingKind::Event, event))
        || modifiers.is_some_and(|name| !names.insert((index, BindingKind::Prop, name)))
    {
        return Err(LegacyReason::Component.into());
    }
    props.push(Prop {
        key: prop,
        value: Some(binding.value),
        dynamic: true,
        handler: false,
        position,
    });
    let handler = alloc.alloc_str(&cstr!("$event => (({}) = $event)", binding.value.text));
    props.push(Prop {
        key: event,
        value: Some(Expr::plain(handler)),
        dynamic: true,
        handler: true,
        position,
    });
    if let Some(name) = modifiers {
        let mut object = String::from("{ ");
        for (i, modifier) in binding.modifiers.iter().enumerate() {
            if i != 0 {
                object.push_str(", ");
            }
            object.push('"');
            object.push_str(&crate::generate::escape_js_string_literal(modifier));
            object.push_str("\": true");
        }
        object.push_str(" }");
        props.push(Prop {
            key: name,
            value: Some(Expr::plain(alloc.alloc_str(&object))),
            dynamic: true,
            handler: false,
            position,
        });
    }
    Ok(())
}

/// A component `:prop` or `@event`, or an outlet `:prop`. Repeated names merge
/// only for `class`/`style`, which the shared generator normalizes together.
fn prop<'a>(
    props: &mut Vec<'a, Prop<'a>>,
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

/// Remove a valued static class/style beside its bound value. Style follows
/// authored order, as official Vapor does; class keeps the retained lane's
/// static-first lowering until that separate compatibility gap is resolved.
fn static_class_or_style<'a>(
    node: &mut Node<'a>,
    binding: &Binding<'a>,
) -> Option<(&'a str, bool)> {
    if binding.kind != BindingKind::Prop || !matches!(binding.name, "class" | "style") {
        return None;
    }
    let Content::Element { attributes, .. } = &mut node.content else {
        return None;
    };
    let position = attributes
        .iter()
        .position(|(name, value, _)| *name == binding.name && value.is_some())?;
    let (_, _, static_span) = *attributes.get(position)?;
    let [name_span, _] = binding.spans;
    let after = binding.name == "style" && static_span.0 > name_span.0;
    attributes.remove(position).1.map(|value| (value, after))
}
