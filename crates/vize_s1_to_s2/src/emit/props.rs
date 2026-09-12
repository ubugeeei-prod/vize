//! Static attrs plus static-name `ui.bind` / `ui.on` props / patch flags.

mod constness;
mod static_expr;
mod ts_view;

use vize_s2::op::{Attribute, BindingOp};

use super::EmitCx;
use super::EmitError;
use super::UnsupportedReason as Reason;
use super::buf::Buf;
use super::on::admit_on;
pub(in crate::emit) use constness::handler_is_constant;
pub(super) use constness::{bind_value_is_static_patchless, bind_value_text};
pub(super) use static_expr::bind_value_uses_legacy_patchless_runtime_expr;

pub(super) use super::props_bind::{
    BindName, StaticBindKeyCasing, bind_name, emit_dynamic_bind_pair, has_prop_modifier,
    is_dynamic_bind_name, is_emitted_key_bind, js_value, static_bind_key,
};
pub(super) use super::props_object::{Piece, PropsObjectOptions, emit_props_object, pieces};
pub(super) use super::props_value::bind_value;

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
    _attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<(), EmitError> {
    admit_bindings_inner(bindings, true)
}

fn admit_bindings_inner(bindings: &[BindingOp<'_>], allow_once: bool) -> Result<(), EmitError> {
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
    Ok(())
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

fn has_dynamic_bind_name(bindings: &[BindingOp<'_>], if_key: Option<&str>) -> bool {
    bindings.iter().any(|binding| match binding {
        BindingOp::Bind(bind) => is_dynamic_bind_name(bind) && !is_emitted_key_bind(bind, if_key),
        BindingOp::Model(model) => super::model::has_dynamic_argument(model),
        _ => false,
    })
}
