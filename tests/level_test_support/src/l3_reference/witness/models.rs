//! Native `v-model` and supplied-slot witnesses on the committed matrices.

use std::path::Path;

use super::{expect_error, mutate, with_steps};
use crate::l3_reference::behavior::run_loop;
use crate::l3_reference::checker_corpus::{json_lines, text_field};
use crate::l3_reference::dump::parse_program;
use crate::l3_reference::json::{self, Json, get, set};
use crate::l3_reference::model_behavior::run;
use crate::l3_reference::program::Program;
use crate::l3_reference::values::{self, Operand};

struct Case {
    program: Program,
    rows: Vec<Operand>,
    scenario: Json,
}

fn read(fixtures: &Path, file: &str) -> Result<String, String> {
    std::fs::read_to_string(fixtures.join(file)).map_err(|e| format!("{file}: {e}"))
}

fn load(fixtures: &Path, stem: &str, name: &str) -> Result<Case, String> {
    let cases = json_lines(&read(fixtures, &format!("{stem}.cases.jsonl"))?)?;
    let lowered = json_lines(&read(fixtures, &format!("{stem}.lowered.jsonl"))?)?;
    let named = |rows: &[Json]| {
        rows.iter()
            .find(|r| text_field(r, "name").ok() == Some(name))
            .cloned()
    };
    let Some(case) = named(&cases) else {
        return Err(format!("missing model case {name}"));
    };
    let Some(graph) = named(&lowered) else {
        return Err(format!("missing model graph {name}"));
    };
    Ok(Case {
        program: parse_program(text_field(&graph, "graph")?)?,
        rows: values::parse(text_field(&graph, "values")?)?,
        scenario: get(&case, "scenario")?.clone(),
    })
}

fn with_context(case: &Case, context: &str) -> Result<Json, String> {
    set(
        &set(&case.scenario, "context", json::parse(context)?)?,
        "steps",
        Json::Array(Vec::new()),
    )
}

