//! Inline static props for native element calls.

mod legacy_constant;

use legacy_constant::legacy_global_constant_expr;
use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, DynamicName};

use crate::pass::StaticLevel;

use super::EmitCx;
use super::EmitError;
use super::hoist::{push_attr_pair, unique_attrs};
use super::js::{escape_js_string, is_valid_js_identifier, js_expr_source};
use super::props::{Piece, bind_value_is_static_patchless, pieces, static_bind_key};
use super::props_bind::{StaticBindKey, StaticBindKeyCasing};
use super::props_value::bind_value;

#[derive(Clone, Copy)]
pub(super) enum PropHoistPosition {
    Root,
    Nested,
    ForItem,
}

pub(super) fn should_hoist(
    cx: &EmitCx<'_>,
    id: Option<NodeId>,
    position: PropHoistPosition,
) -> bool {
    let Some(fact) = id.and_then(|id| cx.facts.static_facts.get(id)) else {
        return false;
    };
    if !fact.props_hoistable {
        return false;
    }
    match position {
        PropHoistPosition::Root => match fact.level {
            StaticLevel::FullyStatic | StaticLevel::HasDynamicText => true,
            StaticLevel::NotStatic => fact.foreign || fact.nested_static,
        },
        PropHoistPosition::Nested => {
            fact.level == StaticLevel::NotStatic && (fact.foreign || fact.nested_static)
        }
        PropHoistPosition::ForItem => true,
    }
}

pub(super) fn root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<Option<String>, EmitError> {
    if !can_root_hoist_props(attributes, bindings) {
        return Ok(None);
    }

    let mut out = String::from("{ ");
    let mut emitted = 0usize;
    let pieces = pieces(attributes, bindings, false)?;
    for (index, piece) in pieces.iter().enumerate() {
        let mut prop = String::default();
        let Some(key) = static_hoist_prop(&mut prop, piece)? else {
            return Ok(None);
        };
        if has_prior_hoist_key(&pieces[..index], key.as_str())? {
            continue;
        }
        if emitted > 0 {
            out.push_str(", ");
        }
        out.push_str(prop.as_str());
        emitted += 1;
    }
    if emitted == 0 {
        return Ok(None);
    }
    out.push_str(" }");
    Ok(Some(out))
}

pub(super) struct ComponentHoistProps {
    pub(super) source: String,
    pub(super) dynamic_values: bool,
    pub(super) non_key: bool,
    pub(super) valued_prop: bool,
}

pub(super) fn component_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> Result<Option<ComponentHoistProps>, EmitError> {
    let pieces = pieces(attributes, bindings, false)?;
    let mut out = String::from("{ ");
    let mut emitted = 0usize;
    let mut dynamic_values = false;
    let mut non_key = false;
    let mut valued_prop = false;
    for (index, piece) in pieces.iter().enumerate() {
        let mut prop = String::default();
        let Some((key, dynamic_value)) = component_hoist_prop(&mut prop, piece)? else {
            return Ok(None);
        };
        if has_prior_component_hoist_key(&pieces[..index], key.as_str())? {
            continue;
        }
        dynamic_values |= dynamic_value;
        non_key |= key.as_str() != "key";
        valued_prop |= match piece {
            Piece::Attr(attr) => attr.value.is_some(),
            Piece::Bind(_) => true,
            _ => false,
        };
        if emitted > 0 {
            out.push_str(", ");
        }
        out.push_str(prop.as_str());
        emitted += 1;
    }
    if emitted == 0 {
        return Ok(None);
    }
    out.push_str(" }");
    Ok(Some(ComponentHoistProps {
        source: out,
        dynamic_values,
        non_key,
        valued_prop,
    }))
}

pub(super) fn static_vnode_surface_can_hoist(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> bool {
    attributes.iter().all(|attr| attr.name != "ref") && bindings.iter().all(root_binding_can_hoist)
}

fn can_root_hoist_props(attributes: &[Attribute<'_>], bindings: &[BindingOp<'_>]) -> bool {
    if attributes.is_empty() && bindings.is_empty() {
        return false;
    }

    if attributes.iter().any(|attr| attr.name == "ref") {
        return false;
    }

    bindings.iter().all(root_binding_can_hoist)
}

fn root_binding_can_hoist(binding: &BindingOp<'_>) -> bool {
    let BindingOp::Bind(bind) = binding else {
        return false;
    };

    if !matches!(bind.name, Some(DynamicName::Static(_))) {
        return false;
    }

    if !bind_value_is_static_patchless(bind) {
        return false;
    }

    let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
        return false;
    };

    !matches!(key.as_str(), "ref" | "class")
}

