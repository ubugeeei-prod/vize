//! Static attrs plus static-name `ui.bind` / `ui.on` props / patch flags.

use alloc::vec::Vec as StdVec;

use vize_carton::String;
use vize_disegno::op::{Attribute, BindingOp};

use super::EmitCx;
use super::EmitError;
use super::buf::Buf;
use super::on::{admit_on, event_key_for, needs_hydration};

pub(super) use super::props_bind::{
    BindName, StaticBindKeyCasing, bind_name, emit_dynamic_bind_pair, has_prop_modifier,
    is_dynamic_bind_name, is_emitted_key_bind, js_value, static_bind_key,
};
pub(super) use super::props_object::{Piece, emit_props_object, pieces};

pub(super) struct Patch {
    pub flag: i32,
    pub dynamic_props: StdVec<String>,
}

pub(super) fn admit_bindings(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<(), EmitError> {
    let mut class = false;
    let mut style = false;
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) if bind.name.is_none() => {
                super::merge::admit_object(bind)?;
            }
            BindingOp::On(on) if on.name.is_none() => {
                super::merge::admit_object_on(on)?;
            }
            BindingOp::Bind(bind) => {
                js_value(bind)?;
                if let BindName::Static(name) = bind_name(bind)? {
                    match name {
                        "class" if class => return Err(EmitError::Unsupported),
                        "class" => class = true,
                        "style" if style => return Err(EmitError::Unsupported),
                        "style" => style = true,
                        _ => {}
                    }
                }
            }
            BindingOp::On(on) => admit_on(on)?,
            BindingOp::Model(model) => super::model::admit(model)?,
            BindingOp::SlotContent(_) => {}
            BindingOp::VueDirective(_) if super::slots::is_slots_spread(binding) => {}
            BindingOp::VueDirective(directive) => super::directive::admit(directive)?,
            _ => return Err(EmitError::Unsupported),
        }
    }
    if style && has_attr(attributes, "style") && static_attr_value(attributes, "style").is_none() {
        return Err(EmitError::Unsupported);
    }
    Ok(())
}

pub(super) fn bind_patch(
    bindings: &[BindingOp<'_>],
    is_component: bool,
    if_key: Option<&str>,
    for_item: bool,
) -> Patch {
    if super::merge::has_object_spread(bindings) {
        return super::merge::object_patch(bindings, is_component, if_key, for_item);
    }
    let mut flag = 0i32;
    let mut dynamic_props = StdVec::new();
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind) => match bind_name(bind) {
                _ if is_emitted_key_bind(bind, if_key) => {
                    if for_item && is_dynamic_bind_name(bind) {
                        flag |= 16;
                    }
                }
                Ok(BindName::Static(raw_name)) => match raw_name {
                    "ref" => flag |= 512,
                    "class" if !is_component => flag |= 2,
                    "style" if !is_component => flag |= 4,
                    "key" => {}
                    _ => {
                        flag |= 8;
                        let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
                            continue;
                        };
                        let owned = String::from(key.as_str());
                        if !dynamic_props.contains(&owned) {
                            dynamic_props.push(owned);
                        }
                        if has_prop_modifier(bind) {
                            flag |= 32;
                        }
                    }
                },
                Ok(BindName::Dynamic(_)) => {
                    flag |= 16;
                    if has_prop_modifier(bind) {
                        flag |= 32;
                    }
                }
                Ok(BindName::Spread) | Err(_) => {}
            },
            BindingOp::On(on) => {
                let Ok(key) = event_key_for(on, !is_component) else {
                    continue;
                };
                flag |= 8;
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key.clone());
                }
                if !is_component && needs_hydration(key.as_str(), on) {
                    flag |= 32;
                }
            }
            BindingOp::Model(model) => {
                super::model::patch(model, is_component, &mut flag, &mut dynamic_props);
            }
            _ => {}
        }
    }
    if super::directive::has_custom(bindings) && flag & (2 | 4 | 8 | 16) == 0 {
        flag |= 512;
    }
    Patch {
        flag,
        dynamic_props,
    }
}

pub(super) fn emit_bind_props(
    cx: &mut EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    if_key: Option<&str>,
    skip_is: bool,
    for_item: bool,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    if super::merge::has_object_spread(bindings) {
        return super::merge::emit_spread_props(
            cx,
            attributes,
            bindings,
            if_key,
            skip_is,
            for_item,
            is_plain_element,
        );
    }
    let pieces = pieces(attributes, bindings, skip_is)?;
    let normalize = has_dynamic_bind_name(bindings, if_key);
    if normalize {
        cx.buf.use_normalize_props();
        cx.buf.push(Buf::normalize_props_alias());
        cx.buf.push("(");
    }
    emit_props_object(
        cx,
        &pieces,
        if_key,
        false,
        for_item && super::directive::has_custom(bindings),
        is_plain_element,
    )?;
    if normalize {
        cx.buf.push(")");
    }
    Ok(())
}

pub(super) fn apply_static_ref_patch(attributes: &[Attribute<'_>], flag: &mut i32) {
    let has_static_ref = attributes.iter().any(|attr| attr.name == "ref");
    if has_static_ref && *flag & (2 | 4 | 8 | 16 | 32 | 1024) == 0 {
        *flag |= 512;
    }
}

fn has_attr(attributes: &[Attribute<'_>], name: &str) -> bool {
    attributes.iter().any(|attr| attr.name == name)
}

fn static_attr_value<'a>(attributes: &'a [Attribute<'a>], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attr| attr.name == name)
        .and_then(|attr| attr.value)
}

fn has_dynamic_bind_name(bindings: &[BindingOp<'_>], if_key: Option<&str>) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) => is_dynamic_bind_name(bind) && !is_emitted_key_bind(bind, if_key),
        _ => false,
    })
}
