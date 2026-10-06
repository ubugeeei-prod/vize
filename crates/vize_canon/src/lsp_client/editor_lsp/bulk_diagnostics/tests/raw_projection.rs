use super::super::{conversion::NativeDiagnostic, selected_semantics};
use serde_json::{Value, json};
use vize_l0::{FxHashSet, cstr};

fn original(
    value: Value,
    members: &FxHashSet<&str>,
    requested: &FxHashSet<&str>,
) -> Option<Vec<NativeDiagnostic>> {
    let diagnostics = selected_semantics::decode(value, None)?;
    if !diagnostics
        .iter()
        .all(|row| !row.file_name().is_empty() && members.contains(row.file_name()))
    {
        return None;
    }
    Some(
        diagnostics
            .into_iter()
            .filter(|row| requested.contains(row.file_name()))
            .collect(),
    )
}

fn compare(value: Value, members: &FxHashSet<&str>, requested: &FxHashSet<&str>) {
    let expected = original(value.clone(), members, requested);
    let actual = selected_semantics::decode_project(value.clone(), members, requested);
    assert_eq!(cstr!("{actual:?}"), cstr!("{expected:?}"), "{value}");
}

fn row(name: &str) -> Value {
    json!({"fileName":name,"pos":0,"end":1,"code":2322,"category":1,
        "text":"Whole diagnostic 😀 with text beyond the compact inline capacity"})
}

#[test]
fn omitted_raw_schema_matches_the_original_owned_decoder_at_every_field_boundary() {
    let members = FxHashSet::from_iter(["/a.ts", "/unrequested.ts"]);
    let requested = FxHashSet::from_iter(["/a.ts"]);
    for empty in [Value::Null, json!([]), json!({}), json!(false), json!(0)] {
        compare(empty, &members, &requested);
    }
    let values = [
        Value::Null,
        json!(false),
        json!(0),
        json!(-1),
        json!(255),
        json!(256),
        json!(u32::MAX),
        json!(u64::from(u32::MAX) + 1),
        json!(i32::MIN),
        json!(i64::from(i32::MIN) - 1),
        json!(i32::MAX),
        json!(i64::from(i32::MAX) + 1),
        json!(1.0),
        json!(""),
        json!("/a.ts"),
        json!("/unrequested.ts"),
        json!([]),
        json!({}),
        json!([false]),
    ];
    for field in [
        "fileName",
        "pos",
        "end",
        "code",
        "category",
        "text",
        "messageChain",
        "relatedInformation",
    ] {
        let mut absent = row("/unrequested.ts");
        absent.as_object_mut().unwrap().remove(field);
        compare(json!([row("/a.ts"), absent.clone()]), &members, &requested);
        if field != "fileName" {
            absent["fileName"] = json!("/a.ts");
            compare(json!([row("/a.ts"), absent]), &members, &requested);
        }
        for value in &values {
            let mut changed = row("/unrequested.ts");
            changed[field] = value.clone();
            compare(json!([row("/a.ts"), changed.clone()]), &members, &requested);
            if field != "fileName" {
                changed["fileName"] = json!("/a.ts");
                compare(json!([row("/a.ts"), changed]), &members, &requested);
            }
        }
    }
    let sequence = json!(["/unrequested.ts", 0, 1, 2322, 1, "Sequence", [], []]);
    for length in 0..=8 {
        let mut sequence = Value::Array(sequence.as_array().unwrap()[..length].to_vec());
        compare(
            json!([row("/a.ts"), sequence.clone()]),
            &members,
            &requested,
        );
        if length > 0 {
            sequence[0] = json!("/a.ts");
            compare(json!([row("/a.ts"), sequence]), &members, &requested);
        }
    }
    compare(
        json!([
            row("/a.ts"),
            ["/unrequested.ts", 0, 1, 2322, 1, "Extra", [], [], 0]
        ]),
        &members,
        &requested,
    );
}

#[test]
fn omitted_recursive_schema_and_late_ownership_refusals_preserve_the_whole_projection() {
    let members = FxHashSet::from_iter(["/a.ts", "/unrequested.ts"]);
    let requested = FxHashSet::from_iter(["/a.ts"]);
    let mut nested = row("/unrequested.ts");
    nested["messageChain"] = json!([{"pos":0,"end":0,"code":0,"category":255,
        "text":"Child","messageChain":[["",0,0,0,1,"Grandchild"]]}]);
    nested["relatedInformation"] = json!([{"fileName":"/not-read.ts","pos":999,
        "end":0,"code":i32::MIN,"category":0,"text":"Raw related"}]);
    nested["unknown"] = json!({"ignored":[null,false,{"recursive":true}]});
    compare(
        json!([nested.clone(), row("/a.ts"), row("/a.ts"), nested.clone()]),
        &members,
        &requested,
    );
    for field in ["messageChain", "relatedInformation"] {
        for child in [
            Value::Null,
            json!(false),
            json!([]),
            json!({"text":"Incomplete"}),
            json!({"pos":0,"end":0,"code":0,"category":1,"text":"Malformed nested",
                "relatedInformation":[{"pos":0,"end":0,"code":0,"category":1,"text":null}]}),
        ] {
            let mut malformed = nested.clone();
            malformed[field] = json!([child]);
            compare(json!([row("/a.ts"), malformed]), &members, &requested);
        }
    }
    for name in ["/foreign.ts", "/./a.ts", "//a.ts", "/%61.ts", ""] {
        compare(
            json!([row("/a.ts"), nested.clone(), row(name)]),
            &members,
            &requested,
        );
    }
    let foreign_requested = FxHashSet::from_iter(["/a.ts", "/foreign.ts"]);
    compare(
        json!([row("/a.ts"), row("/foreign.ts")]),
        &members,
        &foreign_requested,
    );
}
