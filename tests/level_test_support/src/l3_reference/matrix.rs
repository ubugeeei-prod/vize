//! TS-28 and TS-29 committed reference outputs: the ladder traces and the
//! IVM, model, model-loop and slot matrices. Each matrix pairs generated
//! templates and scenarios (`<stem>.cases.jsonl`) with their Rust-checked
//! graph and value pages (`<stem>.lowered.jsonl`); the reference runs every
//! case and must reproduce `<stem>.behavior.jsonl` exactly. The mounted Vue
//! runtimes are gated on the same files.

use std::path::Path;

use super::behavior::run_loop;
use super::checker_corpus::{json_lines, text_field};
use super::dump::parse_program;
use super::json::{self, Json, get, object};
use super::model_behavior;
use super::program::Program;
use super::trace::{Backend, TraceEvent, run};
use super::values::{self, Operand};

pub type Runner = fn(&Program, &[Operand], &Json) -> Result<Json, String>;

/// The committed matrices: stem, runner and minimum case count.
pub const MATRICES: [(&str, Runner, usize); 4] = [
    ("ivm-matrix", run_loop, 30),
    (
        "model-reference",
        |p, r, s| model_behavior::run(p, r, s, false),
        10,
    ),
    (
        "model-loop-reference",
        |p, r, s| model_behavior::run(p, r, s, true),
        4,
    ),
    ("slot-reference", run_loop, 8),
];

/// The behavior lines of one matrix.
pub fn compute(
    runner: Runner,
    minimum: usize,
    cases: &str,
    lowered: &str,
) -> Result<String, String> {
    let cases = json_lines(cases)?;
    let lowered = json_lines(lowered)?;
    if cases.len() != lowered.len() || cases.len() < minimum {
        return Err("matrix case and Dump files are out of step".into());
    }
    let mut out = String::new();
    let mut names: Vec<&str> = Vec::new();
    for (case, graph) in cases.iter().zip(&lowered) {
        let name = text_field(case, "name")?;
        if names.contains(&name) || text_field(graph, "name")? != name {
            return Err(format!("{name}: duplicate or misaligned matrix case"));
        }
        names.push(name);
        let program = parse_program(text_field(graph, "graph")?)?;
        let rows = values::parse(text_field(graph, "values")?)?;
        let trace =
            runner(&program, &rows, get(case, "scenario")?).map_err(|e| format!("{name}: {e}"))?;
        let line = object([("name", Json::String(name.into())), ("trace", trace)]);
        out.push_str(&json::compress(&line));
        out.push('\n');
    }
    Ok(out)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Recompute `<stem>.behavior.jsonl` from its cases and lowered pages.
pub fn behavior(
    fixtures: &Path,
    stem: &str,
    runner: Runner,
    minimum: usize,
) -> Result<String, String> {
    compute(
        runner,
        minimum,
        &read(&fixtures.join(format!("{stem}.cases.jsonl")))?,
        &read(&fixtures.join(format!("{stem}.lowered.jsonl")))?,
    )
    .map_err(|e| format!("{stem}: {e}"))
}

/// The TS-28 graph ladder: hand-written and Rust-lowered pages.
pub const LADDER: [&str; 8] = [
    "static-text",
    "dynamic-button",
    "control-flow",
    "rust-lowered-static-dynamic",
    "rust-lowered-control-slots",
    "rust-lowered-loop-keyed",
    "rust-lowered-loop-unkeyed",
    "rust-lowered-loop-nested",
];

pub fn format_event(event: &TraceEvent) -> String {
    let mut line = format!(
        "{} op#{} {}",
        event.backend.name(),
        event.op_id,
        event.label()
    );
    if let Some(effect) = event.effect {
        line.push_str(&format!(" effect#{effect}"));
    }
    line
}

pub fn backend_trace_text(backend: Backend, program: &Program) -> String {
    run(backend, program)
        .iter()
        .map(|event| format_event(event) + "\n")
        .collect()
}

/// The VDOM trace, then the Vapor trace.
pub fn reference_trace_text(program: &Program) -> String {
    backend_trace_text(Backend::Vdom, program) + &backend_trace_text(Backend::Vapor, program)
}

/// `--check-fixtures` and `--check-backend-fixtures`: every ladder page
/// executes to its committed combined and per-backend traces.
pub fn check_ladder(fixtures: &Path) -> Result<usize, String> {
    for stem in LADDER {
        let program = parse_program(&read(&fixtures.join(format!("{stem}.s3.dump")))?)?;
        for (suffix, actual) in [
            ("trace", reference_trace_text(&program)),
            ("vdom.trace", backend_trace_text(Backend::Vdom, &program)),
            ("vapor.trace", backend_trace_text(Backend::Vapor, &program)),
        ] {
            if read(&fixtures.join(format!("{stem}.{suffix}")))? != actual {
                return Err(format!("{stem}.{suffix}: trace drift\nactual:\n{actual}"));
            }
        }
    }
    Ok(LADDER.len())
}

/// Recompute the behavior lines of the committed matrix `stem`.
pub fn behavior_of(fixtures: &Path, stem: &str) -> Result<String, String> {
    let Some((_, runner, minimum)) = MATRICES.iter().find(|(s, _, _)| *s == stem) else {
        return Err(format!("unknown matrix {stem}"));
    };
    behavior(fixtures, stem, *runner, *minimum)
}
