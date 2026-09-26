//! Expand a checked component model without parsing generated key expressions.

use oxc_allocator::HashSet;
use vize_carton::{Allocator, String, Vec, cstr};

use super::super::{Binding, BindingKind, Expr, Prop};
use super::Result;
use crate::s3::LegacyReason;

/// Expand a checked component model into the same getter, update listener,
/// and optional modifier props as the retained component transform.
pub(super) fn expand<'a>(
    props: &mut Vec<'a, Prop<'a>>,
    binding: &Binding<'a>,
    position: u32,
    names: &mut HashSet<'_, (usize, BindingKind, &'a str, bool)>,
    index: usize,
    alloc: &'a Allocator,
) -> Result<()> {
    let prop = binding.name;
    let computed = binding.dynamic_name.is_some();
    let event = alloc.alloc_str(&cstr!("update:{prop}"));
    let modifiers = if binding.modifiers.is_empty() {
        None
    } else if !computed && matches!(prop, "modelValue" | "model-value") {
        Some("modelModifiers")
    } else {
        Some(alloc.alloc_str(&cstr!("{prop}Modifiers")))
    };
    if !names.insert((index, BindingKind::Prop, prop, computed))
        || !names.insert((index, BindingKind::Event, event, computed))
        || modifiers.is_some_and(|name| !names.insert((index, BindingKind::Prop, name, computed)))
    {
        return Err(LegacyReason::Component.into());
    }
    props.push(Prop {
        key: prop,
        dynamic_name: binding.dynamic_name,
        value: Some(binding.value),
        value_kind: crate::ir::PropValueKind::Expression,
        dynamic: true,
        handler: false,
        position,
    });
    props.push(Prop {
        key: event,
        dynamic_name: binding.dynamic_name,
        value: Some(binding.value),
        value_kind: crate::ir::PropValueKind::ModelUpdate,
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
            dynamic_name: binding.dynamic_name,
            value: Some(Expr::plain(alloc.alloc_str(&object))),
            value_kind: crate::ir::PropValueKind::ModelModifiers,
            dynamic: true,
            handler: false,
            position,
        });
    }
    Ok(())
}
