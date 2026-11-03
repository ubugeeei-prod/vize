//! Control-flow and slot witnesses on `rust-lowered-control-slots`.

use std::path::Path;

use super::{expect_error, keep, mutate};
use crate::l3_reference::behavior::{read_case, run};
use crate::l3_reference::json::{self, Json, object, set};
use crate::l3_reference::program::{Op, OpKind, Program, Region};
use crate::l3_reference::values::{Operand, one};

pub(super) fn expect_value<T: PartialEq>(
    label: &str,
    expected: T,
    result: Result<T, String>,
) -> Result<(), String> {
    match result? == expected {
        true => Ok(()),
        false => Err(format!("incorrect control observation: {label}")),
    }
}

pub(super) fn plus(rows: &[Operand], extra: &[Operand]) -> Vec<Operand> {
    let mut out = rows.to_vec();
    out.extend_from_slice(extra);
    out
}

pub(super) fn at(rows: &[Operand], id: u32, role: &'static str, f: impl Fn(&mut Operand)) -> Vec<Operand> {
    mutate(rows, |r| r.op == id && r.role == role, f)
}

pub(super) fn context(text: &str) -> Result<Json, String> {
    json::parse(text)
}

fn scenario(context: Json) -> Json {
    object([("context", context), ("steps", Json::Array(Vec::new()))])
}

pub(super) fn with_regions(program: &Program, f: impl Fn(&mut Vec<Region>)) -> Program {
    let mut out = program.clone();
    f(&mut out.regions);
    out
}

pub(super) fn with_ops(program: &Program, ops: &[Op]) -> Program {
    let mut out = program.clone();
    out.ops.extend_from_slice(ops);
    out
}

