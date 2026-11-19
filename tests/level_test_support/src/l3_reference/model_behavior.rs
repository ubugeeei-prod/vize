//! Stateful runner for native `v-model` scenarios. Rendering and element
//! identity come from the update machine. Each control's live DOM state is
//! keyed by that identity: the mounted hook creates it, update hooks patch it
//! only on a reactive change, and event steps mutate it as the mounted Vue
//! runner does. Observations add the live `value`, `checked` and `selected`
//! properties exactly where the mounted observer reads them.

use super::behavior::validate_state;
use super::expression::strict_equal;
use super::incremental;
use super::iteration::{alias, contexts, lookup, path};
use super::json::{self, Json, get, get_arr, get_nat, get_str, keys, set};
use super::model::{self, Control, Kind, Live, model_row};
use super::model_events::{apply_fields, dispatch};
use super::observation::observe;
use super::program::{OpKind, Program};
use super::validate::validate;
use super::values::{Operand, for_op, one};
use super::view::{Node, State, identities as state_identities, object};

struct Runner<'p> {
    program: &'p Program,
    rows: &'p [Operand],
    controls: Vec<(u32, Control)>,
    context: Json,
    state: State,
    forms: Vec<(u32, Live)>,
}

/// A live control instance: identity, control and L3 address.
type Instance = (u32, Control, String);

fn op_of(address: &str) -> Result<u64, String> {
    let parts = json::parse(address)?;
    let Some(last) = get_arr(&parts)?.last() else {
        return Err("empty L3 address".into());
    };
    get_nat(last)
}

fn control_nodes(runner: &Runner<'_>, node: &Node, out: &mut Vec<Instance>) -> Result<(), String> {
    let Node::Element {
        address,
        identity,
        children,
        ..
    } = node
    else {
        return Ok(());
    };
    let op = op_of(address)?;
    if let Some((_, control)) = runner.controls.iter().find(|(id, _)| u64::from(*id) == op) {
        out.push((*identity, control.clone(), address.clone()));
    }
    for child in children {
        control_nodes(runner, child, out)?;
    }
    Ok(())
}

fn live_controls(runner: &Runner<'_>) -> Result<Vec<Instance>, String> {
    let mut out = Vec::new();
    for root in &runner.state.roots {
        control_nodes(runner, root, &mut out)?;
    }
    Ok(out)
}

/// Read scope and root write path of one control instance. Inside a single,
/// non-nested array loop, a path through the value alias writes into the
/// source element at the instance's current index, as Vue's item proxy does.
fn instance_scope(
    runner: &Runner<'_>,
    address: &str,
    control: &Control,
) -> Result<(Json, Vec<String>), String> {
    let fields: Vec<String> = control.path.split('.').map(String::from).collect();
    let parts = json::parse(address)?;
    let parts = get_arr(&parts)?;
    let scopes = parts
        .get(..parts.len().saturating_sub(1))
        .unwrap_or_default();
    let entry = match scopes {
        [] => return Ok((runner.context.clone(), fields)),
        [entry] => entry,
        _ => return Err("model controls in nested loops are outside the reference subset".into()),
    };
    let loop_id = u32::try_from(get_nat(
        get_arr(entry)?.first().ok_or("index out of bounds")?,
    )?)
    .map_err(|_| String::from("L3 integer exceeds u32"))?;
    let Some(loop_op) = runner.program.op(loop_id) else {
        return Err("missing loop op".into());
    };
    let source = one(runner.rows, loop_id, "for-source")?;
    if source.kind != "js" || !path(&source.text) {
        return Err("model loops require a path source".into());
    }
    let value_alias = alias(one(runner.rows, loop_id, "for-value")?)?;
    let others: Vec<&str> = for_op(runner.rows, loop_id)
        .into_iter()
        .filter(|r| matches!(r.role.as_str(), "for-key" | "for-index") && r.kind != "absent")
        .map(|r| r.text.as_str())
        .collect();
    let head = fields.first().map(String::as_str);
    if head.is_some_and(|h| others.contains(&h)) {
        return Err("models cannot assign loop index aliases".into());
    }
    let scopes = contexts(runner.program, runner.rows, loop_op, &runner.context)?;
    let Some((index, (_, scope))) = scopes
        .into_iter()
        .enumerate()
        .find(|(_, (id, _))| id == entry)
    else {
        return Err("control scope is not live".into());
    };
    if head != Some(value_alias) {
        return Ok((scope, fields));
    }
    get_arr(&lookup(&runner.context, &source.text)?)?;
    let mut target: Vec<String> = source.text.split('.').map(String::from).collect();
    target.push(index.to_string());
    target.extend(fields.into_iter().skip(1));
    Ok((scope, target))
}

