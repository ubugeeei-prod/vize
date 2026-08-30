//! Object-literal emission for static attrs plus bind / on / model pieces.

use alloc::vec::Vec as StdVec;
use vize_s0::Span;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{Attribute, BindOp, BindingOp, DynamicName, OnOp, VueHtmlOp, VueTextOp};

use super::EmitCx;
use super::EmitError;
use super::error::UnsupportedReason as Reason;
use super::js::{escape_js_string, push_ident_key};
use super::model_key::{ModelModifiersKey, ModelName, ModelUpdateKey};
use super::props_bind::{self, StaticBindKeyCasing};
use super::props_value;
use super::{on, style};

pub(super) fn emit_props_object(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    if_key: Option<&str>,
    skip_normalize: bool,
    empty_key_multiline: bool,
    is_plain_element: bool,
    for_item: bool,
) -> Result<(), EmitError> {
    let skip_class = pieces_have_named(pieces, "class");
    let skip_style = pieces_have_named(pieces, "style");
    let skip_key = if_key.is_some();
    let visible: StdVec<&Piece<'_>> = pieces
        .iter()
        .filter(|piece| !skip_emitted_key(piece, if_key, skip_class, skip_style, skip_key))
        .collect();
    if let Some(key) = if_key
        && visible.is_empty()
    {
        if empty_key_multiline {
            cx.buf.push("{");
            cx.buf.indent();
            cx.buf.newline();
            cx.buf.push("key: ");
            cx.buf.push(key);
            cx.buf.deindent();
            cx.buf.newline();
            cx.buf.push("}");
        } else {
            cx.buf.push("{ key: ");
            cx.buf.push(key);
            cx.buf.push(" }");
        }
        return Ok(());
    }
    let extra = usize::from(if_key.is_some());
    let compact_multiline = if_key.is_none() && pieces_are_dynamic_model_products(&visible);
    let multiline = !compact_multiline
        && ((if_key.is_some() && !visible.is_empty())
            || pieces_have_inline_on(pieces)
            || (!for_item
                && (visible.len() + extra > 1
                    || pieces_have_named(pieces, "class")
                    || pieces_have_named(pieces, "style")
                    || pieces_have_vue_text(pieces))));
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    let mut i = 0;
    if let Some(key) = if_key {
        if multiline {
            cx.buf.newline();
        }
        cx.buf.push("key: ");
        cx.buf.push(key);
        i = 1;
    }
    let mut emitted_merged = StdVec::new();
    for piece in visible.iter() {
        if let Some(key) = super::props_object_merge::event_key(piece, is_plain_element)
            && super::props_object_merge::count(&visible, key.as_str(), is_plain_element) > 1
        {
            if emitted_merged.contains(&key) {
                continue;
            }
            if i > 0 {
                cx.buf.push(",");
            }
            if multiline {
                cx.buf.newline();
            } else if i > 0 {
                cx.buf.push(" ");
            }
            super::props_object_merge::emit_handlers(cx, &visible, key.as_str(), is_plain_element)?;
            emitted_merged.push(key);
            i += 1;
            continue;
        }
        if i > 0 {
            cx.buf.push(",");
        }
        if multiline || (compact_multiline && i > 0) {
            cx.buf.newline();
        } else if i > 0 {
            cx.buf.push(" ");
        }
        match piece {
            Piece::Attr(attr) => emit_static_pair(cx, attr),
            Piece::Bind(bind) => emit_bind_pair(cx, pieces, bind, skip_normalize)?,
            Piece::On(event) => on::emit_on_pair(cx, event, is_plain_element)?,
            Piece::VueHtml(html) => super::html::emit_pair(cx, html)?,
            Piece::VueText(text) => super::vtext::emit_pair(cx, text)?,
            Piece::ModelValue { name, source, .. } => {
                super::model_key::emit_value(cx, *name, source)
            }
            Piece::ModelUpdate { key, source, .. } => {
                super::model_key::emit_update(cx, key, source)
            }
            Piece::ModelModifiers {
                name, modifiers, ..
            } => super::model_key::emit_modifiers(cx, name, modifiers),
        }
        i += 1;
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}");
    } else {
        cx.buf.push(" }");
    }
    Ok(())
}

pub(super) enum Piece<'a> {
    Attr(&'a Attribute<'a>),
    Bind(&'a BindOp<'a>),
    On(&'a OnOp<'a>),
    VueHtml(&'a VueHtmlOp<'a>),
    VueText(&'a VueTextOp<'a>),
    ModelValue {
        name: ModelName<'a>,
        source: &'a str,
        span: Span,
    },
    ModelUpdate {
        key: ModelUpdateKey<'a>,
        source: &'a str,
        span: Span,
    },
    ModelModifiers {
        name: ModelModifiersKey<'a>,
        modifiers: StdVec<&'a str>,
        span: Span,
    },
}

pub(super) fn pieces<'a>(
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
    skip_is: bool,
) -> Result<StdVec<Piece<'a>>, EmitError> {
    let mut out = StdVec::new();
    for attr in attributes.iter() {
        if skip_is && attr.name == "is" {
            continue;
        }
        out.push(Piece::Attr(attr));
    }
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind)
                if skip_is && matches!(bind.name, Some(DynamicName::Static("is"))) => {}
            BindingOp::Bind(bind) => out.push(Piece::Bind(bind)),
            BindingOp::On(on) => out.push(Piece::On(on)),
            BindingOp::Model(model) => super::model::expand(model, &mut out)?,
            BindingOp::VueHtml(html) => out.push(Piece::VueHtml(html)),
            BindingOp::VueText(text) => out.push(Piece::VueText(text)),
            BindingOp::SlotContent(_) => {}
            BindingOp::VueDirective(_) => {}
            BindingOp::VueOnce(_) => {}
            BindingOp::VueMemo(_) => {}
            BindingOp::VueShow(_) => {}
            BindingOp::VueCloak(_) => {}
            _ => {
                return Err(EmitError::unsupported_binding(
                    Reason::UnsupportedBindingKind,
                    binding,
                ));
            }
        }
    }
    out.sort_by_key(|piece| piece.span().start);
    Ok(out)
}