fn reference_tests(program: &Program, rows: &[Operand], script: &Json) -> Result<(), String> {
    run(program, rows, script)?;
    let condition = one(rows, 1, "condition")?.clone();
    let slot = one(rows, 4, "name")?.clone();
    let value = one(rows, 6, "value")?.clone();
    let hidden = scenario(context(r#"{"ready":false,"fallback":"fallback"}"#)?);
    run(program, rows, &hidden)?;
    let text = one(rows, 3, "text")?.clone();
    let tag = one(rows, 2, "tag")?.clone();
    let namespace = one(rows, 2, "namespace")?.clone();
    for (label, damaged) in [
        ("inactive branch missing text", keep(rows, |r| r.op != 3)),
        ("inactive branch duplicate text", plus(rows, &[text])),
        (
            "inactive branch targeted text",
            at(rows, 3, "text", |r| r.target = Some(2)),
        ),
        (
            "inactive branch missing tag",
            keep(rows, |r| r.op != 2 || r.role != "tag"),
        ),
        ("inactive branch duplicate tag", plus(rows, &[tag])),
        (
            "inactive branch missing namespace",
            keep(rows, |r| r.op != 2 || r.role != "namespace"),
        ),
        (
            "inactive branch duplicate namespace",
            plus(rows, &[namespace]),
        ),
    ] {
        expect_error(label, run(program, &damaged, &hidden))?;
    }
    let mut title = slot.clone();
    title.role = "attribute".into();
    title.name = Some("title".into());
    for (label, damaged) in [
        ("missing condition", keep(rows, |r| r.role != "condition")),
        ("duplicate condition", plus(rows, std::slice::from_ref(&condition))),
        (
            "foreign branch region",
            at(rows, 1, "condition", |r| r.region = Some(4)),
        ),
        (
            "missing branch region",
            at(rows, 1, "condition", |r| r.region = Some(99)),
        ),
        (
            "loose-equality condition",
            at(rows, 1, "condition", |r| r.text = "ready == other".into()),
        ),
        (
            "literal condition",
            at(rows, 1, "condition", |r| r.kind = "literal".into()),
        ),
        (
            "sole else branch",
            at(rows, 1, "condition", |r| {
                r.kind = "absent".into();
                r.text = String::new();
            }),
        ),
        ("missing slot name", keep(rows, |r| r.op != 4)),
        ("duplicate slot name", plus(rows, std::slice::from_ref(&slot))),
        (
            "dynamic slot",
            at(rows, 4, "name", |r| r.kind = "js".into()),
        ),
        (
            "empty slot name",
            at(rows, 4, "name", |r| r.text = String::new()),
        ),
        ("duplicate slot props", plus(rows, &[title.clone(), title])),
        (
            "missing text value",
            keep(rows, |r| r.op != 6 || r.role != "value"),
        ),
        ("duplicate text value", plus(rows, std::slice::from_ref(&value))),
        (
            "wrong text target",
            at(rows, 6, "value", |r| r.target = Some(2)),
        ),
        (
            "non-element text target",
            at(rows, 6, "value", |r| r.target = Some(4)),
        ),
        (
            "unsupported text binding",
            at(rows, 6, "binding-kind", |r| r.text = "vue.html".into()),
        ),
        (
            "untargeted text binding",
            keep(rows, |r| r.op != 6 || r.role != "binding-kind"),
        ),
    ] {
        expect_error(label, run(program, &damaged, script))?;
    }
    let region = |id, parent, owner, span| Region {
        id,
        parent: Some(parent),
        owner: Some(owner),
        span,
    };
    for (label, damaged) in [
        (
            "wrong branch owner",
            with_regions(program, |rs| {
                rs.iter_mut()
                    .filter(|r| r.id == 2)
                    .for_each(|r| r.owner = Some(3))
            }),
        ),
        (
            "wrong branch parent",
            with_regions(program, |rs| {
                rs.iter_mut()
                    .filter(|r| r.id == 2)
                    .for_each(|r| r.parent = Some(0))
            }),
        ),
        (
            "missing fallback region",
            with_regions(program, |rs| rs.retain(|r| r.id != 4)),
        ),
        (
            "extra branch region",
            with_regions(program, |rs| rs.push(region(6, 1, 1, condition.span))),
        ),
        (
            "extra fallback region",
            with_regions(program, |rs| rs.push(region(6, 1, 4, slot.span))),
        ),
    ] {
        expect_error(label, run(&damaged, rows, script))?;
    }
    // Conditions follow JavaScript truthiness, exactly like both mounted runtimes.
    let ready = |ready: &str| -> Result<Json, String> {
        let state = context(&format!(r#"{{"ready":{ready},"fallback":"x"}}"#))?;
        run(program, rows, &set(script, "context", state)?)
    };
    for (truthy, same) in [
        ("\"false\"", "true"),
        ("null", "false"),
        ("1", "true"),
        ("\"\"", "false"),
    ] {
        if ready(truthy)? != ready(same)? {
            return Err(format!("condition {truthy} must render like {same}"));
        }
    }
    for state in [
        r#"{"ready":true,"fallback":{}}"#,
        r#"{"ready":true,"fallback":[]}"#,
        r#"{"ready":true,"fallback":1.5}"#,
        r#"{"ready":true,"fallback":"x","$slots":{}}"#,
    ] {
        expect_error(
            "unsupported control context",
            run(program, rows, &set(script, "context", context(state)?)?),
        )?;
    }
    let text_op = Op {
        id: 99,
        kind: OpKind::SetText,
        region: 5,
        effect: None,
        span: value.span,
    };
    let mut text_row = value.clone();
    text_row.op = 99;
    text_row.role = "text".into();
    text_row.target = None;
    expect_error(
        "v-text with children",
        run(
            &with_ops(program, &[text_op]),
            &plus(rows, &[text_row]),
            script,
        ),
    )?;
    let Some(original) = program.op(6) else {
        return Err("missing text binding fixture".into());
    };
    let duplicate = Op {
        id: 99,
        ..*original
    };
    let duplicate_rows = mutate(&keep(rows, |r| r.op == 6), |_| true, |r| r.op = 99);
    expect_error(
        "duplicate v-text binding",
        run(
            &with_ops(program, &[duplicate]),
            &plus(rows, &duplicate_rows),
            script,
        ),
    )
}

fn inactive_value_tests(program: &Program, rows: &[Operand]) -> Result<(), String> {
    let script = scenario(context(r#"{"ready":false,"fallback":"fallback"}"#)?);
    let tag = one(rows, 2, "tag")?.clone();
    let attr = Operand {
        role: "attribute".into(),
        name: Some("title".into()),
        text: "hidden".into(),
        ..tag.clone()
    };
    run(program, &plus(rows, std::slice::from_ref(&attr)), &script)?;
    let variant = |f: &dyn Fn(&mut Operand)| {
        let mut row = attr.clone();
        f(&mut row);
        row
    };
    for (label, damaged) in [
        (
            "inactive dynamic tag",
            at(rows, 2, "tag", |r| r.kind = "js".into()),
        ),
        (
            "inactive unsupported tag",
            at(rows, 2, "tag", |r| r.text = "script".into()),
        ),
        (
            "inactive dynamic namespace",
            at(rows, 2, "namespace", |r| r.kind = "js".into()),
        ),
        (
            "inactive unsupported namespace",
            at(rows, 2, "namespace", |r| r.text = "svg".into()),
        ),
        (
            "inactive dynamic attribute",
            plus(rows, &[variant(&|r| r.kind = "js".into())]),
        ),
        (
            "inactive unnamed attribute",
            plus(rows, &[variant(&|r| r.name = None)]),
        ),
        (
            "inactive unsupported attribute",
            plus(rows, &[variant(&|r| r.name = Some("onclick".into()))]),
        ),
        (
            "inactive duplicate attribute",
            plus(rows, &[attr.clone(), attr.clone()]),
        ),
        (
            "inactive unsupported text",
            at(rows, 3, "text", |r| r.kind = "opaque".into()),
        ),
        (
            "inactive compound text",
            at(rows, 3, "text", |r| {
                r.kind = "js".into();
                r.text = "missing()".into();
            }),
        ),
    ] {
        expect_error(label, run(program, &damaged, &script))?;
    }
    run(
        program,
        &at(rows, 3, "text", |r| {
            r.kind = "js".into();
            r.text = "missing".into();
        }),
        &script,
    )?;
    let Some(element) = program.op(2) else {
        return Err("missing branch element".into());
    };
    let binding = Op {
        id: 99,
        kind: OpKind::SetProp,
        ..*element
    };
    let name = Operand {
        op: 99,
        role: "name".into(),
        target: Some(2),
        text: "disabled".into(),
        ..tag.clone()
    };
    let value = Operand {
        role: "value".into(),
        kind: "js".into(),
        text: "missing".into(),
        ..name.clone()
    };
    let kind = Operand {
        role: "binding-kind".into(),
        text: "bind".into(),
        ..name.clone()
    };
    let button_rows = at(rows, 2, "tag", |r| r.text = "button".into());
    let texted = |row: &Operand, text: &str| Operand {
        text: text.into(),
        ..row.clone()
    };
    run(
        &with_ops(program, &[binding]),
        &plus(&button_rows, &[name.clone(), value.clone(), kind.clone()]),
        &script,
    )?;
    expect_error(
        "inactive property on non-button",
        run(
            &with_ops(program, &[binding]),
            &plus(rows, &[name.clone(), value.clone(), kind.clone()]),
            &script,
        ),
    )?;
    let event = Op {
        kind: OpKind::SetEvent,
        ..binding
    };
    let event_rows = [
        texted(&name, "click"),
        texted(&value, "save"),
        texted(&kind, "on"),
    ];
    let renumber = |rows: &[Operand]| mutate(rows, |_| true, |r| r.op = 100);
    let cases: [(&str, Vec<Op>, Vec<Operand>); 8] = [
        (
            "inactive unsupported property",
            vec![binding],
            vec![texted(&name, "style"), value.clone(), kind.clone()],
        ),
        (
            "inactive unsupported prop expression",
            vec![binding],
            vec![name.clone(), texted(&value, "missing()"), kind.clone()],
        ),
        (
            "inactive literal property",
            vec![binding],
            vec![
                name.clone(),
                Operand {
                    kind: "literal".into(),
                    ..value.clone()
                },
                kind.clone(),
            ],
        ),
        (
            "inactive duplicate properties",
            vec![binding, Op { id: 100, ..binding }],
            plus(
                &[name.clone(), value.clone(), kind.clone()],
                &renumber(&[name.clone(), value.clone(), kind.clone()]),
            ),
        ),
        (
            "inactive duplicate events",
            vec![event, Op { id: 100, ..event }],
            plus(&event_rows, &renumber(&event_rows)),
        ),
        (
            "inactive unsupported event",
            vec![event],
            vec![
                texted(&name, "focus"),
                texted(&value, "save"),
                texted(&kind, "on"),
            ],
        ),
        (
            "inactive unsupported handler",
            vec![event],
            vec![texted(&name, "click"), value.clone(), texted(&kind, "on")],
        ),
        (
            "inactive v-text with children",
            vec![Op {
                kind: OpKind::SetText,
                ..binding
            }],
            vec![value.clone(), texted(&kind, "vue.text")],
        ),
    ];
    for (label, ops, operands) in cases {
        expect_error(
            label,
            run(
                &with_ops(program, &ops),
                &plus(&button_rows, &operands),
                &script,
            ),
        )?;
    }
    let text_binding = Op {
        kind: OpKind::SetText,
        ..binding
    };
    let text_rows = [value.clone(), texted(&kind, "vue.text")];
    let mut without_text = program.clone();
    without_text.ops.retain(|op| op.id != 3);
    let without_text_rows = keep(rows, |r| r.op != 3);
    run(
        &with_ops(&without_text, &[text_binding]),
        &plus(&without_text_rows, &text_rows),
        &script,
    )?;
    expect_error(
        "inactive duplicate v-text",
        run(
            &with_ops(
                &without_text,
                &[
                    text_binding,
                    Op {
                        id: 100,
                        ..text_binding
                    },
                ],
            ),
            &plus(&plus(&without_text_rows, &text_rows), &renumber(&text_rows)),
            &script,
        ),
    )
}

pub fn check(fixtures: &Path) -> Result<(), String> {
    let (program, rows, script) = read_case(&fixtures.join("rust-lowered-control-slots"))?;
    reference_tests(&program, &rows, &script)?;
    inactive_value_tests(&program, &rows)?;
    super::branches::branch_tests(&program, &rows)?;
    super::branches::activation_tests(&program, &rows)
}
