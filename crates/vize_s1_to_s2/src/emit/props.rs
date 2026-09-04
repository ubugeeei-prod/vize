//! Static attrs plus static-name `ui.bind` / `ui.on` props / patch flags.

mod constness;
mod static_expr;
mod ts_view;

use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::on::{admit_on, event_key_for, needs_hydration};
pub(in crate::emit) use constness::handler_is_constant;
pub(super) use constness::{bind_value_is_static_patchless, bind_value_text};
use static_expr::{
    bind_value_uses_legacy_patchless_bounded_string_concat,
    bind_value_uses_legacy_patchless_runtime_expr,
};

pub(super) use super::props_bind::{
    BindName, StaticBindKeyCasing, bind_name, emit_dynamic_bind_pair, has_prop_modifier,
    is_dynamic_bind_name, is_emitted_key_bind, js_value, static_bind_key,
};
pub(super) use super::props_object::{Piece, PropsObjectOptions, emit_props_object, pieces};
pub(super) use super::props_value::bind_value;

pub(super) struct Patch {
    pub flag: i32,
    pub dynamic_props: StdVec<String>,
}

#[derive(Clone, Copy, Default)]
pub(super) struct BindPropsOptions<'a> {
    pub if_key: Option<&'a str>,
    pub skip_is: bool,
    pub for_item: bool,
    pub is_plain_element: bool,
    pub once_layout: bool,
    pub once_cache_initializer: bool,
    pub force_multiline: bool,
}

pub(super) fn admit_element_bindings(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<(), EmitError> {
    admit_bindings_inner(attributes, bindings, true)
}

fn admit_bindings_inner(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    allow_once: bool,
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
                bind_value(bind)?;
                if let BindName::Static(name) = bind_name(bind)? {
                    match name {
                        "class" if class => {
                            return Err(EmitError::unsupported_at(
                                Reason::DuplicateClassBinding,
                                bind.span,
                            ));
                        }
                        "class" => class = true,
                        "style" if style => {
                            return Err(EmitError::unsupported_at(
                                Reason::DuplicateStyleBinding,
                                bind.span,
                            ));
                        }
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
            BindingOp::VueShow(show) => super::directive::admit_show(show)?,
            BindingOp::VueHtml(html) => super::html::admit(html)?,
            BindingOp::VueText(text) => super::vtext::admit(text)?,
            BindingOp::VueCloak(_) => {}
            BindingOp::VueOnce(_) if allow_once => {}
            BindingOp::VueMemo(memo) => super::memo::admit(memo)?,
            _ => {
                return Err(EmitError::unsupported_binding(
                    Reason::UnsupportedBindingKind,
                    binding,
                ));
            }
        }
    }
    if style
        && let Some(attr) = attributes
            .iter()
            .find(|attr| attr.name == "style" && attr.value.is_none())
    {
        return Err(EmitError::unsupported_at(
            Reason::BareStyleAttributeWithDynamicStyle,
            attr.span,
        ));
    }
    Ok(())
}

pub(super) fn bind_patch(
    bindings: &[BindingOp<'_>],
    is_component: bool,
    if_key: Option<&str>,
    for_item: bool,
    is_ts: bool,
    constant_handler: &dyn Fn(&str) -> bool,
    caches_handlers: bool,
) -> Patch {
    if super::merge::has_object_spread(bindings) {
        return super::merge::object_patch(
            bindings,
            is_component,
            if_key,
            for_item,
            is_ts,
            constant_handler,
            caches_handlers,
        );
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
                    "class"
                        if bind_value_is_static_patchless(bind, is_ts)
                            || bind_value_uses_legacy_patchless_runtime_expr(bind) => {}
                    "class" if !is_component => flag |= 2,
                    "style"
                        if bind_value_is_static_patchless(bind, is_ts)
                            || bind_value_uses_legacy_patchless_runtime_expr(bind) => {}
                    "style" if !is_component => flag |= 4,
                    "key" => {}
                    key if key.ends_with("Modifiers")
                        || bind_value_is_static_patchless(bind, is_ts)
                        || bind_value_uses_legacy_patchless_bounded_string_concat(bind) => {}
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
                if super::on_dynamic::is_dynamic_on_name(on) {
                    flag |= 16;
                    continue;
                }
                let Ok(key) = event_key_for(on, !is_component) else {
                    continue;
                };
                // `is_const_handler` / `handler_is_cached`: a handler that
                // is just a constant binding never changes, and a cached
                // one is created once — neither is a patch target. The
                // cache rule reads the option alone, without the
                // const-reference carve-out `needs_von_handler_cache`
                // applies to the emission itself.
                if !handler_is_constant(on, constant_handler)
                    && !(caches_handlers && on.handler.is_some())
                {
                    flag |= 8;
                    if !dynamic_props.contains(&key) {
                        dynamic_props.push(key.clone());
                    }
                }
                if !is_component && needs_hydration(key.as_str(), on) {
                    flag |= 32;
                }
            }
            BindingOp::Model(model) => {
                super::model::patch(
                    model,
                    is_component,
                    &mut flag,
                    &mut dynamic_props,
                    caches_handlers,
                );
            }
            BindingOp::VueHtml(_) => {
                flag |= 8;
                let key = String::from("innerHTML");
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key);
                }
            }
            BindingOp::VueText(_) => {
                flag |= 8;
                let key = String::from("textContent");
                if !dynamic_props.contains(&key) {
                    dynamic_props.push(key);
                }
            }
            _ => {}
        }
    }
    // The shipped `NEED_PATCH` gate names `v-model` beside `v-show`, the
    // custom directives and `ref`. Only a *cached* update handler reaches
    // the difference: without caching the model always sets `PROPS`,
    // which suppresses `NEED_PATCH` on both sides.
    let has_model = bindings
        .iter()
        .any(|binding| matches!(binding, BindingOp::Model(_)));
    if (super::directive::has_runtime(bindings) || has_model) && flag & (2 | 4 | 8 | 16) == 0 {
        flag |= 512;
    }
    if flag & 16 != 0 {
        flag &= !(2 | 4 | 8);
    }
    Patch {
        flag,
        dynamic_props,
    }
}

