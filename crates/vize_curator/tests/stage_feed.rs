//! The historical stage feed v1 (P2-18), pinned through typed collectors.
//! The document validates against the committed schema
//! (`docs/davinci/plan/spolvero-feed.schema.json`) through the shared strict
//! validator (TS-15), and its content equals the dump's pages
//! exactly, escaping law included.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_davinci::dump::collector::Collector;
use vize_davinci::dump::feed::{StageFeed, StageFeedSchemaMismatch};
use vize_davinci::pass::{
    BudgetObserver, Fusability, PassDesc, PassEvent, PassKind, Pipeline, Preserved, run_pipeline,
};

/// A canonical `[budget-observer]` page - the smallest committed-format
/// artifact a pipeline can run over.
const BUDGET: &str =
    "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n";

fn collected_feed(after_change_only: bool) -> serde_json::Value {
    const ALPHA: PassDesc = PassDesc::new(
        "alpha",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::ALL,
    );
    const BETA: PassDesc = PassDesc::new(
        "beta",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::ALL,
    );
    const PASSES: &[PassDesc] = &[ALPHA, BETA];
    const PIPELINE: Pipeline = Pipeline::new("l2", PASSES);
    let mut collector = Collector::new(after_change_only);
    collector.seed(BUDGET);
    let mut budget = BudgetObserver::new();
    run_pipeline(&PIPELINE, &mut budget, |event| {
        collector.after_pass(event, BUDGET);
        Ok(())
    })
    .expect("no-op body cannot fail");
    assert_eq!((budget.walks, budget.passes), (1, 2));
    let feed = StageFeed::of_dump("typed-pass-test", &collector);
    serde_json::from_str(feed.to_json().as_str()).expect("feed is valid JSON")
}

fn negotiate_feed_schema(feed: &serde_json::Value) -> Result<(), SchemaGateError> {
    let version = feed
        .get("schema_version")
        .ok_or(SchemaGateError::MissingVersion)?
        .as_u64()
        .ok_or(SchemaGateError::NonNumericVersion)?;
    let Ok(version) = u32::try_from(version) else {
        return Err(SchemaGateError::NonNumericVersion);
    };
    StageFeed::negotiate_schema_version(version).map_err(SchemaGateError::VersionMismatch)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaGateError {
    MissingVersion,
    NonNumericVersion,
    VersionMismatch(StageFeedSchemaMismatch),
}

/// Load the committed schema relative to this crate's manifest.
fn load_schema() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs/davinci")
        .join("plan")
        .join("spolvero-feed.schema.json");
    let text = std::fs::read_to_string(&path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}

#[test]
fn typed_feed_validates_and_carries_collected_pages_exactly() {
    let feed = collected_feed(false);
    assert_eq!(schema_check::validate(&load_schema(), &feed, "$"), Ok(()));
    assert_eq!(
        feed,
        serde_json::json!({
            "schema_version": 1,
            "command": "typed-pass-test",
            "pages": [
                { "path": null, "stage": "l2", "pass": "alpha", "text": BUDGET },
                { "path": null, "stage": "l2", "pass": "beta", "text": BUDGET },
            ],
            "remarks": [],
        })
    );
}

#[test]
fn a_fully_gated_collector_feeds_zero_pages_loudly() {
    // The gate emitted nothing, and the feed says so instead of being
    // absent: an empty dump is observable, not indistinguishable from an
    // ignored flag.
    let feed = collected_feed(true);
    assert_eq!(schema_check::validate(&load_schema(), &feed, "$"), Ok(()));
    assert_eq!(
        feed,
        serde_json::json!({
            "schema_version": 1,
            "command": "typed-pass-test",
            "pages": [],
            "remarks": [],
        })
    );
}

#[test]
fn the_feed_escapes_page_text_into_valid_json_exactly() {
    const ALPHA: PassDesc = PassDesc::new(
        "alpha",
        PassKind::Optional,
        Fusability::Fusable,
        Preserved::ALL,
    );
    const PASSES: &[PassDesc] = &[ALPHA];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);
    let event = PassEvent {
        pipeline: &PIPELINE,
        group_index: 0,
        group: PIPELINE.group(0).expect("one group"),
        pass_index: 0,
    };

    // Every escape class: quote, backslash, the three named controls, an
    // unnamed control, and multi-byte UTF-8.
    let nasty = "a\"b\\c\nd\re\tf\u{1}g\u{3042}\n";
    let mut dump = Collector::new(false);
    dump.after_pass(&event, nasty);
    let feed = StageFeed::of_dump("typed-pass-test", &dump);

    let json = feed.to_json();
    assert_eq!(
        json.as_str(),
        "{\"schema_version\":1,\"command\":\"typed-pass-test\",\"pages\":[{\"path\":null,\
         \"stage\":\"s2\",\"pass\":\"alpha\",\
         \"text\":\"a\\\"b\\\\c\\nd\\re\\tf\\u0001g\u{3042}\\n\"}],\"remarks\":[]}\n"
    );
    let parsed: serde_json::Value = serde_json::from_str(json.as_str()).expect("feed parses");
    assert_eq!(schema_check::validate(&load_schema(), &parsed, "$"), Ok(()));
    assert_eq!(parsed["pages"][0]["text"], serde_json::json!(nasty));
}

