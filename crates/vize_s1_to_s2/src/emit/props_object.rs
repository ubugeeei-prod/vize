mod pieces;

use alloc::vec::Vec as StdVec;
use vize_s2::expr::{ExprRef, OpaqueReason};
use vize_s2::op::{Attribute, BindOp, DynamicName};

use super::js::{escape_js_string, push_ident_key};
use super::model_key::{ModelModifiersKey, ModelName, ModelUpdateKey};
use super::props_bind::{self, StaticBindKeyCasing};
use super::{EmitCx, EmitError};
use super::{on, props_value, style};
pub(super) use pieces::{Piece, pieces};

#[derive(Clone, Copy, Default)]
pub(super) struct PropsObjectOptions<'a> {
    pub if_key: Option<&'a str>,
    pub skip_normalize: bool,
    pub empty_key_multiline: bool,
    pub is_plain_element: bool,
    pub for_item: bool,
    pub suppress_once_cache_dynamic: bool,
    pub force_multiline: bool,
}

pub(super) fn emit_props_object(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
    options: PropsObjectOptions<'_>,
) -> Result<(), EmitError> {
    let PropsObjectOptions {
        if_key,
        skip_normalize,
        empty_key_multiline,
        is_plain_element,
        for_item,
        suppress_once_cache_dynamic,
        force_multiline,
    } = options;
    let split_once_static_class = cx.once_depth > 0
        && pieces_have_static_attr(pieces, "class")
        && pieces_have_named(pieces, "class");
    let skip_class = pieces_have_named(pieces, "class") && !split_once_static_class;
    let skip_style = pieces_have_named(pieces, "style");
    let skip_key = if_key.is_some() || cx.suppress_template_for_child_key;
    let keep_template_if_static_key = if_key.is_some() && cx.template_if_branch_root;
    if suppress_once_cache_dynamic {
        reserve_skipped_once_helpers(cx, pieces)?;
    }
    let visible: StdVec<&Piece<'_>> = pieces
        .iter()
        .filter(|piece| {
            !skip_emitted_key(
                piece,
                skip_class,
                skip_style,
                skip_key,
                keep_template_if_static_key,
            ) && !(suppress_once_cache_dynamic && skip_once_cache_piece(piece))
        })
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
    let v_for_merge_arg_multiline = skip_normalize && for_item && visible.len() + extra > 1;
    let multiline = !compact_multiline
        && (force_multiline
            || (if_key.is_some() && !visible.is_empty())
            || v_for_merge_arg_multiline
            || (!for_item && pieces_have_inline_on(pieces))
            || (!for_item
                && (visible.len() + extra > 1
                    || pieces_have_named(pieces, "class")
                    || pieces_have_named(pieces, "style")
                    || pieces_have_vue_text(pieces))));
    let event_key_plain_element = is_plain_element && if_key.is_none();
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
        if let Some(key) = super::props_object_merge::event_key(piece, event_key_plain_element)
            && super::props_object_merge::count(&visible, key.as_str(), event_key_plain_element) > 1
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
            super::props_object_merge::emit_handlers(
                cx,
                &visible,
                key.as_str(),
                event_key_plain_element,
            )?;
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
            Piece::Bind(bind) => {
                emit_bind_pair(cx, pieces, bind, skip_normalize, is_plain_element)?
            }
            Piece::On(event) => on::emit_on_pair(cx, event, event_key_plain_element)?,
            Piece::VueHtml(html) => super::html::emit_pair(cx, html)?,
            Piece::VueText(text) => super::vtext::emit_pair(cx, text)?,
            Piece::ModelValue { name, model, .. } => {
                let source = super::model::js_source(model)?;
                super::model_key::emit_value(cx, *name, source.as_str())
            }
            Piece::ModelUpdate { key, model, .. } => {
                let source = super::model::js_source(model)?;
                super::model_key::emit_update(cx, key, model, source.as_str())
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

fn skip_emitted_key(
    piece: &Piece<'_>,
    skip_class: bool,
    skip_style: bool,
    skip_key: bool,
    keep_template_if_static_key: bool,
) -> bool {
    match piece {
        Piece::Attr(attr) => {
            (skip_class && attr.name == "class")
                || (skip_style && attr.name == "style" && attr.value.is_some())
                || (skip_key && attr.name == "key" && !keep_template_if_static_key)
        }
        Piece::Bind(bind) if skip_key && super::props_bind::is_key_bind_name(bind) => true,
        _ => false,
    }
}

fn skip_once_cache_piece(piece: &Piece<'_>) -> bool {
    matches!(piece, Piece::On(_) | Piece::VueHtml(_))
}

fn reserve_skipped_once_helpers(
    cx: &mut EmitCx<'_>,
    pieces: &[Piece<'_>],
) -> Result<(), EmitError> {
    for piece in pieces {
        if let Piece::On(on) = piece {
            on::reserve_skipped_once_helpers(cx, on)?;
        }
    }
    Ok(())
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

fn pieces_have_static_attr(pieces: &[Piece<'_>], name: &str) -> bool {
    pieces
        .iter()
        .any(|piece| matches!(piece, Piece::Attr(attr) if attr.name == name))
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
    is_plain_element: bool,
) -> Result<(), EmitError> {
    if props_bind::emit_dynamic_bind_pair(cx, bind)? {
        return Ok(());
    }
    let raw_name = props_bind::static_bind_name(bind)?;
    let key = props_bind::static_bind_key(bind, StaticBindKeyCasing::Preserve)?;
    let value = props_value::bind_value(bind)?;
    let static_style = static_style_piece(pieces);
    let skip_normalize = skip_normalize
        || style::bind_skips_normalize(raw_name, is_plain_element, static_style.is_some(), &value);
    emit_ref_for(cx, key.as_str());
    push_ident_key(cx, key.as_str());
    cx.buf.push(": ");
    match raw_name {
        "class" => match value.js() {
            Some(_) if skip_normalize && !pieces_have_static_attr(pieces, "class") => {
                value.emit_authored(cx, bind);
            }
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
            Some(_) if skip_normalize && static_style.is_none() => {
                value.emit_authored(cx, bind);
            }
            Some(js) => style::emit_style_value(cx, static_style, bind, js, skip_normalize),
            None => value.emit(cx),
        },
        _ => value.emit_authored(cx, bind),
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
