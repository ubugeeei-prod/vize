//! Branch selection and activation witnesses on `rust-lowered-control-slots`.

use super::control::{at, context, expect_value, plus, with_ops, with_regions};
use super::{expect_error, keep};
use crate::l3_reference::behavior::{button_state, run};
use crate::l3_reference::json::{self, Json, get, get_arr, object, set};
use crate::l3_reference::observation::selected_branch;
use crate::l3_reference::program::{Op, OpKind, Program, Region, Span};
use crate::l3_reference::validate::validate;
use crate::l3_reference::values::{Operand, one};

pub(super) fn branch_tests(program: &Program, rows: &[Operand]) -> Result<(), String> {
    let first = one(rows, 1, "condition")?.clone();
    let second = Operand {
        region: Some(6),
        text: "other".into(),
        ..first.clone()
    };
    let fallback = Operand {
        region: Some(7),
        kind: "absent".into(),
        text: String::new(),
        ..first.clone()
    };
    let branches = vec![first.clone(), second.clone(), fallback.clone()];
    let span = first.span;
    let program = with_regions(program, |rs| {
        rs.push(Region {
            id: 6,
            parent: Some(1),
            owner: Some(1),
            span,
        });
        rs.push(Region {
            id: 7,
            parent: Some(1),
            owner: Some(1),
            span,
        });
    });
    let others = keep(rows, |r| r.op != 1);
    let rows = plus(&others, &branches);
    validate(&program, &rows)?;
    for (state, expected) in [
        (r#"{"ready":true}"#, Some(2)),
        (r#"{"ready":true,"other":true}"#, Some(2)),
        (r#"{"ready":false,"other":true}"#, Some(6)),
        (r#"{"ready":false,"other":false}"#, Some(7)),
    ] {
        expect_value(
            "first matching branch",
            expected,
            selected_branch(&rows, &context(state)?, 1),
        )?;
    }
    let ready = context(r#"{"ready":false}"#)?;
    expect_value(
        "no selected branch",
        None,
        selected_branch(std::slice::from_ref(&first), &ready, 1),
    )?;
    for conditions in [
        [fallback.clone(), second.clone(), first.clone()],
        [first.clone(), fallback.clone(), second.clone()],
        [first.clone(), second.clone(), second.clone()],
    ] {
        expect_error(
            "misordered or duplicate branches",
            validate(&program, &plus(&others, &conditions)),
        )?;
    }
    let blank = |row: &Operand| Operand {
        kind: "absent".into(),
        text: String::new(),
        ..row.clone()
    };
    for conditions in [
        [blank(&first), second.clone(), fallback.clone()],
        [first.clone(), blank(&second), fallback.clone()],
    ] {
        expect_error(
            "else before final region",
            validate(&program, &plus(&others, &conditions)),
        )?;
    }
    for (state, expected) in [
        (r#"{"ready":0,"other":"false"}"#, Some(6)),
        (r#"{"ready":"","other":[]}"#, Some(6)),
        (r#"{"ready":null,"other":0}"#, Some(7)),
        (r#"{"ready":{},"other":0}"#, Some(2)),
    ] {
        expect_value(
            "JavaScript truthiness",
            expected,
            selected_branch(&branches, &context(state)?, 1),
        )?;
    }
    expect_error(
        "unsupported reached condition",
        selected_branch(&branches, &ready, 1),
    )
}

pub(super) fn activation_tests(program: &Program, rows: &[Operand]) -> Result<(), String> {
    let visible = context(r#"{"ready":true,"fallback":"fallback"}"#)?;
    let hidden = set(&visible, "ready", Json::Bool(false))?;
    let rows = at(rows, 2, "tag", |r| r.text = "button".into());
    let span = Span { start: 9, stop: 34 };
    let handler = Op {
        id: 99,
        kind: OpKind::SetEvent,
        region: 2,
        effect: None,
        span,
    };
    let name = Operand {
        op: 99,
        role: "name".into(),
        target: Some(2),
        region: None,
        name: None,
        kind: "literal".into(),
        text: "click".into(),
        qualifier: String::new(),
        span,
    };
    let program = with_ops(program, &[handler]);
    let rows = plus(
        &rows,
        &[
            name.clone(),
            Operand {
                role: "value".into(),
                kind: "js".into(),
                text: "save".into(),
                ..name.clone()
            },
            Operand {
                role: "binding-kind".into(),
                text: "on".into(),
                ..name
            },
        ],
    );
    expect_value(
        "visible branch button",
        (false, true),
        button_state(&program, &rows, &visible),
    )?;
    expect_error(
        "hidden branch button",
        button_state(&program, &rows, &hidden),
    )?;
    let activate = json::parse(r#"[{"activate":"button"}]"#)?;
    let script = object([("context", visible.clone()), ("steps", activate)]);
    let trace = run(&program, &rows, &script)?;
    let snapshot = get_arr(&trace)?.get(1).ok_or("missing snapshot")?;
    expect_value(
        "visible branch delivery",
        &Json::Array(vec!["save".into()]),
        get(snapshot, "events"),
    )?;
    let two_buttons = at(&rows, 5, "tag", |r| r.text = "button".into());
    expect_error(
        "ambiguous active buttons",
        button_state(&program, &two_buttons, &visible),
    )?;
    expect_value(
        "only fallback button is active",
        (false, false),
        button_state(&program, &two_buttons, &hidden),
    )
}
