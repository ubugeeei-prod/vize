//! The stateful reference subset: a built L3 program and its operands are
//! admitted only in the shapes the reference renders. Anything else fails
//! closed before rendering.

use super::iteration::{self, record_argument, validate_expression};
use super::json::Json;
use super::model::{self, model_row};
use super::program::{Op, OpKind, Phase, Program};
use super::values::{Operand, for_op, literal, one};

pub fn text_binding(rows: &[Operand], op: &Op) -> bool {
    op.kind == OpKind::SetText && for_op(rows, op.id).iter().any(|r| r.role == "binding-kind")
}

/// Ops whose binding targets element `id`.
pub fn attached<'p>(program: &'p Program, rows: &[Operand], id: u32) -> Vec<&'p Op> {
    (program.ops.iter())
        .filter(|op| {
            rows.iter()
                .any(|r| r.op == op.id && r.role == "binding-kind" && r.target == Some(id))
        })
        .collect()
}

const TAGS: [&str; 9] = [
    "main", "section", "div", "span", "p", "button", "input", "select", "option",
];

pub fn static_element(rows: &[Operand], id: u32) -> Result<(String, Vec<(String, Json)>), String> {
    let tag = literal(one(rows, id, "tag")?)?;
    if !TAGS.contains(&tag) {
        return Err(format!("unsupported HTML tag {tag}"));
    }
    if literal(one(rows, id, "namespace")?)? != "html" {
        return Err("unsupported namespace".into());
    }
    let mut attrs: Vec<(String, Json)> = Vec::new();
    for row in for_op(rows, id)
        .into_iter()
        .filter(|r| r.role == "attribute")
    {
        let Some(name) = row.name.as_deref() else {
            return Err("missing attribute name".into());
        };
        let known = ["class", "id", "title", "type", "value", "multiple"];
        if !known.contains(&name) || attrs.iter().any(|(n, _)| n == name) {
            return Err("unsupported or duplicate static attribute".into());
        }
        // Form attributes are admitted only where the model reference defines them.
        if (name == "type" && tag != "input")
            || (name == "value" && !matches!(tag, "input" | "option"))
            || (name == "multiple" && tag != "select")
            || ((name == "multiple") != (row.kind == "absent"))
        {
            return Err("unsupported form attribute".into());
        }
        let value = match row.kind.as_str() {
            "absent" => "",
            _ => literal(row)?,
        };
        attrs.push((name.into(), Json::String(value.into())));
    }
    Ok((tag.into(), attrs))
}

fn has_children(program: &Program, op: u32) -> bool {
    (program.regions.iter())
        .any(|r| r.owner == Some(op) && program.ops.iter().any(|c| c.region == r.id))
}

fn validate_element_bindings(
    program: &Program,
    rows: &[Operand],
    op: &Op,
    tag: &str,
) -> Result<(), String> {
    let bindings = attached(program, rows, op.id);
    let models = bindings.iter().filter(|b| model_row(rows, b)).count();
    if matches!(tag, "input" | "select") {
        if models != 1
            || bindings
                .iter()
                .any(|b| !model_row(rows, b) && b.kind != OpKind::SetProp)
        {
            return Err(
                "form controls take one model binding plus data-id/title properties".into(),
            );
        }
    } else if models > 0 {
        return Err("unsupported model target".into());
    }
    if tag == "option" {
        let owner = program.region(op.region).and_then(|r| r.owner);
        let parent = (program.ops.iter())
            .find(|parent| Some(parent.id) == owner && parent.kind == OpKind::InsertNode);
        let parent_tag = match parent {
            Some(parent) => literal(one(rows, parent.id, "tag")?)?,
            None => "",
        };
        if parent_tag != "select" || !bindings.is_empty() {
            return Err("options must be static select children".into());
        }
        for region in program.regions.iter().filter(|r| r.owner == Some(op.id)) {
            for child in program.ops.iter().filter(|c| c.region == region.id) {
                if child.kind != OpKind::SetText || one(rows, child.id, "text")?.kind != "literal" {
                    return Err("option labels must be static text".into());
                }
            }
        }
    }
    for kind in [OpKind::SetEvent, OpKind::SetText] {
        if bindings.iter().filter(|b| b.kind == kind).count() > 1 {
            return Err("unsupported multiple bindings of the same kind".into());
        }
    }
    let mut names: Vec<String> = static_element(rows, op.id)?
        .1
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    for binding in bindings.iter().filter(|b| !model_row(rows, b)) {
        if text_binding(rows, binding) {
            if has_children(program, op.id) {
                return Err("unsupported v-text with authored children".into());
            }
            continue;
        }
        let name = literal(one(rows, binding.id, "name")?)?;
        if binding.kind == OpKind::SetProp {
            if (name == "disabled" && tag != "button")
                || !["disabled", "title", "data-id", "key"].contains(&name)
            {
                return Err("unsupported property".into());
            }
            if names.iter().any(|n| n == name) {
                return Err("duplicate property binding".into());
            }
            names.push(name.into());
            let loop_root = (program.regions.iter()).any(|region| {
                region.id == op.region
                    && (program.ops.iter())
                        .any(|owner| Some(owner.id) == region.owner && owner.kind == OpKind::Loop)
            });
            if name == "key" && !loop_root {
                return Err("key binding requires a loop root".into());
            }
            if one(rows, binding.id, "value")?.kind != "js" {
                return Err("unsupported property expression".into());
            }
        } else if binding.kind == OpKind::SetEvent {
            let value = one(rows, binding.id, "value")?;
            if tag != "button"
                || name != "click"
                || value.kind != "js"
                || (value.text != "save" && record_argument(&value.text).is_none())
            {
                return Err("unsupported event binding".into());
            }
        }
    }
    Ok(())
}