impl Piece<'_> {
    pub(super) fn span(&self) -> Span {
        match self {
            Self::Attr(attr) => attr.span,
            Self::Bind(bind) => bind.span,
            Self::On(on) => on.span,
            Self::VueHtml(html) => html.span,
            Self::VueText(text) => text.span,
            Self::ModelValue { span, .. }
            | Self::ModelUpdate { span, .. }
            | Self::ModelModifiers { span, .. } => *span,
        }
    }
}

fn skip_emitted_key(
    piece: &Piece<'_>,
    if_key: Option<&str>,
    skip_class: bool,
    skip_style: bool,
    skip_key: bool,
) -> bool {
    match piece {
        Piece::Attr(attr) => {
            (skip_class && attr.name == "class")
                || (skip_style && attr.name == "style" && attr.value.is_some())
                || (skip_key && attr.name == "key")
        }
        Piece::Bind(bind) if skip_key && super::props_bind::is_emitted_key_bind(bind, if_key) => {
            true
        }
        _ => false,
    }
}

fn pieces_have_named(pieces: &[Piece<'_>], name: &str) -> bool {
    pieces.iter().any(|piece| match piece {
        Piece::Bind(bind) => matches!(bind.name, Some(DynamicName::Static(n)) if n == name),
        Piece::ModelValue {
            name: ModelName::Static(prop),
            ..
        } => *prop == name,
        Piece::ModelModifiers {
            name: ModelModifiersKey::Static(prop),
            ..
        } => prop.as_str() == name,
        Piece::VueHtml(_) => name == "innerHTML",
        Piece::VueText(_) => name == "textContent",
        _ => false,
    })
}

fn pieces_have_inline_on(pieces: &[Piece<'_>]) -> bool {
    pieces.iter().any(|piece| match piece {
        Piece::On(event) => on::forces_inline_on(event)
            || matches!(event.handler, Some(ExprRef::Js(js)) if on::is_inline_handler_source(js.source))
            || matches!(event.handler, Some(ExprRef::Opaque(opaque)) if opaque.reason == OpaqueReason::MultiStatement),
        Piece::ModelUpdate { .. } => true,
        _ => false,
    })
}

fn pieces_have_vue_text(pieces: &[Piece<'_>]) -> bool {
    pieces
        .iter()
        .any(|piece| matches!(piece, Piece::VueText(_)))
}

fn pieces_are_dynamic_model_products(pieces: &[&Piece<'_>]) -> bool {
    !pieces.is_empty()
        && pieces.iter().all(|piece| {
            matches!(
                piece,
                Piece::ModelValue {
                    name: ModelName::Dynamic(_),
                    ..
                } | Piece::ModelUpdate {
                    key: ModelUpdateKey::Dynamic(_),
                    ..
                } | Piece::ModelModifiers {
                    name: ModelModifiersKey::Dynamic(_),
                    ..
                }
            )
        })
}

fn emit_static_pair(cx: &mut EmitCx<'_>, attr: &Attribute<'_>) {
    emit_ref_for(cx, attr.name);
    push_ident_key(cx, attr.name);
    cx.buf.push(": \"");
    if let Some(value) = attr.value {
        cx.buf.push(escape_js_string(value).as_str());
    }
    cx.buf.push("\"");
}

fn emit_bind_pair(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    bind: &BindOp<'_>,
    skip_normalize: bool,
) -> Result<(), EmitError> {
    if props_bind::emit_dynamic_bind_pair(cx, bind)? {
        return Ok(());
    }
    let raw_name = props_bind::static_bind_name(bind)?;
    let key = props_bind::static_bind_key(bind, StaticBindKeyCasing::Preserve)?;
    let value = props_value::bind_value(bind)?;
    emit_ref_for(cx, key.as_str());
    push_ident_key(cx, key.as_str());
    cx.buf.push(": ");
    match raw_name {
        "class" => match value.js() {
            Some(js) => super::props_class::emit_class_value(cx, pieces, bind, js, skip_normalize),
            None => {
                if !skip_normalize {
                    cx.buf.use_normalize_class();
                    cx.buf.push(super::buf::Buf::normalize_class_alias());
                    cx.buf.push("(");
                }
                value.emit(cx);
                if !skip_normalize {
                    cx.buf.push(")");
                }
            }
        },
        "style" => match value.js() {
            Some(js) => {
                style::emit_style_value(cx, static_style_piece(pieces), bind, js, skip_normalize)
            }
            None => value.emit(cx),
        },
        _ => value.emit(cx),
    }
    Ok(())
}

fn emit_ref_for(cx: &mut EmitCx<'_>, name: &str) {
    if cx.in_v_for && name == "ref" {
        cx.buf.push("ref_for: true, ");
    }
}

fn static_style_piece<'a>(pieces: &'a [Piece<'a>]) -> Option<(&'a Attribute<'a>, &'a str)> {
    pieces.iter().find_map(|piece| match piece {
        Piece::Attr(attr) if attr.name == "style" => attr.value.map(|value| (*attr, value)),
        _ => None,
    })
}
