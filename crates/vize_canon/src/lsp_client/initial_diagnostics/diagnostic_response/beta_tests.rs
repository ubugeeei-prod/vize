use super::{NativeDiagnostic, SnapshotText};
use serde_json::{Value, json};

fn wire() -> Value {
    json!({"fileName":"/workspace/a.ts","pos":8,"end":14,"code":2322,"category":1,"text":"whole error",
        "startPosition":{"line":3,"character":0},"endPosition":{"line":3,"character":6},
        "sourceLines":[{"line":3,"text":"target\r\n"}]})
}
fn convert(row: Value) -> Result<lsp_types::Diagnostic, vize_l0::String> {
    let source = SnapshotText::new("😀\r\nx\u{2028}é\u{2029}target\r\n")?;
    serde_json::from_value::<NativeDiagnostic>(row)
        .map_err(|error| vize_l0::cstr!("{error}"))?
        .into_lsp("/workspace/a.ts", |_| Ok(&source))
}
#[test]
fn beta_ecma_metadata_is_validated_but_lsp_ranges_keep_crlf_only_and_custom_source() {
    let mut row = wire();
    row["source"] = json!("example-mapper");
    assert_eq!(
        serde_json::to_value(convert(row).unwrap()).unwrap(),
        json!({
        "range":{"start":{"line":1,"character":4},"end":{"line":1,"character":10}},
        "severity":1,"code":2322,"source":"example-mapper","message":"whole error"})
    );
    let mut row = wire();
    row["source"] = json!("");
    assert_eq!(convert(row).unwrap().source.as_deref(), Some("ts"));
}
#[test]
fn metadata_null_unknown_fields_partial_ranges_and_wrong_source_never_become_clean() {
    for field in ["startPosition", "endPosition", "sourceLines", "source"] {
        let mut row = wire();
        row[field] = Value::Null;
        assert!(convert(row).is_err());
    }
    for field in ["startPosition", "endPosition", "sourceLines"] {
        let mut row = wire();
        row.as_object_mut().unwrap().remove(field);
        assert!(convert(row).is_err());
    }
    for (field, value) in [
        ("pos", json!(-1)),
        ("pos", json!(u32::MAX)),
        ("end", json!(7)),
        ("end", json!(17)),
        ("fileName", json!("/foreign/a.ts")),
        ("startPosition", json!({"line":1,"character":4})),
        ("endPosition", json!({"line":3,"character":5})),
        ("startPosition", json!({"line":-1,"character":0})),
        ("startPosition", json!({"line":3,"character":u64::MAX})),
        ("sourceLines", json!([])),
        ("sourceLines", json!([{"line":3,"text":"target\n"}])),
        ("sourceLines", json!([{"line":2,"text":"target\r\n"}])),
    ] {
        let mut row = wire();
        row[field] = value;
        assert!(convert(row).is_err(), "field {field}");
    }
    for target in [
        "root",
        "startPosition",
        "sourceLine",
        "messageChain",
        "relatedInformation",
    ] {
        let mut row = wire();
        match target {
            "root" => row["futureDiagnosticField"] = json!(true),
            "startPosition" => row["startPosition"]["futurePositionField"] = json!(0),
            "sourceLine" => row["sourceLines"][0]["futureLineField"] = json!(0),
            field => {
                row[field] = json!([{"pos":0,"end":0,"code":1,"category":1,"text":"child","futureChildField":true}])
            }
        }
        assert!(
            serde_json::from_value::<NativeDiagnostic>(row).is_err(),
            "{target}"
        );
    }
}
#[test]
fn beta_long_context_is_exact_first_two_last_two_with_all_original_line_endings() {
    let source = SnapshotText::new("a\r\nb\rc\nd\u{2028}e\u{2029}f").unwrap();
    let row = json!({"fileName":"/workspace/a.ts","pos":0,"end":12,"code":1,"category":1,"text":"long",
        "startPosition":{"line":0,"character":0},"endPosition":{"line":5,"character":1},
        "sourceLines":[{"line":0,"text":"a\r\n"},{"line":1,"text":"b\r"},{"line":4,"text":"e\u{2029}"},{"line":5,"text":"f"}]});
    let convert = |row: Value| {
        serde_json::from_value::<NativeDiagnostic>(row)
            .unwrap()
            .into_lsp("/workspace/a.ts", |_| Ok(&source))
    };
    assert!(convert(row.clone()).is_ok());
    for field in [0, 1, 2, 3] {
        let mut bad = row.clone();
        bad["sourceLines"][field]["text"] = json!("neighbor");
        assert!(convert(bad).is_err());
    }
    let mut bad = row.clone();
    bad["sourceLines"].as_array_mut().unwrap().swap(0, 1);
    assert!(convert(bad).is_err());
    let mut bad = row.clone();
    bad["sourceLines"].as_array_mut().unwrap().remove(2);
    assert!(convert(bad).is_err());
}
#[test]
fn chained_metadata_is_checked_on_its_own_source_and_cannot_hide_a_bad_location() {
    let mut row = wire();
    row["messageChain"] = json!([wire()]);
    assert_eq!(
        convert(row.clone()).unwrap().message,
        "whole error\n  whole error"
    );
    row["messageChain"][0]["sourceLines"][0]["text"] = json!("wrong");
    assert!(convert(row).is_err());
}
