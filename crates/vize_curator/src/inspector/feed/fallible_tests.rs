use super::{StageFeed, StagePage, StageUnavailable};
use davinci_test_support::schema;
use vize_l0::String;

#[test]
fn empty_and_successful_ladder_feeds_keep_the_existing_wire_bytes() {
    let mut feed = StageFeed::new("analyze-sfc");
    assert_eq!(
        feed.to_json().as_str(),
        "{\"schema_version\":1,\"command\":\"analyze-sfc\",\"pages\":[],\"remarks\":[]}\n"
    );
    feed.pages.push(StagePage {
        path: Some(String::from("src/App.vue")),
        stage: String::from("s1"),
        pass: String::from("parse"),
        text: String::from("<p>λ</p>"),
    });
    assert_eq!(
        feed.to_json().as_str(),
        "{\"schema_version\":1,\"command\":\"analyze-sfc\",\"pages\":[{\"path\":\"src/App.vue\",\"stage\":\"s1\",\"pass\":\"parse\",\"text\":\"<p>λ</p>\"}],\"remarks\":[]}\n"
    );
}

#[test]
fn failed_ladder_inspections_escape_full_reasons_and_validate_separately() {
    let mut feed = StageFeed::new("analyze-sfc");
    let reason = "cannot print \"binding\"\\\nλ\u{1}";
    feed.unavailable.push(StageUnavailable {
        path: Some(String::from("src/App.vue")),
        stage: String::from("s2"),
        pass: String::from("hoist-static"),
        reason: String::from(reason),
    });
    let json = feed.to_json();
    let parsed: serde_json::Value = serde_json::from_str(json.as_str()).unwrap();
    let schema_value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../docs/davinci/plan/spolvero-feed.schema.json"
    ))
    .unwrap();
    assert_eq!(schema::validate(&schema_value, &parsed, "$"), Ok(()));
    assert!(parsed["pages"].as_array().unwrap().is_empty());
    assert_eq!(parsed["unavailable"][0]["reason"], reason);
    assert_eq!(parsed["unavailable"][0]["pass"], "hoist-static");
    let mut invalid = parsed;
    invalid["unavailable"][0]["text"] = serde_json::json!("invalid page substitution");
    assert!(schema::validate(&schema_value, &invalid, "$").is_err());
}
