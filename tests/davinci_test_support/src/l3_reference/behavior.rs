//! TS-28 stateful runners: the single-button scenario runner and the keyed
//! loop runner, whose trees come from the update machine.

use std::path::Path;

use super::folio::parse_program;
use super::incremental;
use super::iteration::identifier;
use super::json::{self, Json, get, get_arr, get_str, keys, object, set};
use super::observation::{observe, render};
use super::program::{OpKind, Program};
use super::values::{self, Operand};
use super::view::{State, View, click, identities, state_tree};

const STATE_RESERVED: [&str; 7] = [
    "save",
    "record",
    "$event",
    "$slots",
    "__proto__",
    "constructor",
    "prototype",
];

pub fn validate_state(context: &Json) -> Result<(), String> {
    for name in keys(context)? {
        if !identifier(name) || STATE_RESERVED.contains(&name) {
            return Err("unsupported state key".into());
        }
    }
    Ok(())
}

/// `patch` applied field by field in key order.
pub fn patched(context: &Json, patch: &Json) -> Result<Json, String> {
    let mut next = context.clone();
    for (name, value) in json::entries(patch)? {
        next = set(&next, name, value.clone())?;
    }
    Ok(next)
}

fn snapshot(tree: Json, events: &[String]) -> Json {
    let events = events.iter().cloned().map(Json::String).collect();
    object([("tree", tree), ("events", Json::Array(events))])
}

pub fn button_state(
    program: &Program,
    rows: &[Operand],
    context: &Json,
) -> Result<(bool, bool), String> {
    match observe(program, rows, context, &Json::Null)?
        .buttons
        .as_slice()
    {
        [button] => Ok(*button),
        _ => Err("expected one interaction target".into()),
    }
}

pub fn run(program: &Program, rows: &[Operand], script: &Json) -> Result<Json, String> {
    let record = rows.iter().any(|row| {
        row.role == "value"
            && row.text.starts_with("record(")
            && program
                .ops
                .iter()
                .any(|op| op.id == row.op && op.kind == OpKind::SetEvent)
    });
    if record {
        return Err("record events require the loop behavior runner".into());
    }
    if keys(script)? != ["context", "steps"] {
        return Err("unsupported scenario fields".into());
    }
    let mut context = get(script, "context")?.clone();
    validate_state(&context)?;
    let steps = get_arr(get(script, "steps")?)?;
    let mut events: Vec<String> = Vec::new();
    let mut trace = vec![snapshot(render(program, rows, &context)?, &events)];
    for step in steps {
        match keys(step)?.as_slice() {
            ["patch"] => {
                let patch = get(step, "patch")?;
                validate_state(patch)?;
                context = patched(&context, patch)?;
            }
            ["activate"] => {
                if get(step, "activate")? != &Json::String("button".into()) {
                    return Err("unsupported activation target".into());
                }
                let (disabled, handler) = button_state(program, rows, &context)?;
                if !disabled && handler {
                    events.push("save".into());
                }
            }
            ["event", "selector"] => {
                if get(step, "event")? != &Json::String("click".into())
                    || get(step, "selector")? != &Json::String("button".into())
                {
                    return Err("unsupported interaction".into());
                }
                let (disabled, handler) = button_state(program, rows, &context)?;
                if disabled {
                    return Err(
                        "disabled synthetic dispatch is outside this reference subset".into(),
                    );
                }
                if handler {
                    events.push("save".into());
                }
            }
            _ => return Err("unsupported interaction step".into()),
        }
        trace.push(snapshot(render(program, rows, &context)?, &events));
    }
    trace.push(snapshot(Json::Array(Vec::new()), &events));
    Ok(Json::Array(trace))
}

/// The graph, value page and scenario of one committed stem.
pub fn read_case(stem: &Path) -> Result<(Program, Vec<Operand>, Json), String> {
    let read = |suffix: &str| {
        let path = format!("{}.{suffix}", stem.display());
        std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))
    };
    Ok((
        parse_program(&read("s3.folio")?)?,
        values::parse(&read("values.folio")?)?,
        json::parse(&read("scenario.json")?)?,
    ))
}

/// Compare a runner's trace with the committed `<stem>.behavior.json`.
pub fn check(
    stem: &Path,
    runner: fn(&Program, &[Operand], &Json) -> Result<Json, String>,
) -> Result<(), String> {
    let (program, rows, script) = read_case(stem)?;
    let path = format!("{}.behavior.json", stem.display());
    let expected =
        json::parse(&std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?)?;
    let actual = runner(&program, &rows, &script)?;
    match actual == expected {
        true => Ok(()),
        false => Err(format!(
            "{path}: observation drift: {}",
            json::compress(&actual)
        )),
    }
}

fn loop_snapshot(state: &State, events: &[Json]) -> Json {
    object([
        ("tree", state_tree(state)),
        ("events", Json::Array(events.to_vec())),
        ("identities", identities(state)),
    ])
}

/// Render fresh, then reconcile into `state` through the update machine.
pub fn update(
    program: &Program,
    rows: &[Operand],
    context: &Json,
    state: &State,
    slots: &Json,
) -> Result<State, String> {
    let fresh: Vec<View> = observe(program, rows, context, slots)?.nodes;
    incremental::update(program.regions.len() + 2, state, &fresh).map_err(String::from)
}

/// Supplied slots as a caller passes them: static text or one displayed prop.
fn validate_slots(slots: &Json) -> Result<(), String> {
    for (name, spec) in json::entries(slots)? {
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err("unsupported supplied slot name".into());
        }
        match keys(spec)?.as_slice() {
            ["text"] => {
                get_str(get(spec, "text")?)?;
            }
            ["prop"] => {
                if !identifier(get_str(get(spec, "prop")?)?) {
                    return Err("unsupported supplied slot prop".into());
                }
            }
            _ => return Err("supplied slots render static text or one prop".into()),
        }
    }
    Ok(())
}

pub fn run_loop(program: &Program, rows: &[Operand], script: &Json) -> Result<Json, String> {
    let fields = keys(script)?;
    if fields != ["context", "steps"] && fields != ["context", "slots", "steps"] {
        return Err("unsupported loop scenario fields".into());
    }
    let slots = get(script, "slots").cloned().unwrap_or(Json::Null);
    if slots != Json::Null {
        validate_slots(&slots)?;
    }
    let mut context = get(script, "context")?.clone();
    validate_state(&context)?;
    let mut state = update(program, rows, &context, &State::default(), &slots)?;
    let mut events: Vec<Json> = Vec::new();
    let mut trace = vec![loop_snapshot(&state, &events)];
    for step in get_arr(get(script, "steps")?)? {
        match keys(step)?.as_slice() {
            ["patch"] => {
                let patch = get(step, "patch")?;
                validate_state(patch)?;
                context = patched(&context, patch)?;
                state = update(program, rows, &context, &state, &slots)?;
            }
            ["click"] => {
                if let Some(event) = click(&state, get_str(get(step, "click")?)?)? {
                    events.push(event);
                }
            }
            _ => return Err("unsupported loop interaction step".into()),
        }
        trace.push(loop_snapshot(&state, &events));
    }
    state.roots.clear();
    trace.push(loop_snapshot(&state, &events));
    Ok(Json::Array(trace))
}