fn static_hoist_prop<'a>(
    out: &mut String,
    piece: &Piece<'a>,
) -> Result<Option<HoistKey<'a>>, EmitError> {
    let Some(key) = hoist_key(piece)? else {
        return Ok(None);
    };
    match piece {
        Piece::Attr(attr) => {
            push_attr_pair(out, attr);
        }
        Piece::Bind(bind) => {
            push_key(out, key.as_str());
            out.push_str(": ");
            if let Some(js) = bind_value(bind)?.js() {
                let source = js_expr_source(js);
                out.push_str(source.as_str());
            }
        }
        _ => return Ok(None),
    }
    Ok(Some(key))
}

fn component_hoist_prop<'a>(
    out: &mut String,
    piece: &Piece<'a>,
) -> Result<Option<(HoistKey<'a>, bool)>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => {
            push_attr_pair(out, attr);
            Ok(Some((HoistKey::Borrowed(attr.name), false)))
        }
        Piece::Bind(bind) => {
            let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
                return Ok(None);
            };
            let dynamic_value = !bind_value_is_static_patchless(bind);
            if matches!(key.as_str(), "ref" | "class") {
                return Ok(None);
            }
            let value = bind_value(bind)?;
            let Some(js) = value.js() else {
                return Ok(None);
            };
            if dynamic_value && !legacy_global_constant_expr(js.ast, js.source) {
                return Ok(None);
            }
            push_key(out, key.as_str());
            out.push_str(": ");
            let source = js_expr_source(js);
            out.push_str(source.as_str());
            Ok(Some((HoistKey::StaticBind(key), dynamic_value)))
        }
        _ => Ok(None),
    }
}

fn has_prior_hoist_key(pieces: &[Piece<'_>], key: &str) -> Result<bool, EmitError> {
    for piece in pieces {
        if hoist_key(piece)?.is_some_and(|prior| prior.as_str() == key) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_prior_component_hoist_key(pieces: &[Piece<'_>], key: &str) -> Result<bool, EmitError> {
    for piece in pieces {
        if component_hoist_key(piece)?.is_some_and(|prior| prior.as_str() == key) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn hoist_key<'a>(piece: &Piece<'a>) -> Result<Option<HoistKey<'a>>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => Ok(Some(HoistKey::Borrowed(attr.name))),
        Piece::Bind(bind) if bind_value_is_static_patchless(bind) => {
            let key = static_bind_key(bind, StaticBindKeyCasing::Preserve)?;
            if matches!(key.as_str(), "ref" | "class") {
                return Ok(None);
            }
            Ok(Some(HoistKey::StaticBind(key)))
        }
        _ => Ok(None),
    }
}

fn component_hoist_key<'a>(piece: &Piece<'a>) -> Result<Option<HoistKey<'a>>, EmitError> {
    match piece {
        Piece::Attr(attr) if attr.name != "ref" => Ok(Some(HoistKey::Borrowed(attr.name))),
        Piece::Bind(bind) => {
            let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
                return Ok(None);
            };
            if key.as_str() == "ref"
                || (key.as_str() == "class" && !bind_value_is_static_patchless(bind))
            {
                return Ok(None);
            }
            Ok(Some(HoistKey::StaticBind(key)))
        }
        _ => Ok(None),
    }
}

enum HoistKey<'a> {
    Borrowed(&'a str),
    StaticBind(StaticBindKey<'a>),
}

impl HoistKey<'_> {
    fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(text) => text,
            Self::StaticBind(key) => key.as_str(),
        }
    }
}

fn push_key(out: &mut String, key: &str) {
    if !is_valid_js_identifier(key) {
        out.push('"');
        out.push_str(escape_js_string(key).as_str());
        out.push('"');
        return;
    }
    out.push_str(key);
}

pub(super) fn emit_inline<'a>(
    cx: &mut EmitCx<'_>,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
) {
    let unique = unique_attrs(attributes);
    let multiline = unique.len() > 1;
    if multiline {
        cx.buf.push("{");
        cx.buf.indent();
    } else {
        cx.buf.push("{ ");
    }
    for (i, attr) in unique.iter().enumerate() {
        if i > 0 {
            cx.buf.push(",");
        }
        if multiline {
            cx.buf.newline();
        } else if i > 0 {
            cx.buf.push(" ");
        }
        if cx.in_v_for && attr.name == "ref" {
            cx.buf.push("ref_for: true, ");
        }
        let mut pair = String::default();
        push_attr_pair(&mut pair, attr);
        cx.buf.push(pair.as_str());
    }
    if multiline {
        cx.buf.deindent();
        cx.buf.newline();
        cx.buf.push("}");
    } else {
        cx.buf.push(" }");
    }
}
