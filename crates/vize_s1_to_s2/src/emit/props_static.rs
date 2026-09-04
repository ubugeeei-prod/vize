//! Inline static props for native element calls.

mod hoist_pair;
mod legacy_constant;

use alloc::vec::Vec as StdVec;
use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::op::{Attribute, BindingOp, DynamicName};

use crate::pass::{StaticFacts, StaticLevel};

use super::EmitCx;
use super::EmitError;
use super::hoist::{push_attr_pair, unique_attrs};
use super::props::{Piece, pieces, static_bind_key};
use super::props_bind::StaticBindKeyCasing;
use hoist_pair::{
    bind_value_is_legacy_static_prop, component_hoist_prop, for_item_hoist_prop,
    has_for_item_legacy_global_key, has_prior_component_hoist_key, has_prior_for_item_hoist_key,
    has_prior_hoist_key, multiline_props_object, static_hoist_prop,
};

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
    should_hoist_inner(cx, id, position, false)
}

/// The same question for a component, which reaches the shipped
/// props-hoist through the component branch rather than the plain
/// element one — the branch that reads the prefixed constant rule. A
/// plain element keeps the pass's answer: a `v-bind` whose value is a
/// global still lands in `dynamicProps`, and the element branch declines
/// a props hoist once the patch flag carries `PROPS`.
pub(super) fn should_hoist_component(
    cx: &EmitCx<'_>,
    id: Option<NodeId>,
    position: PropHoistPosition,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> bool {
    let widened = prefixed_props_hoistable(cx, attributes, bindings);
    should_hoist_inner(cx, id, position, widened)
}

fn should_hoist_inner(
    cx: &EmitCx<'_>,
    id: Option<NodeId>,
    position: PropHoistPosition,
    prefixed_hoistable: bool,
) -> bool {
    let Some(fact) = id.and_then(|id| cx.facts.static_facts.get(id)) else {
        return false;
    };
    if !fact.props_hoistable && !prefixed_hoistable {
        return false;
    }
    match position {
        PropHoistPosition::Root => match fact.level {
            StaticLevel::FullyStatic | StaticLevel::HasDynamicText => true,
            StaticLevel::NotStatic => {
                fact.foreign || fact.nested_static || inline_root_arm(cx, fact)
            }
        },
        PropHoistPosition::Nested => {
            fact.level == StaticLevel::NotStatic && (fact.foreign || fact.nested_static)
        }
        PropHoistPosition::ForItem => true,
    }
}

/// The pass publishes `props_hoistable` under the *unprefixed* constant
/// rule, and by its own contract stays option-free: any identifier
/// reference makes an expression dynamic there. The shipped lane reads
/// an expression's constness off `processExpression`, which only runs
/// with `prefix_identifiers` — so with prefixing on a prop the pass
/// called dynamic gets asked again, against the same runtime-dependency
/// rule the emit already ports for `normalizeStyle`. That is how
/// `:range="[new Date(2019, 2, 4), new Date(2019, 2, 24)]"` hoists: its
/// only free name is an allowed global.
fn prefixed_props_hoistable(
    cx: &EmitCx<'_>,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> bool {
    cx.scope.prefixes_identifiers()
        && !(attributes.is_empty() && bindings.is_empty())
        && attributes.iter().all(|attribute| attribute.name != "ref")
        && bindings
            .iter()
            .all(|binding| prefixed_hoistable_binding(cx, binding))
}

/// The pass's `hoistable_binding` shape gates, with its value question
/// answered by the prefixed rule instead of the unprefixed walk.
fn prefixed_hoistable_binding(cx: &EmitCx<'_>, binding: &BindingOp<'_>) -> bool {
    let BindingOp::Bind(bind) = binding else {
        return false;
    };
    if !matches!(bind.name, Some(DynamicName::Static(_))) {
        return false;
    }
    let Ok(key) = static_bind_key(bind, StaticBindKeyCasing::Preserve) else {
        return false;
    };
    if matches!(key.as_str(), "ref" | "class") {
        return false;
    }
    let Some(vize_s2::expr::ExprRef::Js(js)) = bind.value else {
        return false;
    };
    super::constant_expr::is_constant_expression(js.ast, &cx.scope)
}

/// The shipped `hoist_static_inner` arm only an inlined render function
/// reaches: `is_root && ctx.options.inline && has_only_native_element_descendants(el)`.
/// Position is the caller's ([`PropHoistPosition::Root`]), the predicate
/// is the pass's, and the option is the emit's — so a template whose
/// root has static props and no component/structural descendant hoists
/// them where a non-inlined render function would keep them inline.
fn inline_root_arm(cx: &EmitCx<'_>, fact: &StaticFacts) -> bool {
    cx.scope.inline() && fact.native_descendants
}

/// The same arm asked as its own question, for the component gate.
/// A component's own hoist decision is reconstructed from the codegen
/// shape it ends up in (slots, builtin helpers, array children); the
/// inline root arm sits *before* all of that in the shipped lane — the
/// transform hoists the props and codegen then always spells them
/// `_hoisted_N` — so the component gate takes it as a separate
/// disjunct rather than through that reconstruction.
pub(super) fn inline_root_hoist(
    cx: &EmitCx<'_>,
    id: Option<NodeId>,
    position: PropHoistPosition,
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
) -> bool {
    let widened = prefixed_props_hoistable(cx, attributes, bindings);
    matches!(position, PropHoistPosition::Root)
        && id
            .and_then(|id| cx.facts.static_facts.get(id))
            .is_some_and(|fact| (fact.props_hoistable || widened) && inline_root_arm(cx, fact))
}

pub(super) fn props_hoistable(cx: &EmitCx<'_>, id: Option<NodeId>) -> bool {
    id.and_then(|id| cx.facts.static_facts.get(id))
        .is_some_and(|fact| fact.props_hoistable)
}

pub(super) fn has_legacy_global_for_item_key(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(has_for_item_legacy_global_key)
}

pub(super) fn root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    is_ts: bool,
) -> Result<Option<String>, EmitError> {
    root_hoist_props_with_layout(attributes, bindings, None, is_ts)
}

