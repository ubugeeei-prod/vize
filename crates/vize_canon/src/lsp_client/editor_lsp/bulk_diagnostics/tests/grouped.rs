use super::super::{conversion, positions::Positions, selected_semantics};
use serde_json::{Value, json};
use vize_l0::{FxHashMap, FxHashSet, String};

#[test]
fn whole_categories_preserve_requested_order_related_rows_duplicates_and_empty_sources() {
    let members = FxHashSet::from_iter(["/a.ts", "/empty.ts", "/unrequested.ts", "/dependency.ts"]);
    let requested = FxHashSet::from_iter(["/a.ts", "/empty.ts"]);
    let category = selected_semantics::decode_project(
        json!([
            {"fileName":"/unrequested.ts","pos":4000,"end":4001,"code":2322,
             "category":1,"text":"Unrequested","relatedInformation":[
                {"fileName":"/unrequested-related.ts","pos":0,"end":1,"code":0,
                 "category":1,"text":"Unrequested related"}]},
            {"fileName":"/a.ts","pos":2,"end":3,"code":6133,"category":1,
             "text":"Style","messageChain":[{"pos":0,"end":0,"code":0,
                "category":1,"text":"Complete chain"}],"relatedInformation":[
                {"fileName":"/dependency.ts","pos":2,"end":3,"code":0,
                 "category":1,"text":"Owned related"}]},
            {"fileName":"/a.ts","pos":2,"end":3,"code":2322,"category":1,
             "text":"Second retained row"}
        ]),
        &members,
        &requested,
    )
    .unwrap();
    let documents = FxHashMap::from_iter([
        (String::from("file:///a.ts"), String::from("😀x")),
        (String::from("file:///empty.ts"), String::from("export {};")),
    ]);
    let mut related_reads = Vec::new();
    let actual = conversion::project_diagnostics_with_source(
        &[category],
        &[
            "file:///a.ts".into(),
            "file:///empty.ts".into(),
            "file:///a.ts".into(),
        ],
        &documents,
        |uri| {
            related_reads.push(String::from(uri));
            Ok::<_, &'static str>(Positions::new("😀d"))
        },
    )
    .unwrap()
    .unwrap();
    let expected = json!([
        {"range":{"start":{"line":0,"character":2},"end":{"line":0,"character":3}},
         "severity":2,"code":6133,"source":"ts","message":"Style\n  Complete chain",
         "relatedInformation":[{"location":{"uri":"file:///dependency.ts",
            "range":{"start":{"line":0,"character":2},"end":{"line":0,"character":3}}},
            "message":"Owned related"}]},
        {"range":{"start":{"line":0,"character":2},"end":{"line":0,"character":3}},
         "severity":1,"code":2322,"source":"ts","message":"Second retained row"}
    ]);
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        json!([expected, [], expected])
    );
    assert_eq!(related_reads, ["file:///dependency.ts"]);
    for empty in [Value::Null, json!([])] {
        assert!(
            selected_semantics::decode_project(empty, &members, &requested)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn whole_categories_refuse_foreign_alias_fileless_and_malformed_unrequested_rows_atomically() {
    let members = FxHashSet::from_iter(["/project/a.ts", "/project/unrequested.ts"]);
    let requested = FxHashSet::from_iter(["/project/a.ts"]);
    let original = json!({"fileName":"/project/a.ts","pos":0,"end":1,
        "code":2322,"category":1,"text":"Complete original"});
    for name in [
        "/foreign.ts",
        "/project/./a.ts",
        "/project//a.ts",
        "/project/%61.ts",
        "",
    ] {
        let foreign = json!({"fileName":name,"pos":0,"end":1,
            "code":2322,"category":1,"text":"Foreign"});
        assert!(
            selected_semantics::decode_project(
                json!([original.clone(), foreign]),
                &members,
                &requested,
            )
            .is_none()
        );
    }
    for malformed in [
        json!({"fileName":"/project/unrequested.ts","pos":"0","end":1,
            "code":2322,"category":1,"text":"Wrong position type"}),
        json!({"fileName":"/project/unrequested.ts","pos":0,"end":1,
            "code":2322,"category":1,"text":"Wrong nested type","messageChain":[false]}),
        json!({"fileName":"/project/unrequested.ts","pos":0,"end":1,
            "code":2322,"category":1,"text":"Wrong related type","relatedInformation":[false]}),
        json!({"pos":0,"end":1,"code":2322,"category":1,"text":"Absent main identity"}),
    ] {
        assert!(
            selected_semantics::decode_project(
                json!([original.clone(), malformed]),
                &members,
                &requested,
            )
            .is_none()
        );
    }
    assert!(
        selected_semantics::decode_project(json!({"invented":"envelope"}), &members, &requested)
            .is_none()
    );
}