/// Re-render through the update machine, then run mount and update hooks.
fn rerender(mut runner: Runner<'_>, assigning: Option<u32>) -> Result<Runner<'_>, String> {
    let fresh = observe(runner.program, runner.rows, &runner.context, &Json::Null)?;
    runner.state = incremental::update(
        runner.program.regions.len() + 2,
        &runner.state,
        &fresh.nodes,
    )?;
    let mut forms = Vec::new();
    for (identity, control, address) in live_controls(&runner)? {
        let model = lookup(
            &instance_scope(&runner, &address, &control)?.0,
            &control.path,
        )?;
        let live = match runner.forms.iter().find(|(id, _)| *id == identity) {
            Some((_, live)) => model::update(&control, live, &model, assigning == Some(identity))?,
            None => model::mount(&control, &model)?,
        };
        forms.push((identity, live));
    }
    runner.forms = forms;
    Ok(runner)
}

/// Reactive assignments trigger only when `Object.is` would see a change.
fn changed(old: Option<&Json>, new: &Json) -> bool {
    old.is_none_or(|old| !strict_equal(old, new).unwrap_or(false))
}

fn set_path(value: &Json, fields: &[String], next: Json) -> Result<Json, String> {
    let Some((field, rest)) = fields.split_first() else {
        return Ok(next);
    };
    let numeric = !field.is_empty() && field.bytes().all(|b| b.is_ascii_digit());
    let index = numeric.then(|| field.parse::<usize>().ok()).flatten();
    match (value, index) {
        (Json::Array(items), Some(index)) => {
            let Some(inner) = items.get(index) else {
                return Err("array write out of range".into());
            };
            let mut items = items.clone();
            let updated = set_path(inner, rest, next)?;
            if let Some(slot) = items.get_mut(index) {
                *slot = updated;
            }
            Ok(Json::Array(items))
        }
        _ => {
            json::get_obj(value)?;
            match rest.is_empty() {
                true => set(value, field, next),
                false => set(value, field, set_path(get(value, field)?, rest, next)?),
            }
        }
    }
}

fn assign<'p>(
    mut runner: Runner<'p>,
    control: &Control,
    address: &str,
    next: Json,
) -> Result<(Runner<'p>, bool), String> {
    let (scope, target) = instance_scope(&runner, address, control)?;
    let old = lookup(&scope, &control.path).ok();
    runner.context = set_path(&runner.context, &target, next.clone())?;
    Ok((runner, changed(old.as_ref(), &next)))
}

/// Selector subset: `tag`, `tag[value=...]` or `tag[data-id=...]`.
fn selects(selector: &str, node: &Node) -> bool {
    let Node::Element { tag, payload, .. } = node else {
        return false;
    };
    match selector.split('[').collect::<Vec<_>>().as_slice() {
        [name] => name == tag,
        [name, rest] => match rest.split('=').collect::<Vec<_>>().as_slice() {
            [attr, value] => {
                let mut chars = value.chars();
                chars.next_back();
                let wanted = chars.as_str().replace('"', "");
                let found = payload
                    .attributes
                    .iter()
                    .find(|(k, _)| k == attr)
                    .map(|(_, v)| v);
                matches!(*attr, "value" | "data-id")
                    && value.ends_with(']')
                    && name == tag
                    && found == Some(&Json::String(wanted))
            }
            _ => false,
        },
        _ => false,
    }
}

