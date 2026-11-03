//! Keyed and positional loop witnesses, and the exhaustive reconciliation
//! matrix of the update machine.

use std::path::Path;

use super::{expect_error, keep, mutate, with_steps};
use crate::l3_reference::behavior::{check as check_case, read_case, run_loop};
use crate::l3_reference::incremental::update;
use crate::l3_reference::iteration::contexts;
use crate::l3_reference::json::{self, Json, get, object, set};
use crate::l3_reference::program::{OpKind, Program};
use crate::l3_reference::values::{Operand, one};
use crate::l3_reference::view::{State, View, all_targets, element};

fn plus(rows: &[Operand], extra: &[Operand]) -> Vec<Operand> {
    let mut out = rows.to_vec();
    out.extend_from_slice(extra);
    out
}

fn negative_tests(program: &Program, rows: &[Operand], script: &Json) -> Result<(), String> {
    let Some(loop_op) = program.ops.iter().find(|op| op.kind == OpKind::Loop) else {
        return Err("missing loop".into());
    };
    let source = one(rows, loop_op.id, "for-source")?.clone();
    let value = one(rows, loop_op.id, "for-value")?.clone();
    let index = one(rows, loop_op.id, "for-key")?.clone();
    let Some(key) = rows
        .iter()
        .find(|r| r.role == "name" && r.text == "key")
        .cloned()
    else {
        return Err("missing identity binding".into());
    };
    let change = |id: u32, role: &'static str, f: &dyn Fn(&mut Operand)| {
        mutate(rows, |r| r.op == id && r.role == role, f)
    };
    let state = object([
        ("items", Json::Array(Vec::new())),
        ("item", "outer".into()),
        ("position", "outer".into()),
    ]);
    let empty = set(
        &set(script, "context", state)?,
        "steps",
        Json::Array(Vec::new()),
    )?;
    let id = loop_op.id;
    for (label, damaged) in [
        ("missing source", keep(rows, |r| *r != source)),
        ("duplicate source", plus(rows, &[source.clone()])),
        ("missing value alias", keep(rows, |r| *r != value)),
        ("duplicate value alias", plus(rows, &[value.clone()])),
        (
            "same aliases",
            change(id, "for-key", &|r| r.text = value.text.clone()),
        ),
        (
            "destructured alias",
            change(id, "for-value", &|r| r.text = "{id}".into()),
        ),
        (
            "reserved alias",
            change(id, "for-value", &|r| r.text = "record".into()),
        ),
        (
            "event alias",
            change(id, "for-value", &|r| r.text = "$event".into()),
        ),
        ("missing index slot", keep(rows, |r| *r != index)),
        (
            "third array alias",
            change(id, "for-index", &|r| {
                r.kind = "js".into();
                r.text = "third".into();
            }),
        ),
        (
            "compound source",
            change(id, "for-source", &|r| r.text = "items || []".into()),
        ),
        (
            "prototype source",
            change(id, "for-source", &|r| r.text = "items.__proto__".into()),
        ),
        (
            "dynamic key name",
            change(key.op, "name", &|r| r.kind = "js".into()),
        ),
        (
            "unsupported inactive expression",
            change(key.op, "value", &|r| r.text = "item.id()".into()),
        ),
    ] {
        expect_error(label, run_loop(program, &damaged, &empty))?;
    }
    let context = get(script, "context")?;
    let with_items = |items: Json| -> Result<Json, String> {
        set(
            &set(script, "context", set(context, "items", items)?)?,
            "steps",
            Json::Array(Vec::new()),
        )
    };
    for bad in ["null", "{}", "true", "1.5", "2147483648"] {
        let item = object([("id", json::parse(bad)?), ("label", "bad".into())]);
        expect_error(
            "unsupported key",
            run_loop(program, rows, &with_items(Json::Array(vec![item]))?),
        )?;
    }
    let typed: Vec<Json> = [Json::from(1), Json::from("1")]
        .into_iter()
        .map(|id| object([("id", id), ("label", "typed".into())]))
        .collect();
    let scopes = contexts(
        program,
        rows,
        loop_op,
        &set(context, "items", Json::Array(typed))?,
    )?;
    let first = scopes.first().map(|s| &s.0);
    if scopes.len() != 2 || scopes.get(1).map(|s| &s.0) == first {
        return Err("string and numeric keys collided".into());
    }
    let item = object([("id", "same".into()), ("label", "duplicate".into())]);
    expect_error(
        "duplicate key",
        run_loop(
            program,
            rows,
            &with_items(Json::Array(vec![item.clone(), item]))?,
        ),
    )?;
    for source in [
        Json::Null,
        Json::Bool(false),
        "abc".into(),
        (-1).into(),
        1001.into(),
        object([("0", "index-like".into())]),
    ] {
        expect_error(
            "non-array source",
            run_loop(program, rows, &with_items(source)?),
        )?;
    }
    for step in [
        "{}",
        r#"{"click":"missing"}"#,
        r#"{"click":1}"#,
        r#"{"click":"a","patch":{}}"#,
        r#"{"patch":{"record":null}}"#,
        r#"{"patch":{"$event":{}}}"#,
    ] {
        expect_error(
            "invalid interaction",
            run_loop(program, rows, &with_steps(script, &[step])?),
        )?;
    }
    let Some(body) = program
        .regions
        .iter()
        .find(|r| r.owner == Some(id))
        .copied()
    else {
        return Err("missing body".into());
    };
    let mut without = program.clone();
    without.regions.retain(|r| r.id != body.id);
    expect_error("missing loop body", run_loop(&without, rows, &empty))?;
    let mut doubled = program.clone();
    doubled
        .regions
        .push(crate::l3_reference::program::Region { id: 999, ..body });
    expect_error("multiple loop bodies", run_loop(&doubled, rows, &empty))?;
    let Some(root) =
        (program.ops.iter()).find(|op| op.region == body.id && op.kind == OpKind::InsertNode)
    else {
        return Err("missing loop root".into());
    };
    let title = Operand {
        op: root.id,
        role: "attribute".into(),
        kind: "literal".into(),
        text: "static".into(),
        name: Some("title".into()),
        target: None,
        span: root.span,
        ..source.clone()
    };
    expect_error(
        "static and dynamic property collision",
        run_loop(program, &plus(rows, &[title]), &empty),
    )?;
    let Some(outer) =
        (program.ops.iter()).find(|op| op.region == 0 && op.kind == OpKind::InsertNode)
    else {
        return Err("missing outer root".into());
    };
    let outside_key = mutate(rows, |r| r.op == key.op, |r| r.target = Some(outer.id));
    let mut outside = program.clone();
    outside
        .ops
        .iter_mut()
        .filter(|op| op.id == key.op)
        .for_each(|op| op.region = 0);
    expect_error(
        "key outside loop root",
        run_loop(&outside, &outside_key, &empty),
    )
}

