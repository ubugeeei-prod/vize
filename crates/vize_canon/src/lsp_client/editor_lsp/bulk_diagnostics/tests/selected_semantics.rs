use super::super::selected_semantics;
use serde_json::json;
use vize_l0::{FxHashMap, String};

#[test]
fn selected_semantics_queries_only_exact_distinct_original_members_in_request_order() {
    let sources = FxHashMap::from_iter([
        ("file:///z.ts", "/z.ts"),
        ("file:///a%20%23.ts", "/a #.ts"),
        ("file:///dependency.ts", "/dependency.ts"),
    ]);
    let uris = ["file:///z.ts", "file:///a%20%23.ts", "file:///z.ts"].map(String::from);
    assert_eq!(
        selected_semantics::names(&uris, |uri| sources.get(uri).copied()),
        Some(vec!["/z.ts", "/a #.ts"])
    );
    for alias in [
        "file:///./z.ts",
        "file:////z.ts",
        "file:///%7a.ts",
        "file:///foreign.ts",
    ] {
        let requested = [String::from("file:///z.ts"), String::from(alias)];
        assert_eq!(
            selected_semantics::names(&requested, |uri| sources.get(uri).copied()),
            None
        );
    }
}

#[test]
fn selected_semantic_responses_refuse_foreign_main_rows_without_dropping_related_info() {
    let row = json!({"fileName":"/a.ts","pos":0,"end":1,"code":2322,"category":1,
        "text":"Original","relatedInformation":[{"fileName":"/dependency.ts","pos":0,"end":1,
        "code":2322,"category":1,"text":"Original related"}]});
    assert_eq!(
        selected_semantics::decode(serde_json::Value::Null, Some("/a.ts"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        selected_semantics::decode(json!([]), Some("/a.ts"))
            .unwrap()
            .len(),
        0
    );
    let selected = selected_semantics::decode(json!([row.clone()]), Some("/a.ts")).unwrap();
    assert_eq!(selected.len(), 1);
    assert!(selected[0].belongs_to("/a.ts"));
    assert!(selected_semantics::decode(json!([row.clone(), row]), Some("/foreign.ts")).is_none());
    assert!(selected_semantics::decode(json!({"invented":"envelope"}), Some("/a.ts")).is_none());
}