fn distinct(ids: &[u32]) -> bool {
    ids.iter()
        .enumerate()
        .all(|(i, a)| !ids.iter().take(i).any(|b| a == b))
}

fn allowed_roles(op: &Op, model: bool, text: bool) -> Result<&'static [&'static str], String> {
    Ok(match op.kind {
        OpKind::InsertNode => &["tag", "namespace", "attribute"],
        OpKind::SetProp if model => &[
            "name",
            "model-read",
            "model-write",
            "model-attribute",
            "binding-kind",
        ],
        OpKind::SetProp | OpKind::SetEvent => &["name", "value", "binding-kind"],
        OpKind::SetText if text => &["value", "binding-kind"],
        OpKind::SetText => &["text"],
        OpKind::Branch => &["condition"],
        OpKind::SlotOutlet => &["name", "attribute"],
        OpKind::Loop => &["for-source", "for-value", "for-key", "for-index"],
        _ => return Err(format!("unsupported stateful op#{}", op.id)),
    })
}

fn validate_binding(
    program: &Program,
    rows: &[Operand],
    op: &Op,
    text: bool,
) -> Result<(), String> {
    if for_op(rows, op.id).len() != if text { 2 } else { 3 } {
        return Err("unexpected binding operand count".into());
    }
    let value = one(rows, op.id, "value")?;
    if op.kind != OpKind::SetEvent {
        validate_expression(value)?;
    }
    let kind = one(rows, op.id, "binding-kind")?;
    let target_ok = program.ops.iter().any(|target| {
        Some(target.id) == value.target
            && target.region == op.region
            && (target.kind == OpKind::InsertNode
                || (target.kind == OpKind::SlotOutlet && op.kind == OpKind::SetProp && !text))
    });
    if value.target != kind.target || !target_ok {
        return Err("malformed binding target".into());
    }
    if !text && one(rows, op.id, "name")?.target != value.target {
        return Err("malformed binding name target".into());
    }
    let expected = match (text, op.kind) {
        (true, _) => "vue.text",
        (false, OpKind::SetProp) => "bind",
        _ => "on",
    };
    if literal(kind)? != expected {
        return Err("unsupported binding kind".into());
    }
    Ok(())
}

fn validate_branch(operands: &[&Operand], children: &[u32]) -> Result<(), String> {
    let regions: Vec<Option<u32>> = operands.iter().map(|r| r.region).collect();
    let owned: Vec<Option<u32>> = children.iter().map(|id| Some(*id)).collect();
    if children.is_empty() || regions != owned {
        return Err("conditions must name every owned branch region in authored order".into());
    }
    for (index, row) in operands.iter().enumerate() {
        if row.kind == "absent" {
            if index == 0 || index + 1 != operands.len() {
                return Err("else must be the final branch".into());
            }
        } else if row.kind != "js" {
            return Err("unsupported condition".into());
        } else {
            validate_expression(row)?;
        }
    }
    Ok(())
}