pub(super) fn prune_legacy_patchless_dynamic_props(
    bindings: &[BindingOp<'_>],
    dynamic_props: &mut StdVec<String>,
) {
    for binding in bindings.iter() {
        let BindingOp::Bind(bind) = binding else {
            continue;
        };
        if has_prop_modifier(bind) || !bind_value_uses_legacy_patchless_runtime_expr(bind) {
            continue;
        }
        let Ok(BindName::Static(_)) = bind_name(bind) else {
            continue;
        };
        let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
            continue;
        };
        dynamic_props.retain(|name| name.as_str() != key.as_str());
    }
}

pub(super) fn emit_bind_props(
    cx: &mut EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    options: BindPropsOptions<'_>,
) -> Result<(), EmitError> {
    let BindPropsOptions {
        if_key,
        skip_is,
        for_item,
        is_plain_element,
        once_layout,
        once_cache_initializer,
        force_multiline,
    } = options;
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
    let normalize = !for_item && has_dynamic_bind_name(bindings, if_key);
    if normalize {
        cx.buf.use_normalize_props();
        cx.buf.push(Buf::normalize_props_alias());
        cx.buf.push("(");
    }
    emit_props_object(
        cx,
        &pieces,
        PropsObjectOptions {
            if_key,
            skip_normalize: false,
            empty_key_multiline: for_item
                && (super::directive::has_runtime(bindings) || once_layout || force_multiline),
            is_plain_element,
            for_item,
            suppress_once_cache_dynamic: once_cache_initializer,
            force_multiline: once_layout || force_multiline,
        },
    )?;
    if normalize {
        cx.buf.push(")");
    }
    Ok(())
}

pub(super) fn apply_static_ref_patch(attributes: &[Attribute<'_>], flag: &mut i32) {
    let has_static_ref = attributes.iter().any(|attr| attr.name == "ref");
    if has_static_ref && *flag & (2 | 4 | 8 | 16 | 32) == 0 {
        *flag |= 512;
    }
}

fn has_dynamic_bind_name(bindings: &[BindingOp<'_>], if_key: Option<&str>) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) => is_dynamic_bind_name(bind) && !is_emitted_key_bind(bind, if_key),
        BindingOp::Model(model) => super::model::has_dynamic_argument(model),
        _ => false,
    })
}
