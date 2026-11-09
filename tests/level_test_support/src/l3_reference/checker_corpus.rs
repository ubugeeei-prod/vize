//! Runs the C-23 checker over every committed L3 graph: the hand-written and
//! Rust-lowered ladder pages, the TS-29 matrix and the model and slot
//! references. Every graph must be well formed, and seven targeted damages of
//! a Rust-lowered graph must each produce exactly the expected codes.

use std::path::Path;

use serde_json::Value;

use super::checker::violations;
use super::folio::parse_program;
use super::program::{EdgeKind, Op, Phase, Program, StateEdge};

pub const GRAPH_FILES: [&str; 8] = [
    "static-text",
    "dynamic-button",
    "control-flow",
    "rust-lowered-static-dynamic",
    "rust-lowered-control-slots",
    "rust-lowered-loop-keyed",
    "rust-lowered-loop-unkeyed",
    "rust-lowered-loop-nested",
];

pub const LOWERED_FILES: [&str; 3] = ["ivm-matrix", "model-reference", "slot-reference"];

/// Non-empty lines of a JSON Lines file, each parsed.
pub fn json_lines(text: &str) -> Result<Vec<Value>, String> {
    text.split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).map_err(|e| e.to_string()))
        .collect()
}

/// A string field of a JSON object.
pub fn text_field<'v>(value: &'v Value, name: &str) -> Result<&'v str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {name}"))
}

type Damage = (&'static str, Program, Vec<&'static str>);

fn damages(p: &Program) -> Result<Vec<Damage>, String> {
    let last = *p.ops.last().ok_or("empty program")?;
    let child = *p
        .regions
        .iter()
        .find(|r| r.id != 0)
        .ok_or("no child region")?;
    let grand = *p
        .regions
        .iter()
        .find(|r| r.parent == Some(child.id))
        .ok_or("no nested region")?;
    let with_ops = |f: &dyn Fn(&mut Op)| {
        let mut out = p.clone();
        out.ops.iter_mut().filter(|o| o.id == last.id).for_each(f);
        out
    };
    let cycle = {
        let mut out = p.clone();
        for r in &mut out.regions {
            if r.id == child.id {
                r.parent = Some(grand.id);
            } else if r.id == grand.id {
                r.parent = Some(child.id);
            }
        }
        out
    };
    let mut duplicate = p.clone();
    duplicate.ops.push(last);
    let mut dangling = p.clone();
    dangling.edges.push(StateEdge {
        source: last.id,
        target: 999,
        kind: EdgeKind::DataDependency,
        effect: None,
    });
    let mut backward = p.clone();
    backward.phase = Phase::Scheduled;
    backward.ops.reverse();
    Ok(vec![
        ("duplicate op id", duplicate, vec!["L3V001"]),
        (
            "op span escapes its region",
            with_ops(&|o| o.span.stop = o.span.stop.saturating_add(10000)),
            vec!["L3V006"],
        ),
        (
            "op in a missing region",
            with_ops(&|o| o.region = 999),
            vec!["L3V003", "L3V007"],
        ),
        (
            "op names a missing effect",
            with_ops(&|o| o.effect = Some(999)),
            vec!["L3V007"],
        ),
        ("dangling edge target", dangling, vec!["L3V004"]),
        ("parent cycle", cycle, vec!["L3V005", "L3V006"]),
        ("backward scheduled edges", backward, vec!["L3V008"]),
    ])
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Check every committed graph and the damage witnesses; returns the count.
pub fn check_folios(fixtures: &Path) -> Result<usize, String> {
    let mut programs: Vec<(String, String)> = Vec::new();
    for stem in GRAPH_FILES {
        let text = read(&fixtures.join(format!("{stem}.s3.folio")))?;
        programs.push((stem.into(), text));
    }
    for stem in LOWERED_FILES {
        for row in json_lines(&read(&fixtures.join(format!("{stem}.lowered.jsonl")))?)? {
            let (Ok(name), Ok(graph)) = (text_field(&row, "name"), text_field(&row, "graph"))
            else {
                return Err(format!("{stem}: malformed lowered row"));
            };
            programs.push((format!("{stem}/{name}"), graph.into()));
        }
    }
    for (name, text) in &programs {
        let codes = violations(&parse_program(text)?);
        if !codes.is_empty() {
            return Err(format!("{name}: {codes:?}"));
        }
    }
    let nested = programs
        .iter()
        .find(|(name, _)| name == "rust-lowered-loop-nested")
        .map_or("", |(_, text)| text.as_str());
    for (label, damaged, expected) in damages(&parse_program(nested)?)? {
        let actual = violations(&damaged);
        if actual != expected {
            return Err(format!("{label}: expected {expected:?}, got {actual:?}"));
        }
    }
    if programs.len() < 50 {
        return Err("folio checker corpus is too small".into());
    }
    Ok(programs.len())
}