/// Every ordered subset of three keys: all insertion, deletion and
/// permutation pairs, including empty lists, without borrowing either
/// runtime's update algorithm or any expected output it generated.
const ARRANGEMENTS: [&[&str]; 16] = [
    &[],
    &["a"],
    &["b"],
    &["c"],
    &["a", "b"],
    &["b", "a"],
    &["a", "c"],
    &["c", "a"],
    &["b", "c"],
    &["c", "b"],
    &["a", "b", "c"],
    &["a", "c", "b"],
    &["b", "a", "c"],
    &["b", "c", "a"],
    &["c", "a", "b"],
    &["c", "b", "a"],
];

fn views(keyed: bool, keys: &[&str], suffix: &str) -> Vec<View> {
    (keys.iter().enumerate())
        .map(|(index, key)| {
            let shown = format!("{key}{suffix}");
            element(
                if keyed {
                    (*key).into()
                } else {
                    index.to_string()
                },
                "button".into(),
                vec![("data-id".into(), Json::from(*key))],
                false,
                Some(Json::from(shown.clone())),
                vec![View::Text(shown)],
            )
        })
        .collect()
}

fn reconciliation_tests() -> Result<(), String> {
    for keyed in [true, false] {
        for before in ARRANGEMENTS {
            for after in ARRANGEMENTS {
                let state = update(4, &State::default(), &views(keyed, before, "old"))?;
                let updated = update(4, &state, &views(keyed, after, "new"))?;
                let actual = all_targets(&updated);
                let mut next = u32::try_from(before.len()).map_err(|e| e.to_string())?;
                for (index, key) in after.iter().enumerate() {
                    let retained = match keyed {
                        true => before.iter().position(|b| b == key),
                        false => (index < before.len()).then_some(index),
                    };
                    let expected = match retained {
                        Some(identity) => u32::try_from(identity).map_err(|e| e.to_string())?,
                        None => {
                            next += 1;
                            next - 1
                        }
                    };
                    let Some(target) = actual.get(index) else {
                        return Err("missing reconciled target".into());
                    };
                    if target.name != *key
                        || target.identity != expected
                        || target.event != Some(Json::from(format!("{key}new")))
                    {
                        return Err("incorrect retained identity or stale event payload".into());
                    }
                }
                if actual.len() != after.len() || updated.next_identity != next {
                    return Err("incorrect allocation or removal count".into());
                }
                let cleared = update(4, &updated, &[])?;
                let reinserted = update(4, &cleared, &views(keyed, &["a"], "again"))?;
                if all_targets(&reinserted)
                    .iter()
                    .map(|t| t.identity)
                    .collect::<Vec<_>>()
                    != [next]
                {
                    return Err("retired identity was resurrected".into());
                }
            }
        }
    }
    expect_error(
        "duplicate materialized address",
        update(4, &State::default(), &views(true, &["a", "a"], "")).map_err(String::from),
    )
}

pub fn check(fixtures: &Path) -> Result<(), String> {
    for name in ["keyed", "unkeyed", "nested"] {
        check_case(
            &fixtures.join(format!("rust-lowered-loop-{name}")),
            run_loop,
        )?;
    }
    reconciliation_tests()?;
    let (program, rows, script) = read_case(&fixtures.join("rust-lowered-loop-keyed"))?;
    negative_tests(&program, &rows, &script)
}