pub(super) fn cached_root_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    line_indent: usize,
    is_ts: bool,
) -> Result<Option<String>, EmitError> {
    root_hoist_props_with_layout(attributes, bindings, Some(line_indent), is_ts)
}

/// The shipped `genObjectExpression`'s second multiline arm: a property
/// whose value is not a `SimpleExpression`. `class` and `style` reach
/// codegen as the objects `transformElement` normalized them into, and
/// `v-text` as a `toDisplayString` call, so a lone one of those still
/// breaks the object over lines. The single-line assembly below is the
/// first arm's `properties.length > 1` half.
fn has_non_simple_value(pieces: &[Piece<'_>]) -> bool {
    super::props_object::pieces_have_named(pieces, "class")
        || super::props_object::pieces_have_named(pieces, "style")
        || super::props_object::pieces_have_vue_text(pieces)
}

fn root_hoist_props_with_layout(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    multiline_indent: Option<usize>,
    is_ts: bool,
) -> Result<Option<String>, EmitError> {
    if !can_root_hoist_props(attributes, bindings, is_ts) {
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

pub(super) struct ComponentHoistProps {
    pub(super) source: String,
    pub(super) dynamic_values: bool,
    pub(super) non_key: bool,
    pub(super) valued_prop: bool,
    pub(super) all_static_binds: bool,
}

pub(super) fn component_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    is_ts: bool,
) -> Result<Option<ComponentHoistProps>, EmitError> {
    let pieces = pieces(attributes, bindings, false)?;
    let mut out = String::from("{ ");
    let mut emitted = 0usize;
    let mut dynamic_values = false;
    let mut non_key = false;
    let mut valued_prop = false;
    let mut all_static_binds = true;
    for (index, piece) in pieces.iter().enumerate() {
        let mut prop = String::default();
        let Some((key, dynamic_value)) = component_hoist_prop(&mut prop, piece, is_ts)? else {
            return Ok(None);
        };
        if has_prior_component_hoist_key(&pieces[..index], key.as_str(), is_ts)? {
            continue;
        }
        dynamic_values |= dynamic_value;
        non_key |= key.as_str() != "key";
        valued_prop |= match piece {
            Piece::Attr(attr) => attr.value.is_some(),
            Piece::Bind(_) => true,
            _ => false,
        };
        all_static_binds &= matches!(piece, Piece::Bind(_)) && !dynamic_value;
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
        all_static_binds,
    }))
}

pub(super) fn for_item_hoist_props(
    attributes: &[Attribute<'_>],
    bindings: &[BindingOp<'_>],
    is_ts: bool,
) -> Result<Option<String>, EmitError> {
    let pieces = pieces(attributes, bindings, false)?;
    let mut out = String::from("{ ");
    let mut emitted = 0usize;
    for (index, piece) in pieces.iter().enumerate() {
        let mut prop = String::default();
        let Some(key) = for_item_hoist_prop(&mut prop, piece, is_ts)? else {
            return Ok(None);
        };
        if has_prior_for_item_hoist_key(&pieces[..index], key.as_str(), is_ts)? {
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

pub(super) fn static_vnode_surface_can_hoist(
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
    if attributes.is_empty() && bindings.is_empty() {
        return false;
    }

    if attributes.iter().any(|attr| attr.name == "ref") {
        return false;
    }

    bindings
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

/// The inline lane's `ref_key` pair: `ref="name"` naming a writable
/// setup binding is emitted as `ref_key: "name", ref: name` so the
/// runtime's `setRef` writes back into `instance.refs`.
fn inline_template_ref<'a>(cx: &EmitCx<'_>, attr: &'a Attribute<'a>) -> Option<&'a str> {
    (attr.name == "ref")
        .then_some(attr.value)
        .flatten()
        .filter(|name| cx.scope.writes_template_ref(name))
}

pub(super) fn emit_inline<'a>(
    cx: &mut EmitCx<'_>,
    attributes: impl Iterator<Item = &'a Attribute<'a>>,
    force_multiline: bool,
) {
    let unique = unique_attrs(attributes);
    let multiline = unique.len() > 1 || force_multiline;
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
        if let Some(name) = inline_template_ref(cx, attr) {
            cx.buf.push("ref_key: \"");
            cx.buf.push(name);
            cx.buf.push("\", ref: ");
            cx.buf.push(name);
            continue;
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