fn validate_slot(
    program: &Program,
    rows: &[Operand],
    op: &Op,
    operands: &[&Operand],
    children: usize,
) -> Result<(), String> {
    if children != 1 {
        return Err("unsupported slot shape".into());
    }
    if literal(one(rows, op.id, "name")?)?.is_empty() {
        return Err("unsupported slot name".into());
    }
    let mut props: Vec<String> = Vec::new();
    for row in operands.iter().filter(|r| r.role == "attribute") {
        props.push(row.name.clone().unwrap_or_default());
        literal(row)?;
    }
    for binding in attached(program, rows, op.id) {
        props.push(literal(one(rows, binding.id, "name")?)?.into());
    }
    let unique = props
        .iter()
        .enumerate()
        .all(|(i, p)| !props.iter().take(i).any(|q| q == p));
    if props.iter().any(String::is_empty) || !unique {
        return Err("unsupported or duplicate slot prop".into());
    }
    Ok(())
}

pub fn validate(program: &Program, rows: &[Operand]) -> Result<(), String> {
    if program.phase != Phase::Built {
        return Err("stateful reference requires built L3".into());
    }
    let regions: Vec<u32> = program.regions.iter().map(|r| r.id).collect();
    if !distinct(&program.ids()) || !distinct(&regions) {
        return Err("duplicate L3 identity".into());
    }
    let roots: Vec<u32> = (program.regions.iter())
        .filter(|r| r.parent.is_none())
        .map(|r| r.id)
        .collect();
    if roots != [0] {
        return Err("expected one root region #0".into());
    }
    for region in &program.regions {
        match (region.parent, region.owner) {
            (None, None) => {}
            (Some(parent), Some(owner)) => {
                let owner_ok = program.ops.iter().any(|op| {
                    op.id == owner
                        && op.region == parent
                        && [
                            OpKind::InsertNode,
                            OpKind::Branch,
                            OpKind::SlotOutlet,
                            OpKind::Loop,
                        ]
                        .contains(&op.kind)
                });
                if parent >= region.id || program.region(parent).is_none() || !owner_ok {
                    return Err("unsupported region ownership".into());
                }
            }
            _ => return Err("malformed region ownership".into()),
        }
    }
    for row in rows {
        let Some(op) = program.op(row.op) else {
            return Err("operand references unknown op".into());
        };
        if row.span.start < op.span.start || row.span.stop > op.span.stop {
            return Err("operand span escapes op".into());
        }
    }
    for op in &program.ops {
        if program.region(op.region).is_none() {
            return Err("unknown op region".into());
        }
        let operands = for_op(rows, op.id);
        let children: Vec<u32> = (program.regions.iter())
            .filter(|r| r.owner == Some(op.id))
            .map(|r| r.id)
            .collect();
        if op.kind != OpKind::Branch && children.len() > 1 {
            return Err("duplicate child region".into());
        }
        let text = text_binding(rows, op);
        let model = model_row(rows, op);
        if model {
            model::control(program, rows, op)?;
        }
        let allowed = allowed_roles(op, model, text)?;
        if operands.iter().any(|r| !allowed.contains(&r.role.as_str())) {
            return Err("unsupported operand role".into());
        }
        if op.kind == OpKind::InsertNode {
            let (tag, _) = static_element(rows, op.id)?;
            validate_element_bindings(program, rows, op, &tag)?;
        }
        if op.kind == OpKind::SetText && !text {
            validate_expression(one(rows, op.id, "text")?)?;
        }
        if !model {
            if matches!(op.kind, OpKind::SetProp | OpKind::SetEvent) || text {
                validate_binding(program, rows, op, text)?;
            } else if operands.iter().any(|r| r.target.is_some()) {
                return Err("unexpected structural target".into());
            }
        }
        match op.kind {
            OpKind::Branch => validate_branch(&operands, &children)?,
            OpKind::SlotOutlet => validate_slot(program, rows, op, &operands, children.len())?,
            OpKind::Loop => iteration::validate(program, rows, op)?,
            _ => {}
        }
    }
    Ok(())
}