fn negative_tests(fixtures: &Path) -> Result<(), String> {
    let stem = "model-reference";
    let text = load(fixtures, stem, "text-ime")?;
    let number = load(fixtures, stem, "text-trim-number")?;
    let single = load(fixtures, stem, "select-single")?;
    let multiple = load(fixtures, stem, "select-multiple")?;
    let toggle = load(fixtures, stem, "checkbox-boolean")?;
    let run_case = |case: &Case, script: &Json| run(&case.program, &case.rows, script, false);
    for (label, case, step) in [
        (
            "checked on a text input",
            &text,
            r#"{"event":"input","selector":"input","checked":true}"#,
        ),
        (
            "unmodelled key event",
            &text,
            r#"{"event":"keydown","selector":"input"}"#,
        ),
        (
            "missing target",
            &text,
            r#"{"event":"input","selector":"textarea"}"#,
        ),
        (
            "unknown step field",
            &text,
            r#"{"event":"input","selector":"input","bubbles":false}"#,
        ),
        (
            "fractional number",
            &number,
            r#"{"event":"input","selector":"input","value":"4.5"}"#,
        ),
        (
            "exponent number",
            &number,
            r#"{"event":"input","selector":"input","value":"1e3"}"#,
        ),
        (
            "exotic whitespace",
            &number,
            r#"{"event":"input","selector":"input","value":"\u00a042"}"#,
        ),
        (
            "selectedValues on a single select",
            &single,
            r#"{"event":"change","selector":"select","selectedValues":["a"]}"#,
        ),
        (
            "single select without a selection",
            &single,
            r#"{"event":"change","selector":"select","value":"zzz"}"#,
        ),
        (
            "value on a multiple select",
            &multiple,
            r#"{"event":"change","selector":"select","value":"a"}"#,
        ),
    ] {
        expect_error(label, run_case(case, &with_steps(&case.scenario, &[step])?))?;
    }
    expect_error(
        "string checkbox model",
        run_case(&toggle, &with_context(&toggle, r#"{"on":"yes"}"#)?),
    )?;
    expect_error(
        "scalar multiple-select model",
        run_case(&multiple, &with_context(&multiple, r#"{"picks":"a"}"#)?),
    )?;
    expect_error(
        "structured text model",
        run_case(&text, &with_context(&text, r#"{"value":[1]}"#)?),
    )?;
    let damage =
        |role: &'static str, f: &dyn Fn(&mut Operand)| mutate(&text.rows, |r| r.role == role, f);
    for (label, rows) in [
        (
            "divergent read and write paths",
            damage("model-write", &|r| r.text = "other".into()),
        ),
        (
            "compound model path",
            damage("model-read", &|r| r.text = "value || other".into()),
        ),
        (
            "unmodelled model attribute",
            damage("model-attribute", &|r| r.name = Some("true-value".into())),
        ),
        (
            "model argument",
            damage("name", &|r| {
                r.kind = "literal".into();
                r.text = "arg".into();
            }),
        ),
        (
            "element kind mismatch",
            damage("model-attribute", &|r| r.text = "select".into()),
        ),
    ] {
        expect_error(label, run(&text.program, &rows, &text.scenario, false))?;
    }
    for name in ["trim", "number"] {
        let rows = mutate(
            &number.rows,
            |row| row.role == "model-attribute" && row.name.as_deref() == Some(name),
            |row| {
                row.kind = "literal".into();
                row.text = "true".into();
            },
        );
        expect_error(
            &format!("non-absent {name} modifier"),
            run(&number.program, &rows, &number.scenario, false),
        )?;
    }
    Ok(())
}

/// Looped controls: index aliases are not assignable, and nested loops and
/// identity-free observation fail closed.
fn loop_tests(fixtures: &Path) -> Result<(), String> {
    let keyed = load(fixtures, "model-loop-reference", "keyed-text-ime")?;
    expect_error(
        "looped control without identities",
        run(&keyed.program, &keyed.rows, &keyed.scenario, false),
    )?;
    run(&keyed.program, &keyed.rows, &keyed.scenario, true)?;
    let mut index_write = mutate(
        &keyed.rows,
        |r| matches!(r.role.as_str(), "model-read" | "model-write"),
        |r| {
            r.text = "position".into();
        },
    );
    index_write = mutate(
        &index_write,
        |r| r.role == "for-key",
        |r| {
            r.kind = "js".into();
            r.text = "position".into();
        },
    );
    expect_error(
        "index alias assignment",
        run(&keyed.program, &index_write, &keyed.scenario, true),
    )?;
    let step = r#"{"event":"input","selector":"input[data-id=zzz]","value":"x"}"#;
    expect_error(
        "missing keyed target",
        run(
            &keyed.program,
            &keyed.rows,
            &with_steps(&keyed.scenario, &[step])?,
            true,
        ),
    )
}

/// A caller may supply a named slot as static text or as one displayed slot
/// prop. Any other shape, name or missing prop must be rejected.
fn slot_tests(fixtures: &Path) -> Result<(), String> {
    let named = load(fixtures, "slot-reference", "named-prop")?;
    for slots in [
        r#"{"header":{"text":1}}"#,
        r#"{"header":{"prop":"missing"}}"#,
        r#"{"header":{"text":"x","prop":"item"}}"#,
        r#"{"header":{"html":"<b>"}}"#,
        r#"{"header":{"prop":"item.label"}}"#,
        r#"{"bad name":{"text":"x"}}"#,
        "[]",
    ] {
        let script = set(&named.scenario, "slots", json::parse(slots)?)?;
        expect_error(
            &format!("supplied slots {slots}"),
            run_loop(&named.program, &named.rows, &script),
        )?;
    }
    let dynamic = mutate(
        &named.rows,
        |r| r.role == "name" && r.target.is_none(),
        |r| r.kind = "js".into(),
    );
    expect_error(
        "dynamic slot name",
        run_loop(&named.program, &dynamic, &named.scenario),
    )
}

pub fn check(fixtures: &Path) -> Result<(), String> {
    negative_tests(fixtures)?;
    loop_tests(fixtures)?;
    slot_tests(fixtures)
}
