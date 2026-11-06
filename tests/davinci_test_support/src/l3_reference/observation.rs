//! Rendering of a validated stateful program over JSON state.

use super::expression::truthy;
use super::iteration::{self, contexts, display, evaluate};
use super::json::{self, Json, get, get_bool, get_str};
use super::model::model_row;
use super::program::{OpKind, Program};
use super::validate::{attached, static_element, text_binding, validate};
use super::values::{Operand, for_op, literal, one};
use super::view::{View, append, element, tree};

pub fn selected_branch(rows: &[Operand], context: &Json, id: u32) -> Result<Option<u32>, String> {
    for row in for_op(rows, id) {
        let selected = row.kind == "absent" || truthy(&evaluate(context, row)?);
        if selected {
            return Ok(row.region);
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, Default)]
pub struct Rendered {
    pub nodes: Vec<View>,
    /// Per button: disabled, and whether it has a click handler.
    pub buttons: Vec<(bool, bool)>,
}

impl Rendered {
    fn splice(&mut self, other: Self) {
        for node in other.nodes {
            append(&mut self.nodes, node);
        }
        self.buttons.extend(other.buttons);
    }
}

/// A slot prop as the outlet passes it: a static attribute or a bound value.
fn slot_prop(
    program: &Program,
    rows: &[Operand],
    context: &Json,
    outlet: u32,
    prop: &str,
) -> Result<Json, String> {
    if let Some(row) = for_op(rows, outlet)
        .into_iter()
        .find(|r| r.role == "attribute" && r.name.as_deref() == Some(prop))
    {
        return Ok(Json::String(literal(row)?.into()));
    }
    for binding in attached(program, rows, outlet) {
        if literal(one(rows, binding.id, "name")?)? == prop {
            return evaluate(context, one(rows, binding.id, "value")?);
        }
    }
    Err(format!("supplied slot reads missing prop {prop}"))
}

/// Supplied slot content: static text, or one displayed slot prop.
fn supplied_text(
    program: &Program,
    rows: &[Operand],
    context: &Json,
    outlet: u32,
    spec: &Json,
) -> Result<String, String> {
    match json::keys(spec)?.as_slice() {
        ["text"] => Ok(get_str(get(spec, "text")?)?.into()),
        ["prop"] => display(&slot_prop(
            program,
            rows,
            context,
            outlet,
            get_str(get(spec, "prop")?)?,
        )?),
        _ => Err("supplied slots render static text or one prop".into()),
    }
}

/// The scope rendering runs in: state, loop identities so far and supplied slots.
#[derive(Clone, Copy)]
struct Scope<'s> {
    context: &'s Json,
    identities: &'s [Json],
    slots: &'s Json,
}

fn render_element(
    program: &Program,
    rows: &[Operand],
    scope: Scope<'_>,
    fuel: usize,
    op: u32,
) -> Result<(View, Vec<(bool, bool)>), String> {
    let context = scope.context;
    let (tag, mut attrs) = static_element(rows, op)?;
    let mut disabled = false;
    let bindings = attached(program, rows, op);
    for binding in (bindings.iter()).filter(|b| b.kind == OpKind::SetProp && !model_row(rows, b)) {
        let name = literal(one(rows, binding.id, "name")?)?;
        if name == "disabled" {
            disabled = get_bool(&evaluate(context, one(rows, binding.id, "value")?)?)?;
        } else if name != "key" {
            let value = evaluate(context, one(rows, binding.id, "value")?)?;
            if value != Json::Null {
                attrs.push((name.into(), Json::String(display(&value)?)));
            }
        }
    }
    if disabled {
        attrs.push(("disabled".into(), Json::String(String::new())));
    }
    let child = program.regions.iter().find(|r| r.owner == Some(op));
    let texts: Vec<_> = bindings.iter().filter(|b| text_binding(rows, b)).collect();
    let rendered = match texts.as_slice() {
        [] => match child {
            None => Rendered::default(),
            Some(child) => render_region(program, rows, scope, fuel, child.id)?,
        },
        [binding] => {
            if child.is_some_and(|r| program.ops.iter().any(|o| o.region == r.id)) {
                return Err("unsupported v-text with authored children".into());
            }
            let text = display(&evaluate(context, one(rows, binding.id, "value")?)?)?;
            let mut nodes = Vec::new();
            append(&mut nodes, View::Text(text));
            Rendered {
                nodes,
                buttons: Vec::new(),
            }
        }
        _ => return Err("duplicate v-text binding".into()),
    };
    let events: Vec<_> = bindings
        .iter()
        .filter(|b| b.kind == OpKind::SetEvent)
        .collect();
    let event = match events.as_slice() {
        [] => None,
        [binding] => Some(iteration::event(context, one(rows, binding.id, "value")?)?),
        _ => return Err("duplicate event binding".into()),
    };
    let mut address = scope.identities.to_vec();
    address.push(Json::from(op));
    let address = json::compress(&Json::Array(address));
    let mut buttons = Vec::new();
    if tag == "button" {
        buttons.push((disabled, !events.is_empty()));
    }
    buttons.extend(rendered.buttons);
    Ok((
        element(address, tag, attrs, disabled, event, rendered.nodes),
        buttons,
    ))
}

