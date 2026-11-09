//! Fail-closed witnesses: each damages one committed case (graph, operands,
//! state or scenario) so that it leaves the modelled contract, and requires a
//! reference error rather than a guess. Positive variants pin observations.

mod branches;
pub mod control;
pub mod loops;
pub mod models;

use std::path::Path;

use super::behavior::{read_case, run};
use super::expression;
use super::json::{self, Json, get, get_arr, get_str, set};
use super::program::{Program, StateEdge};
use super::values::{self, KINDS, Operand, ROLES};

pub fn expect_error<T>(label: &str, result: Result<T, String>) -> Result<(), String> {
    match result {
        Err(_) => Ok(()),
        Ok(_) => Err(format!("negative witness accepted: {label}")),
    }
}

pub fn mutate(
    rows: &[Operand],
    hit: impl Fn(&Operand) -> bool,
    f: impl Fn(&mut Operand),
) -> Vec<Operand> {
    let mut rows = rows.to_vec();
    rows.iter_mut().filter(|r| hit(r)).for_each(f);
    rows
}

pub fn keep(rows: &[Operand], hit: impl Fn(&Operand) -> bool) -> Vec<Operand> {
    rows.iter().filter(|r| hit(r)).cloned().collect()
}

pub fn with_steps(script: &Json, steps: &[&str]) -> Result<Json, String> {
    let steps = steps
        .iter()
        .map(|s| json::parse(s))
        .collect::<Result<Vec<_>, _>>()?;
    set(script, "steps", Json::Array(steps))
}

fn parse_row(fields: Vec<Json>) -> Result<Operand, String> {
    values::parse_operand(&format!("operand={}", json::compress(&Json::Array(fields))))
}

const TARGETED: [&str; 7] = [
    "binding-kind",
    "value",
    "modifier",
    "model-read",
    "model-write",
    "model-attribute",
    "params",
];

fn parser_tests() -> Result<(), String> {
    for role in ROLES {
        for kind in KINDS {
            let target = if TARGETED.contains(&role) {
                Json::from(1)
            } else {
                Json::Null
            };
            let region = if role == "condition" {
                Json::from(1)
            } else {
                Json::Null
            };
            let name = match role {
                "attribute" | "model-attribute" => Json::from("class"),
                _ => Json::Null,
            };
            let text = if kind == "absent" {
                String::new()
            } else {
                String::from("quoted\"\\\n\t\u{96ea}")
            };
            let qualifier = if matches!(kind, "opaque" | "foreign") {
                "dialect"
            } else {
                ""
            };
            let row = parse_row(vec![
                0.into(),
                role.into(),
                target,
                region,
                name,
                kind.into(),
                text.clone().into(),
                qualifier.into(),
                0.into(),
                20.into(),
            ])?;
            if row.role != role
                || row.kind != kind
                || row.text != text
                || row.qualifier != qualifier
            {
                return Err("L3 operand parsing lost authored payload".into());
            }
        }
    }
    let valid: Vec<Json> = vec![
        0.into(),
        "text".into(),
        Json::Null,
        Json::Null,
        Json::Null,
        "literal".into(),
        "hello".into(),
        "".into(),
        0.into(),
        5.into(),
    ];
    let damaged: [(usize, Json); 12] = [
        (0, (-1).into()),
        (0, 4_294_967_296_u64.into()),
        (1, "unknown".into()),
        (2, 1.into()),
        (3, 1.into()),
        (4, "extra".into()),
        (5, "unknown".into()),
        (5, "absent".into()),
        (5, "opaque".into()),
        (5, "foreign".into()),
        (7, "extra".into()),
        (8, 6.into()),
    ];
    for (index, value) in damaged {
        let mut fields = valid.clone();
        if let Some(slot) = fields.get_mut(index) {
            *slot = value;
        }
        expect_error(&format!("malformed field {index}"), parse_row(fields))?;
    }
    expect_error("short tuple", parse_row(Vec::new()))?;
    let mut long = valid.clone();
    long.push(0.into());
    expect_error("long tuple", parse_row(long))?;
    for role in [
        "value",
        "binding-kind",
        "model-attribute",
        "condition",
        "attribute",
    ] {
        let mut fields = valid.clone();
        if let Some(slot) = fields.get_mut(1) {
            *slot = role.into();
        }
        expect_error("missing reference", parse_row(fields))?;
    }
    for text in [
        "",
        "operand=[]",
        "[s3-values-dump]\n",
        "[s3-values-dump]\n[unknown]",
    ] {
        expect_error("malformed value page", values::parse(text))?;
    }
    Ok(())
}