fn find<'n>(selector: &str, node: &'n Node, out: &mut Vec<&'n Node>) {
    if let Node::Element { children, .. } = node {
        if selects(selector, node) {
            out.push(node);
        }
        for child in children {
            find(selector, child, out);
        }
    }
}

const STEP_FIELDS: [&str; 5] = ["event", "selector", "value", "checked", "selectedValues"];

fn event<'p>(runner: Runner<'p>, step: &Json) -> Result<Runner<'p>, String> {
    if keys(step)?.iter().any(|f| !STEP_FIELDS.contains(f)) {
        return Err("unsupported model interaction field".into());
    }
    let name = get_str(get(step, "event")?)?;
    let selector = get_str(get(step, "selector")?)?;
    let mut found = Vec::new();
    for root in &runner.state.roots {
        find(selector, root, &mut found);
    }
    let Some(Node::Element { identity, .. }) = found.first() else {
        return Err("missing interaction target".into());
    };
    let identity = *identity;
    let instances = live_controls(&runner)?;
    let Some((_, control, address)) = instances.into_iter().find(|(id, _, _)| *id == identity)
    else {
        return Err("target is not a model control".into());
    };
    let Some((_, live)) = runner.forms.iter().find(|(id, _)| *id == identity) else {
        return Err("control has no live state".into());
    };
    let mut live = live.clone();
    apply_fields(&control, &mut live, step)?;
    let assigned = dispatch(&control, &mut live, name)?;
    let mut runner = runner;
    for (id, state) in &mut runner.forms {
        if *id == identity {
            *state = live.clone();
        }
    }
    let Some(next) = assigned else {
        return Ok(runner);
    };
    let (runner, did_change) = assign(runner, &control, &address, next)?;
    if !did_change {
        return Ok(runner);
    }
    let assigning = (control.kind == Kind::Select).then_some(identity);
    rerender(runner, assigning)
}

/// Observation JSON with live form properties; `selection` threads the
/// enclosing select's option states through its descendants.
fn node_json(
    runner: &Runner<'_>,
    pairs: &[(u32, Control)],
    selection: Vec<bool>,
    node: &Node,
) -> (Json, Vec<bool>) {
    let (identity, tag, payload, children) = match node {
        Node::Text(value) => return (Json::String(value.clone()), selection),
        Node::Element {
            identity,
            tag,
            payload,
            children,
            ..
        } => (identity, tag, payload, children),
    };
    let live = (runner.forms.iter())
        .find(|(id, _)| id == identity)
        .map(|(_, live)| live.clone())
        .unwrap_or_default();
    let control = pairs.iter().find(|(id, _)| id == identity).map(|(_, c)| c);
    let (selection, own) = match (tag.as_str(), selection.split_first()) {
        ("option", Some((selected, rest))) => (
            rest.to_vec(),
            vec![("selected".to_owned(), Json::Bool(*selected))],
        ),
        _ => (selection, Vec::new()),
    };
    let mut sel = if tag == "select" {
        live.selected.clone()
    } else {
        Vec::new()
    };
    let mut kids = Vec::new();
    for child in children {
        let (json, next) = node_json(runner, pairs, sel, child);
        kids.push(json);
        sel = next;
    }
    let form = match (tag.as_str(), control) {
        ("input", Some(control)) => vec![
            (
                "value".to_owned(),
                Json::String(model::display_value(control, &live)),
            ),
            ("checked".to_owned(), Json::Bool(live.checked)),
        ],
        ("select", Some(control)) => vec![(
            "value".to_owned(),
            Json::String(model::display_value(control, &live)),
        )],
        ("button", _) => vec![("disabled".to_owned(), Json::Bool(payload.disabled))],
        _ => Vec::new(),
    };
    let mut fields = vec![
        ("tag".to_owned(), Json::String(tag.clone())),
        ("attributes".to_owned(), object(&payload.attributes)),
        ("children".to_owned(), Json::Array(kids)),
    ];
    fields.extend(own);
    fields.extend(form);
    (object(&fields), selection)
}