fn render_region(
    program: &Program,
    rows: &[Operand],
    scope: Scope<'_>,
    fuel: usize,
    region: u32,
) -> Result<Rendered, String> {
    let Some(fuel) = fuel.checked_sub(1) else {
        return Err("cyclic L3 region graph".into());
    };
    let mut out = Rendered::default();
    for op in program.ops.iter().filter(|op| op.region == region) {
        match op.kind {
            OpKind::InsertNode => {
                let (node, buttons) = render_element(program, rows, scope, fuel, op.id)?;
                append(&mut out.nodes, node);
                out.buttons.extend(buttons);
            }
            OpKind::SetText => {
                if !text_binding(rows, op) {
                    let text = display(&evaluate(scope.context, one(rows, op.id, "text")?)?)?;
                    append(&mut out.nodes, View::Text(text));
                }
            }
            OpKind::SlotOutlet => {
                let name = literal(one(rows, op.id, "name")?)?;
                match get(scope.slots, name) {
                    Ok(spec) => {
                        let text = supplied_text(program, rows, scope.context, op.id, spec)?;
                        append(&mut out.nodes, View::Text(text));
                    }
                    Err(_) => {
                        let Some(fallback) =
                            program.regions.iter().find(|r| r.owner == Some(op.id))
                        else {
                            return Err("missing slot fallback".into());
                        };
                        out.splice(render_region(program, rows, scope, fuel, fallback.id)?);
                    }
                }
            }
            OpKind::Branch => {
                if let Some(selected) = selected_branch(rows, scope.context, op.id)? {
                    out.splice(render_region(program, rows, scope, fuel, selected)?);
                }
            }
            OpKind::Loop => {
                let Some(body) = program.regions.iter().find(|r| r.owner == Some(op.id)) else {
                    return Err("missing loop body".into());
                };
                for (identity, context) in contexts(program, rows, op, scope.context)? {
                    let mut identities = scope.identities.to_vec();
                    identities.push(identity);
                    let inner = Scope {
                        context: &context,
                        identities: &identities,
                        slots: scope.slots,
                    };
                    out.splice(render_region(program, rows, inner, fuel, body.id)?);
                }
            }
            OpKind::SetProp | OpKind::SetEvent => {}
            _ => return Err("unsupported stateful op".into()),
        }
    }
    Ok(out)
}

pub fn observe(
    program: &Program,
    rows: &[Operand],
    context: &Json,
    slots: &Json,
) -> Result<Rendered, String> {
    validate(program, rows)?;
    let scope = Scope {
        context,
        identities: &[],
        slots,
    };
    render_region(program, rows, scope, program.regions.len() + 1, 0)
}

pub fn render(program: &Program, rows: &[Operand], context: &Json) -> Result<Json, String> {
    Ok(tree(&observe(program, rows, context, &Json::Null)?.nodes))
}