fn reference_tests(program: &Program, rows: &[Operand], script: &Json) -> Result<(), String> {
    let with_handler = run(program, rows, script)?;
    let mut no_handler = program.clone();
    no_handler.ops.retain(|op| op.id != 3);
    no_handler
        .edges
        .retain(|e: &StateEdge| e.source != 3 && e.target != 3);
    no_handler.effects.retain(|e| e.owner != 3);
    let expected = Json::Array(
        get_arr(&with_handler)?
            .iter()
            .map(|snapshot| set(snapshot, "events", Json::Array(Vec::new())))
            .collect::<Result<_, _>>()?,
    );
    if run(&no_handler, &keep(rows, |r| r.op != 3), script)? != expected {
        return Err(
            "handler-free activation must preserve the full trace without save events".into(),
        );
    }
    let mut duplicate = program.clone();
    duplicate
        .ops
        .extend(program.ops.iter().filter(|op| op.id == 3).map(|op| {
            let mut op = *op;
            op.id = 99;
            op
        }));
    let mut duplicate_rows = rows.to_vec();
    duplicate_rows.extend(mutate(&keep(rows, |r| r.op == 3), |_| true, |r| r.op = 99));
    expect_error(
        "multiple click handlers",
        run(&duplicate, &duplicate_rows, script),
    )?;
    let texted = |f: &dyn Fn(&mut Operand)| mutate(rows, |r| r.role == "text", f);
    for kind in ["opaque", "foreign", "vue.filter", "absent"] {
        expect_error(
            "unsupported expression kind",
            run(program, &texted(&|r| r.kind = kind.into()), script),
        )?;
    }
    for text in [
        "label == '!'",
        "label / 2",
        "label()",
        "label[0]",
        "label ?? 'x'",
        "1.5",
        "typeof label",
        "label.__proto__",
        "'\\u0041'",
        "undefined",
        "label,label",
        "`x`",
    ] {
        expect_error(
            &format!("unsupported expression {text}"),
            run(program, &texted(&|r| r.text = text.into()), script),
        )?;
    }
    expect_error(
        "missing text",
        run(program, &keep(rows, |r| r.role != "text"), script),
    )?;
    let mut doubled = rows.to_vec();
    doubled.extend(keep(rows, |r| r.role == "text"));
    expect_error("duplicate text", run(program, &doubled, script))?;
    expect_error(
        "dangling operand",
        run(program, &mutate(rows, |_| true, |r| r.op = 999), script),
    )?;
    let mut looped = program.clone();
    looped
        .ops
        .iter_mut()
        .filter(|op| op.id == 4)
        .for_each(|op| op.kind = super::program::OpKind::Loop);
    expect_error("unsupported op", run(&looped, rows, script))?;
    let mut cyclic = program.clone();
    cyclic
        .regions
        .iter_mut()
        .filter(|r| r.id == 1)
        .for_each(|r| r.parent = Some(2));
    expect_error("cyclic region", run(&cyclic, rows, script))?;
    let mut scheduled = program.clone();
    scheduled.phase = super::program::Phase::Scheduled;
    expect_error("unsupported phase", run(&scheduled, rows, script))?;
    expect_error(
        "wrong binding target",
        run(
            program,
            &mutate(rows, |r| r.role == "value", |r| r.target = Some(999)),
            script,
        ),
    )?;
    expect_error(
        "unsupported handler",
        run(
            program,
            &mutate(
                rows,
                |r| r.op == 3 && r.role == "value",
                |r| r.text = "other".into(),
            ),
            script,
        ),
    )?;
    for text in [
        r#"{"context":{},"steps":[]}"#,
        r#"{"context":{"locked":true,"label":{}},"steps":[]}"#,
        r#"{"context":{"locked":"false","label":"x"},"steps":[]}"#,
        r#"{"context":{"locked":false,"label":1.5},"steps":[]}"#,
        r#"{"context":{"locked":false,"label":2147483648},"steps":[]}"#,
        r#"{"context":{"locked":false,"label":"x","save":null},"steps":[]}"#,
    ] {
        expect_error("unsupported state", run(program, rows, &json::parse(text)?))?;
    }
    // Activation is supported when locked; synthetic dispatch remains excluded.
    for text in [
        "{}",
        r#"{"patch":{},"ignored":true}"#,
        r#"{"patch":{"save":null}}"#,
        r#"{"activate":"missing"}"#,
        r#"{"activate":false}"#,
        r#"{"activate":"button","event":"click"}"#,
        r#"{"activate":"button","patch":{}}"#,
        r#"{"event":"change","selector":"button"}"#,
        r#"{"event":"click","selector":"missing"}"#,
        r#"{"event":"click","selector":"button"}"#,
    ] {
        expect_error(
            "unsupported step",
            run(program, rows, &with_steps(script, &[text])?),
        )?;
    }
    Ok(())
}

/// TS-28 operand-page parser and single-button witnesses.
pub fn behavior(fixtures: &Path) -> Result<(), String> {
    parser_tests()?;
    let (program, rows, script) = read_case(&fixtures.join("rust-lowered-static-dynamic"))?;
    reference_tests(&program, &rows, &script)
}

/// Pins the TS-29 expression subset against `expression-cases.json`: every
/// supported case evaluates to exactly the recorded value, and every
/// unsupported case fails closed. A tooling test evaluates the same supported
/// cases with Node's JavaScript engine.
pub fn expressions(fixtures: &Path) -> Result<(usize, usize), String> {
    let path = fixtures.join("expression-cases.json");
    let table = json::parse(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)?;
    let context = get(&table, "context")?;
    let (mut supported, mut unsupported) = (0, 0);
    for case in get_arr(get(&table, "cases")?)? {
        let expr = get_str(get(case, "expr")?)?;
        match (get(case, "value"), expression::evaluate(context, expr)) {
            (Ok(expected), Ok(actual)) => {
                if actual != *expected {
                    return Err(format!(
                        "`{expr}` evaluated to {}, expected {}",
                        json::compress(&actual),
                        json::compress(expected)
                    ));
                }
                supported += 1;
            }
            (Ok(_), Err(message)) => {
                return Err(format!("`{expr}` should be supported: {message}"));
            }
            (Err(_), Ok(actual)) => {
                return Err(format!(
                    "`{expr}` must fail closed, got {}",
                    json::compress(&actual)
                ));
            }
            (Err(_), Err(_)) => {
                get_str(get(case, "unsupported")?)?;
                unsupported += 1;
            }
        }
    }
    if supported < 50 || unsupported < 30 {
        return Err("expression table is too small".into());
    }
    Ok((supported, unsupported))
}