fn snapshot(runner: &Runner<'_>, identities: bool) -> Result<Json, String> {
    let pairs: Vec<(u32, Control)> = live_controls(runner)?
        .into_iter()
        .map(|(id, c, _)| (id, c))
        .collect();
    let tree = (runner.state.roots.iter())
        .map(|node| node_json(runner, &pairs, Vec::new(), node).0)
        .collect();
    let mut fields = vec![
        ("tree".to_owned(), Json::Array(tree)),
        ("events".to_owned(), Json::Array(Vec::new())),
    ];
    if identities {
        fields.push(("identities".to_owned(), state_identities(&runner.state)));
    }
    Ok(object(&fields))
}

fn inside_loop(program: &Program, region: u32, fuel: usize) -> bool {
    let Some(fuel) = fuel.checked_sub(1) else {
        return true;
    };
    match program.region(region) {
        Some(found) => match (found.parent, found.owner) {
            (Some(parent), Some(owner)) => {
                program
                    .ops
                    .iter()
                    .any(|op| op.id == owner && op.kind == OpKind::Loop)
                    || inside_loop(program, parent, fuel)
            }
            _ => false,
        },
        None => false,
    }
}

/// Run one model scenario; looped controls need `identities`.
pub fn run(
    program: &Program,
    rows: &[Operand],
    script: &Json,
    identities: bool,
) -> Result<Json, String> {
    if keys(script)? != ["context", "steps"] {
        return Err("unsupported model scenario fields".into());
    }
    let context = get(script, "context")?.clone();
    validate_state(&context)?;
    validate(program, rows)?;
    let mut controls = Vec::new();
    for op in program.ops.iter().filter(|op| model_row(rows, op)) {
        let control = model::control(program, rows, op)?;
        let Some(target) = for_op(rows, op.id).iter().find_map(|r| r.target) else {
            return Err("untargeted model".into());
        };
        let Some(element) = program.op(target) else {
            return Err("missing model target".into());
        };
        if inside_loop(program, element.region, program.regions.len() + 1) && !identities {
            return Err("looped model controls are observed with element identities".into());
        }
        controls.push((target, control));
    }
    let runner = Runner {
        program,
        rows,
        controls,
        context,
        state: State::default(),
        forms: Vec::new(),
    };
    let mut runner = rerender(runner, None)?;
    let mut trace = vec![snapshot(&runner, identities)?];
    for step in get_arr(get(script, "steps")?)? {
        let fields = keys(step)?;
        if fields == ["patch"] {
            let patch = get(step, "patch")?;
            validate_state(patch)?;
            let mut dirty = false;
            for (name, value) in json::entries(patch)? {
                dirty = dirty || changed(get(&runner.context, name).ok(), value);
                runner.context = set(&runner.context, name, value.clone())?;
            }
            if dirty {
                runner = rerender(runner, None)?;
            }
        } else if fields.contains(&"event") {
            runner = event(runner, step)?;
        } else {
            return Err("unsupported model step".into());
        }
        trace.push(snapshot(&runner, identities)?);
    }
    let mut last = vec![
        ("tree".to_owned(), Json::Array(Vec::new())),
        ("events".to_owned(), Json::Array(Vec::new())),
    ];
    if identities {
        last.push(("identities".to_owned(), Json::Array(Vec::new())));
    }
    trace.push(object(&last));
    Ok(Json::Array(trace))
}
