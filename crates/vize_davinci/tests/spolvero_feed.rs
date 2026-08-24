//! The Spolvero feed v1 (P2-18), pinned end to end on the `davinci-opt`
//! surface: `--folio-dir` writes `spolvero.json` beside the pages, the
//! document validates against the committed schema
//! (`davinci-road/plan/spolvero-feed.schema.json`) through the shared strict
//! validator (TS-15), and its content equals the dump's pages
//! exactly, escaping law included.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use davinci_test_support::schema as schema_check;
use vize_davinci::folio::dump::FolioDump;
use vize_davinci::folio::feed::SpolveroFeed;
use vize_davinci::pass::{Fusability, PassDesc, PassEvent, PassKind, Pipeline, Preserved};

/// A canonical `[budget-observer]` page - the smallest committed-format
/// artifact a pipeline can run over.
const BUDGET: &str =
    "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n";

fn run_folio_dir(dir: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "--pipeline",
        "s2(alpha,beta)",
        "--stage",
        "budget-observer",
        "--folio-dir",
        dir.to_str().expect("UTF-8 dir"),
    ];
    args.extend_from_slice(extra);
    let mut child = Command::new(env!("CARGO_BIN_EXE_davinci-opt"))
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("davinci-opt spawns");
    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(BUDGET.as_bytes())
        .expect("stdin accepts the folio");
    drop(child.stdin.take());
    child.wait_with_output().expect("davinci-opt exits")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn read_feed(dir: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(dir.join("spolvero.json")).expect("feed reads");
    serde_json::from_str(&text).expect("feed is valid JSON")
}

/// Load the committed schema relative to this crate's manifest.
fn load_schema() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("davinci-road")
        .join("plan")
        .join("spolvero-feed.schema.json");
    let text = std::fs::read_to_string(&path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}

#[test]
fn folio_dir_feed_validates_and_carries_the_dump_pages_exactly() {
    let dir = temp_dir("p2-18-feed");
    let output = run_folio_dir(&dir, &[]);
    assert_eq!(output.status.code(), Some(0));

    let feed = read_feed(&dir);
    assert_eq!(schema_check::validate(&load_schema(), &feed, "$"), Ok(()));
    assert_eq!(
        feed,
        serde_json::json!({
            "schema_version": 1,
            "command": "davinci-opt",
            "pages": [
                { "path": null, "stage": "s2", "pass": "alpha", "text": BUDGET },
                { "path": null, "stage": "s2", "pass": "beta", "text": BUDGET },
            ],
        })
    );
}

#[test]
fn a_fully_gated_dump_feeds_zero_pages_loudly() {
    let dir = temp_dir("p2-18-feed-gated");
    let output = run_folio_dir(&dir, &["--folio-after-change"]);
    assert_eq!(output.status.code(), Some(0));

    // The gate emitted nothing, and the feed says so instead of being
    // absent: an empty dump is observable, not indistinguishable from an
    // ignored flag.
    let feed = read_feed(&dir);
    assert_eq!(schema_check::validate(&load_schema(), &feed, "$"), Ok(()));
    assert_eq!(
        feed,
        serde_json::json!({
            "schema_version": 1,
            "command": "davinci-opt",
            "pages": [],
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
    let mut dump = FolioDump::new(false);
    dump.after_pass(&event, nasty);
    let feed = SpolveroFeed::of_dump("davinci-opt", &dump);

    let json = feed.to_json();
    assert_eq!(
        json.as_str(),
        "{\"schema_version\":1,\"command\":\"davinci-opt\",\"pages\":[{\"path\":null,\
         \"stage\":\"s2\",\"pass\":\"alpha\",\
         \"text\":\"a\\\"b\\\\c\\nd\\re\\tf\\u0001g\u{3042}\\n\"}]}\n"
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
        "command": "davinci-opt",
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
    bad_stage["pages"][0]["stage"] = serde_json::Value::from("S2 disegno");
    let error = schema_check::validate(&schema, &bad_stage, "$").expect_err("bad stage must fail");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.pages[0].stage`: string does not match pattern \
         `^[a-z][a-z0-9-]*$`"
    );
}
