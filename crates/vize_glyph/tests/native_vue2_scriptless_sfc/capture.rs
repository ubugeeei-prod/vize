//! Every actual attempt is retained before any independent golden judgment.

use super::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::panic::{AssertUnwindSafe, catch_unwind};
use vize_glyph::FormatResult;
use vize_glyph::native_doc::NativeVue2SfcRefusal;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: std::string::String,
    source: std::string::String,
    expected: std::string::String,
    width: usize,
    indent: usize,
    line_ending: std::string::String,
}
#[derive(Deserialize)]
struct Packet {
    cases: std::vec::Vec<Case>,
}
fn policy(case: &Case) -> NativeVue2SfcOptions {
    let ending = match case.line_ending.as_str() {
        "Lf" => LineEnding::Lf,
        "CrLf" => LineEnding::CrLf,
        _ => panic!("complete explicit ending policy"),
    };
    options(case.width, case.indent, ending)
}
fn formatted(result: &Result<FormatResult, NativeVue2SfcRefusal>) -> Value {
    match result {
        Ok(result) => json!({
            "status": "printed", "printed": result.code.as_str(),
            "changed": result.changed, "refusal": null,
        }),
        Err(refusal) => json!({
            "status": "refused", "printed": null, "changed": null,
            "refusal": format!("{refusal:?}"),
        }),
    }
}
fn attempt(case: &Case, row: &mut Value) {
    let opts = policy(case);
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, &case.source, opts);
    let first = owner.format();
    row["format"] = formatted(&first);
    row["repeat"] = formatted(&owner.format());
    row["byteLength"] = json!(owner.source().len());
    match owner.selected() {
        Ok(selected) => {
            let span = selected.block().span();
            row["beforeUtf8"] = json!({"start": span.start, "end": span.end});
            row["containerIndex"] = json!(selected.container_index());
            row["openingName"] = json!({
                "start": selected.opening_name().start, "end": selected.opening_name().end,
            });
            row["closingName"] = json!({
                "start": selected.closing_name().start, "end": selected.closing_name().end,
            });
        }
        Err(refusal) => row["selectionRefusal"] = json!(format!("{refusal:?}")),
    }
    if let Ok(result) = first {
        let second_arena = Allocator::default();
        let second = observe_native_vue2_sfc_in(&second_arena, &result.code, opts);
        row["fixedPoint"] = formatted(&second.format());
        match second.selected() {
            Ok(selected) => {
                let span = selected.block().span();
                row["afterUtf8"] = json!({"start": span.start, "end": span.end});
            }
            Err(refusal) => row["afterSelectionRefusal"] = json!(format!("{refusal:?}")),
        }
    }
    row["status"] = json!("completed");
}

#[test]
fn whole_original_sfc_attempts_are_captured_before_complete_goldens_and_fixed_points() {
    let packet: Packet = serde_json::from_str(include_str!(
        "../fixtures/native-vue2-scriptless-sfc-2.7.16.json"
    ))
    .unwrap();
    assert_eq!(packet.cases.len(), 32);
    let mut captures = std::vec::Vec::new();
    for case in &packet.cases {
        let mut row = json!({
            "id": case.id, "source": case.source, "width": case.width,
            "indent": case.indent, "lineEnding": case.line_ending,
            "status": "attempting", "panic": null, "format": null,
            "repeat": null, "fixedPoint": null, "beforeUtf8": null,
            "afterUtf8": null, "byteLength": null, "containerIndex": null,
            "openingName": null, "closingName": null,
            "selectionRefusal": null, "afterSelectionRefusal": null,
        });
        if let Err(payload) = catch_unwind(AssertUnwindSafe(|| attempt(case, &mut row))) {
            let message = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| {
                    payload
                        .downcast_ref::<std::string::String>()
                        .map(|s| s.as_str())
                })
                .unwrap_or("non-string panic payload");
            row["status"] = json!("panicked");
            row["panic"] = json!(message);
        }
        captures.push(row);
    }
    if let Some(path) = std::env::var_os("VIZE_GLYPH_VUE2_SFC_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&json!({
                "schema": "vize.native-vue2-scriptless-sfc-capture", "version": 2,
                "sourceHead": std::env::var("VIZE_GLYPH_VUE2_SFC_SOURCE_HEAD").ok(),
                "executionCommit": std::env::var("GITHUB_SHA").ok(),
                "workflowRun": std::env::var("GITHUB_RUN_ID").ok(), "cases": captures,
            }))
            .unwrap(),
        )
        .unwrap();
    }
    // All actual attempts and refusals already exist before any expected output check.
    for (case, actual) in packet.cases.iter().zip(&captures) {
        assert_eq!(actual["status"], "completed", "{}: {actual}", case.id);
        assert_eq!(
            actual["format"]["status"], "printed",
            "{}: {actual}",
            case.id
        );
        assert_eq!(actual["format"]["printed"], case.expected, "{}", case.id);
        fixed(&case.source, &case.expected, policy(case));
    }
}
