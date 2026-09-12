//! DOM patch-flag facts for VNode emission.
//!
//! P3-7 starts moving patch flags out of ad-hoc call construction and into a
//! named fact surface. The current facts are still DOM-realization facts: they
//! consume S2 binding ops, static-value classification, handler cache facts,
//! and the emitter's position inputs, then the VNode writer only applies the
//! few position-local masks (`v-for`, `v-once`, `v-memo`).

use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, OnOp};

use super::props::{
    BindName, StaticBindKeyCasing, bind_name, bind_value_is_static_patchless,
    bind_value_uses_legacy_patchless_runtime_expr, has_prop_modifier, is_dynamic_bind_name,
    is_emitted_key_bind, static_bind_key,
};

pub(super) struct PatchFacts {
    pub flag: i32,
    pub dynamic_props: StdVec<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn binding_patch_facts<'a>(
    bindings: &[BindingOp<'a>],
    is_component: bool,
    if_key: Option<&str>,
    for_item: bool,
    is_ts: bool,
    constant_handler: &dyn Fn(&str) -> bool,
    handler_is_cached: &dyn Fn(&OnOp<'a>) -> bool,
    caches_handlers: bool,
) -> PatchFacts {
    if super::merge::has_object_spread(bindings) {
        return super::merge::object_patch(
            bindings,
            is_component,
            if_key,
            for_item,
            is_ts,
            constant_handler,
            handler_is_cached,
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
                            || (!for_item
                                && bind_value_uses_legacy_patchless_runtime_expr(bind)) => {}
                    "class" if !is_component => flag |= 2,
                    "style"
                        if bind_value_is_static_patchless(bind, is_ts)
                            || (!for_item
                                && bind_value_uses_legacy_patchless_runtime_expr(bind)) => {}
                    "style" if !is_component => flag |= 4,
                    "key" => {}
                    key if key.ends_with("Modifiers")
                        || bind_value_is_static_patchless(bind, is_ts) => {}
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
                let Ok(key) = super::on::event_key_for(on, !is_component) else {
                    continue;
                };
                if !super::props::handler_is_constant(on, constant_handler)
                    && !handler_is_cached(on)
                {
                    flag |= 8;
                    if !dynamic_props.contains(&key) {
                        dynamic_props.push(key.clone());
                    }
                }
                if !is_component && super::on::needs_hydration(key.as_str(), on) {
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
    PatchFacts {
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

pub(super) fn apply_static_ref_patch(attributes: &[Attribute<'_>], flag: &mut i32) {
    let has_static_ref = attributes.iter().any(|attr| attr.name == "ref");
    // The shipped gate is the *normal prop* flags alone: NEED_PATCH still
    // combines with TEXT and NEED_HYDRATION, because neither of those updates
    // a ref on its own.
    if has_static_ref && *flag & (2 | 4 | 8 | 16) == 0 {
        *flag |= 512;
    }
}