#[test]
fn the_schema_refuses_version_and_shape_mismatches_loudly() {
    let schema = load_schema();
    let valid = serde_json::json!({
        "schema_version": 1,
        "command": "typed-pass-test",
        "pages": [{ "path": null, "stage": "s2", "pass": "alpha", "text": "" }],
    });
    assert_eq!(schema_check::validate(&schema, &valid, "$"), Ok(()));

    let mut wrong_version = valid.clone();
    wrong_version["schema_version"] = serde_json::Value::from(2);
    let error =
        schema_check::validate(&schema, &wrong_version, "$").expect_err("wrong version must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.schema_version`: value does not equal const `1`"
    );

    let mut extra_key = valid.clone();
    extra_key
        .as_object_mut()
        .expect("feed is an object")
        .insert("transport".into(), serde_json::Value::from("tbd"));
    let error = schema_check::validate(&schema, &extra_key, "$").expect_err("extra key must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$`: unexpected property `transport`"
    );

    let mut page_missing_text = valid.clone();
    page_missing_text["pages"][0]
        .as_object_mut()
        .expect("page is an object")
        .remove("text");
    let error = schema_check::validate(&schema, &page_missing_text, "$")
        .expect_err("missing page text must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.pages[0]`: missing required property `text`"
    );

    let mut bad_stage = valid;
    bad_stage["pages"][0]["stage"] = serde_json::Value::from("L2 disegno");
    let error = schema_check::validate(&schema, &bad_stage, "$").expect_err("bad stage must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.pages[0].stage`: string does not match pattern \
         `^[a-z][a-z0-9-]*$`"
    );
}

#[test]
fn consumers_negotiate_schema_version_before_reading_pages() {
    let mut feed = serde_json::json!({
        "schema_version": 1,
        "command": "typed-pass-test",
        "pages": "not read until the version is accepted",
    });
    assert_eq!(negotiate_feed_schema(&feed), Ok(()));

    feed.as_object_mut()
        .expect("feed is an object")
        .remove("schema_version");
    assert_eq!(
        negotiate_feed_schema(&feed),
        Err(SchemaGateError::MissingVersion)
    );

    feed["schema_version"] = serde_json::Value::from("1");
    assert_eq!(
        negotiate_feed_schema(&feed),
        Err(SchemaGateError::NonNumericVersion)
    );

    feed["schema_version"] = serde_json::Value::from(2);
    assert_eq!(
        negotiate_feed_schema(&feed),
        Err(SchemaGateError::VersionMismatch(StageFeedSchemaMismatch {
            expected: 1,
            found: 2,
        }))
    );
}

#[test]
fn the_feed_carries_remarks_beside_the_pages() {
    use vize_davinci::dump::feed::StageRemark;
    use vize_davinci::pass::RemarkKind;
    use vize_davinci::pass::observer::{RecordedArg, RecordedRemark, RemarkArgValue};
    use vize_l0::{Span, String};

    let arg = |key: &str, value: &str| RecordedArg {
        key: String::from(key),
        value: RemarkArgValue::Str(String::from(value)),
    };
    let mut feed = StageFeed::new("inspector");
    feed.remarks.push(StageRemark {
        path: Some(String::from("src/App.vue")),
        remark: RecordedRemark {
            stage: String::from("s2"),
            pass: String::from("hoist-static"),
            kind: RemarkKind::Missed,
            name: String::from("static-props"),
            span: Span::new(10, 42),
            args: vec![arg("tag", "button"), arg("blocker", "binding")],
        },
    });
    let json = feed.to_json();
    assert_eq!(
        json.as_str(),
        "{\"schema_version\":1,\"command\":\"inspector\",\"pages\":[],\"remarks\":[\
         {\"path\":\"src/App.vue\",\"stage\":\"s2\",\"pass\":\"hoist-static\",\
         \"kind\":\"missed\",\"name\":\"static-props\",\"span\":{\"start\":10,\"end\":42},\
         \"args\":[{\"key\":\"tag\",\"value\":\"button\"},\
         {\"key\":\"blocker\",\"value\":\"binding\"}]}]}\n"
    );
    let mut parsed: serde_json::Value = serde_json::from_str(json.as_str()).expect("feed parses");
    let schema = load_schema();
    assert_eq!(schema_check::validate(&schema, &parsed, "$"), Ok(()));

    // A pre-P3-13 v1 document (no `remarks` member) stays valid: the member
    // is additive.
    let mut earlier = parsed.clone();
    earlier
        .as_object_mut()
        .expect("feed is an object")
        .remove("remarks");
    assert_eq!(schema_check::validate(&schema, &earlier, "$"), Ok(()));

    parsed["remarks"][0]["kind"] = serde_json::Value::from("hoisted");
    let error = schema_check::validate(&schema, &parsed, "$").expect_err("bad kind fails");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.remarks[0].kind`: string does not match pattern \
         `^(applied|missed|analysis)$`"
    );
}
