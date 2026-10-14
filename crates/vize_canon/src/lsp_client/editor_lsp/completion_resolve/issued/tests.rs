use super::*;
use serde_json::json;

const URI: &str = "file:///project/Comp.vue.ts";

fn item() -> Value {
    json!({"label":"count","kind":6,"sortText":"11",
        "data":{"sessionId":1,"fileName":"/project/Comp.vue.ts","position":15361,"name":"count"}})
}

#[test]
fn actual_cross_root_and_recovered_items_keep_whole_fields_but_refuse_foreign_positions() {
    for source in [
        include_str!("../../../../../tests/fixtures/issued_completions/cross-root.json"),
        include_str!(
            "../../../../../tests/fixtures/issued_completions/native-epoch-recovered.json"
        ),
    ] {
        let fixture: Value = serde_json::from_str(source).unwrap();
        assert_eq!(
            fixture["source"],
            "bd77ac6637a7f59d80a9a9c7ecdd6076ff783608"
        );
        assert_eq!(fixture["run"], 38130825597_u64);
        let issued = &fixture["wholeIssuedParams"]["data"]["vizeCompletion"]["item"];
        let foreign = &fixture["wholeForeignParams"]["data"]["vizeCompletion"]["item"];
        let uri = fixture["wholeIssuedParams"]["data"]["vizeCompletion"]["requestUri"]
            .as_str()
            .unwrap();
        let mut authority = IssuedCompletions::default();
        let response = json!({"items":[issued]});
        let original = response.clone();
        authority.record(3, uri, Some(&response));
        assert_eq!(response, original);
        assert!(authority.allows(3, uri, issued));
        assert!(!authority.allows(3, uri, foreign));
        assert_ne!(
            fixture["wholeForeignParams"],
            fixture["wholeHistoricalResolved"]
        );
        assert_eq!(fixture["wholeValidResolved"]["label"], "count");
        assert_eq!(
            fixture["wholeValidResolved"]["detail"],
            "(property) count: Ref<number, number>"
        );
    }
}

#[test]
fn every_opaque_and_native_field_must_match_the_issued_item() {
    let original = item();
    let mut authority = IssuedCompletions::default();
    authority.record(4, URI, Some(&json!([original])));
    for (key, replacement) in [
        ("sessionId", json!(2)),
        ("fileName", json!("/foreign.vue.ts")),
        ("position", json!(15362)),
        ("name", json!("foreign")),
    ] {
        let mut foreign = original.clone();
        foreign["data"][key] = replacement;
        assert!(!authority.allows(4, URI, &foreign));
    }
    let mut foreign = original.clone();
    foreign["label"] = json!("other");
    assert!(!authority.allows(4, URI, &foreign));
    foreign = original.clone();
    foreign["documentation"] = json!("unissued document");
    assert!(!authority.allows(4, URI, &foreign));
    assert!(!authority.allows(4, "file:///foreign.vue.ts", &original));
}

#[test]
fn changed_or_closed_documents_and_new_native_sessions_cannot_reuse_evidence() {
    let item = item();
    let mut authority = IssuedCompletions::default();
    authority.record(4, URI, Some(&json!([item])));
    assert!(!authority.allows(5, URI, &item));
    authority.invalidate();
    assert!(!authority.allows(4, URI, &item));
    assert!(authority.documents.is_empty());
    let recovered = IssuedCompletions::default();
    assert!(!recovered.allows(4, URI, &item));
}

#[test]
fn latest_document_response_replaces_evidence_and_preserves_other_current_documents() {
    let first = item();
    let mut second = first.clone();
    second["data"]["position"] = json!(15363);
    let mut authority = IssuedCompletions::default();
    authority.record(4, URI, Some(&json!([first])));
    authority.record(4, "file:///sibling.vue.ts", Some(&json!([second])));
    authority.record(4, URI, Some(&json!([second])));
    assert!(!authority.allows(4, URI, &first));
    assert!(authority.allows(4, URI, &second));
    assert!(authority.allows(4, "file:///sibling.vue.ts", &second));
    assert_eq!(authority.documents.len(), 2);
    authority.record(5, URI, Some(&json!([first])));
    assert!(!authority.allows(5, "file:///sibling.vue.ts", &second));
    assert_eq!(authority.documents.len(), 1);
    authority.record(5, URI, None);
    assert!(!authority.allows(5, URI, &first));
}

#[test]
fn original_list_defaults_and_explicit_item_fields_keep_the_same_raw_capability() {
    let mut authority = IssuedCompletions::default();
    let defaults = json!({"data":{"sessionId":1,"position":3},"insertTextFormat":2});
    let response = json!({"items":[{"label":"greet"},
        {"label":"explicit","data":{"sessionId":2},"insertTextFormat":1},
        {"label":"null","data":null}],"itemDefaults":defaults});
    let original = response.clone();
    authority.record(7, URI, Some(&response));
    assert_eq!(response, original);
    assert!(authority.allows(
        7,
        URI,
        &json!({"label":"greet",
        "data":{"sessionId":1,"position":3},"insertTextFormat":2})
    ));
    assert!(authority.allows(
        7,
        URI,
        &json!({"label":"explicit",
        "data":{"sessionId":2},"insertTextFormat":1})
    ));
    assert!(!authority.allows(7, URI, &response["items"][0]));
    assert!(!authority.allows(7, URI, &response["items"][2]));
}

#[test]
fn malformed_empty_or_null_responses_cannot_lend_previous_capabilities() {
    let item = item();
    let mut authority = IssuedCompletions::default();
    for response in [
        json!(null),
        json!({}),
        json!({"items":false}),
        json!([]),
        json!({"items":[false,{"label":"null","data":null}]}),
    ] {
        authority.record(4, URI, Some(&json!([item])));
        assert!(authority.allows(4, URI, &item));
        authority.record(4, URI, Some(&response));
        assert!(!authority.allows(4, URI, &item));
        assert!(authority.documents.is_empty());
    }
}
