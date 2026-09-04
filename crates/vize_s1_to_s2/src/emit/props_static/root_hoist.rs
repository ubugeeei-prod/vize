use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, DynamicName};

use super::super::EmitError;
use super::super::hoist::push_empty_attr_pair;
use super::super::props::{Piece, pieces, static_bind_key};
use super::super::props_bind::StaticBindKeyCasing;
use super::hoist_pair::{
    bind_value_is_legacy_static_prop, has_prior_hoist_key, multiline_props_object,
    static_hoist_prop,
};

pub(in crate::emit) fn root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    is_ts: bool,
    scope_id: Option<&str>,
) -> Result<Option<String>, EmitError> {
    root_hoist_props_with_layout(attributes, bindings, None, is_ts, scope_id)
}

pub(in crate::emit) fn cached_root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    line_indent: usize,
    is_ts: bool,
    scope_id: Option<&str>,
) -> Result<Option<String>, EmitError> {
    root_hoist_props_with_layout(attributes, bindings, Some(line_indent), is_ts, scope_id)
}

/// The shipped `genObjectExpression`'s second multiline arm: a property
/// whose value is not a `SimpleExpression`. `class` and `style` reach
/// codegen as the objects `transformElement` normalized them into, and
/// `v-text` as a `toDisplayString` call, so a lone one of those still
/// breaks the object over lines. The single-line assembly below is the
/// first arm's `properties.length > 1` half.
fn has_non_simple_value(pieces: &[Piece<'_>]) -> bool {
    super::super::props_object::pieces_have_named(pieces, "class")
        || super::super::props_object::pieces_have_named(pieces, "style")
        || super::super::props_object::pieces_have_vue_text(pieces)
}

fn root_hoist_props_with_layout(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    multiline_indent: Option<usize>,
    is_ts: bool,
    scope_id: Option<&str>,
) -> Result<Option<String>, EmitError> {
    let scope = scope_id.filter(|scope| !attributes.iter().any(|attr| attr.name == *scope));
    if !can_root_hoist_props(attributes, bindings, is_ts)
        && !(attributes.is_empty() && bindings.is_empty() && scope.is_some())
    {
        return Ok(None);
    }

    let mut props = StdVec::new();
    let pieces = pieces(attributes, bindings, false)?;
    for (index, piece) in pieces.iter().enumerate() {
        let mut prop = String::default();
        let Some(key) = static_hoist_prop(&mut prop, piece, is_ts)? else {
            return Ok(None);
        };
        if has_prior_hoist_key(&pieces[..index], key.as_str(), is_ts)? {
            continue;
        }
        props.push(prop);
    }
    if let Some(scope_id) = scope {
        let mut prop = String::default();
        push_empty_attr_pair(&mut prop, scope_id);
        props.push(prop);
    }
    if props.is_empty() {
        return Ok(None);
    }
    if let Some(line_indent) = multiline_indent
        && (props.len() > 1 || has_non_simple_value(&pieces))
    {
        return Ok(Some(multiline_props_object(&props, line_indent)));
    }

    let mut out = String::from("{ ");
    for (index, prop) in props.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(prop.as_str());
    }
    out.push_str(" }");
    Ok(Some(out))
}

pub(in crate::emit) fn static_vnode_surface_can_hoist(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    is_ts: bool,
) -> bool {
    attributes.iter().all(|attr| attr.name != "ref")
        && bindings
            .iter()
            .all(|binding| root_binding_can_hoist(binding, is_ts))
}

fn can_root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    is_ts: bool,
) -> bool {
    (!attributes.is_empty() || !bindings.is_empty())
        && attributes.iter().all(|attr| attr.name != "ref")
        && bindings
            .iter()
            .all(|binding| root_binding_can_hoist(binding, is_ts))
}

fn root_binding_can_hoist(binding: &BindingOp<'_>, is_ts: bool) -> bool {
    let BindingOp::Bind(bind) = binding else {
        return false;
    };
    if !matches!(bind.name, Some(DynamicName::Static(_))) {
        return false;
    }
    if !bind_value_is_legacy_static_prop(bind, is_ts) {
        return false;
    }
    let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
        return false;
    };
    !matches!(key.as_str(), "ref" | "class")
}
