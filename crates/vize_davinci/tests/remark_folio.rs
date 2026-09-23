//! The `[remarks]` page and the remark JSON document (P3-13), pinned
//! exactly: canonical text, both round-trip laws, `Display` elision, every
//! parse rejection's message, and the JSON document validated against the
//! committed schema (`docs/davinci/plan/remarks.schema.json`) through the
//! shared strict validator (TS-15).

use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_davinci::folio::remarks::{RemarkLog, RemarksSchemaMismatch};
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_davinci::pass::RemarkKind;
use vize_davinci::pass::observer::{RecordedArg, RecordedRemark, RemarkArgValue};
use vize_s0::{Span, String};

fn arg(key: &str, value: RemarkArgValue) -> RecordedArg {
    RecordedArg {
        key: String::from(key),
        value,
    }
}

fn text(value: &str) -> RemarkArgValue {
    RemarkArgValue::Str(String::from(value))
}

/// Two passes, all three kinds, every value type, and a string argument
/// needing every escape class.
fn sample() -> RemarkLog {
    RemarkLog::new(vec![
        RecordedRemark {
            stage: String::from("s2"),
            pass: String::from("hoist-static"),
            kind: RemarkKind::Missed,
            name: String::from("static-subtree"),
            span: Span::new(0, 120),
            args: vec![
                arg("tag", text("section")),
                arg("blocker", text("child")),
                arg("op", text("ui.component")),
            ],
        },
        RecordedRemark {
            stage: String::from("s2"),
            pass: String::from("hoist-static"),
            kind: RemarkKind::Applied,
            name: String::from("static-props"),
            span: Span::new(0, 120),
            args: vec![arg("tag", text("section"))],
        },
        RecordedRemark {
            stage: String::from("s3"),
            pass: String::from("partition"),
            kind: RemarkKind::Analysis,
            name: String::from("region-size"),
            span: Span::new(7, 7),
            args: vec![
                arg("ops", RemarkArgValue::Int(-3)),
                arg("static", RemarkArgValue::Bool(false)),
                arg("label", text("a \"b\" c\\d\ne\u{1}f \u{3042}")),
            ],
        },
    ])
}

const SAMPLE_FULL: &str = "[remarks]\n\n[remarks.entries]\n\
s2.hoist-static missed static-subtree @0:120 tag=\"section\" blocker=\"child\" op=\"ui.component\"\n\
s2.hoist-static applied static-props @0:120 tag=\"section\"\n\
s3.partition analysis region-size @7:7 ops=-3 static=false label=\"a \\\"b\\\" c\\\\d\\ne\\u0001f \u{3042}\"\n\n";

#[test]
fn the_page_prints_canonically_and_round_trips() {
    let log = sample();
    let printed = log.print_to_string(FolioMode::Full);
    assert_eq!(printed.as_str(), SAMPLE_FULL);
    assert_eq!(RemarkLog::parse(SAMPLE_FULL), Ok(log));
    let reparsed = RemarkLog::parse(SAMPLE_FULL).expect("canonical text parses");
    assert_eq!(
        reparsed.print_to_string(FolioMode::Full).as_str(),
        SAMPLE_FULL
    );
}

#[test]
fn an_empty_log_is_the_bare_header() {
    let empty = RemarkLog::default();
    assert_eq!(
        empty.print_to_string(FolioMode::Full).as_str(),
        "[remarks]\n\n"
    );
    assert_eq!(RemarkLog::parse("[remarks]\n\n"), Ok(empty));
}

#[test]
fn display_elides_spans_only() {
    assert_eq!(
        sample().print_to_string(FolioMode::Display).as_str(),
        "[remarks]\n\n[remarks.entries]\n\
         s2.hoist-static missed static-subtree tag=\"section\" blocker=\"child\" op=\"ui.component\"\n\
         s2.hoist-static applied static-props tag=\"section\"\n\
         s3.partition analysis region-size ops=-3 static=false label=\"a \\\"b\\\" c\\\\d\\ne\\u0001f \u{3042}\"\n\n"
    );
}

#[test]
fn a_unicode_escape_normalizes_on_the_first_print() {
    let input = "[remarks]\n\n[remarks.entries]\ns2.p applied n @1:2 k=\"\\u0041\\u3042\"\n\n";
    let log = RemarkLog::parse(input).expect("escaped text parses");
    assert_eq!(
        log.print_to_string(FolioMode::Full).as_str(),
        "[remarks]\n\n[remarks.entries]\ns2.p applied n @1:2 k=\"A\u{3042}\"\n\n"
    );
}

fn reject(entry: &str) -> FolioError {
    let input = vize_s0::cstr!("[remarks]\n\n[remarks.entries]\n{entry}\n");
    RemarkLog::parse(input.as_str()).expect_err("malformed entry must fail")
}

#[test]
fn every_malformed_entry_is_rejected_with_its_line_and_reason() {
    let cases: [(&str, &str); 14] = [
        (
            "hoist applied n @1:2",
            "remark origin `hoist` is not `stage.pass`",
        ),
        (
            "S2.p applied n @1:2",
            "stage `S2` is not a lowercase kebab-case identifier",
        ),
        (
            "s2.p- applied n @1:2",
            "pass `p-` is not a lowercase kebab-case identifier",
        ),
        (
            "s2.p done n @1:2",
            "unknown remark kind `done` (expected applied, missed, analysis)",
        ),
        (
            "s2.p applied N @1:2",
            "remark name `N` is not a lowercase kebab-case identifier",
        ),
        (
            "s2.p applied n 1:2",
            "remark span `1:2` is not `@start:end` with start <= end",
        ),
        (
            "s2.p applied n @3:2",
            "remark span `@3:2` is not `@start:end` with start <= end",
        ),
        (
            "s2.p applied n @01:2",
            "remark span `@01:2` is not `@start:end` with start <= end",
        ),
        (
            "s2.p applied n @1:2 k",
            "remark argument `k` is missing `=`",
        ),
        (
            "s2.p applied n @1:2 k=1 k=2",
            "duplicate remark argument `k`",
        ),
        (
            "s2.p applied n @1:2 k=01",
            "remark value `01` is not a string, integer, or boolean",
        ),
        (
            "s2.p applied n @1:2 k=-0",
            "remark value `-0` is not a string, integer, or boolean",
        ),
        (
            "s2.p applied n @1:2 k=\"x\"y",
            "unexpected `y` after a string value",
        ),
        ("s2.p applied n @1:2 k=\"x", "unterminated string value"),
    ];
    for (entry, message) in cases {
        assert_eq!(
            reject(entry),
            FolioError::new(4, String::from(message)),
            "{entry}"
        );
    }
    assert_eq!(
        RemarkLog::parse("[remarks]\ncount=1\n"),
        Err(FolioError::new(2, String::from("unknown field `count`")))
    );
    assert_eq!(
        RemarkLog::parse("[remarks]\n\n[remarks.notes]\n"),
        Err(FolioError::new(
            3,
            String::from("unknown section [remarks.notes]")
        ))
    );
    assert_eq!(
        RemarkLog::parse(""),
        Err(FolioError::new(0, String::from("missing [remarks] header")))
    );
}

fn load_schema() -> serde_json::Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/davinci/plan/remarks.schema.json");
    let text = std::fs::read_to_string(path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}

#[test]
fn the_json_document_is_exact_and_schema_valid() {
    let json = sample().to_json("davinci-opt");
    let parsed: serde_json::Value = serde_json::from_str(json.as_str()).expect("valid JSON");
    assert_eq!(schema_check::validate(&load_schema(), &parsed, "$"), Ok(()));
    assert_eq!(
        parsed,
        serde_json::json!({
            "schema_version": 1,
            "command": "davinci-opt",
            "remarks": [
                {
                    "stage": "s2", "pass": "hoist-static", "kind": "missed",
                    "name": "static-subtree", "span": { "start": 0, "end": 120 },
                    "args": [
                        { "key": "tag", "value": "section" },
                        { "key": "blocker", "value": "child" },
                        { "key": "op", "value": "ui.component" },
                    ],
                },
                {
                    "stage": "s2", "pass": "hoist-static", "kind": "applied",
                    "name": "static-props", "span": { "start": 0, "end": 120 },
                    "args": [{ "key": "tag", "value": "section" }],
                },
                {
                    "stage": "s3", "pass": "partition", "kind": "analysis",
                    "name": "region-size", "span": { "start": 7, "end": 7 },
                    "args": [
                        { "key": "ops", "value": -3 },
                        { "key": "static", "value": false },
                        { "key": "label", "value": "a \"b\" c\\d\ne\u{1}f \u{3042}" },
                    ],
                },
            ],
        })
    );
    assert_eq!(
        json.as_str().split('\n').count(),
        2,
        "one line plus newline"
    );
}

#[test]
fn the_schema_refuses_version_and_shape_drift() {
    let schema = load_schema();
    let mut parsed: serde_json::Value =
        serde_json::from_str(sample().to_json("davinci-opt").as_str()).expect("valid JSON");
    parsed["remarks"][0]["kind"] = serde_json::Value::from("hoisted");
    let error = schema_check::validate(&schema, &parsed, "$").expect_err("bad kind fails");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.remarks[0].kind`: string does not match pattern \
         `^(applied|missed|analysis)$`"
    );
    parsed["remarks"][0]["kind"] = serde_json::Value::from("missed");
    parsed["remarks"][0]["args"][0]["value"] = serde_json::json!(1.5);
    let error = schema_check::validate(&schema, &parsed, "$").expect_err("float fails");
    assert_eq!(
        error.message().as_str(),
        "schema violation at `$.remarks[0].args[0].value`: expected \
         [string,integer,boolean], found number"
    );
    assert_eq!(RemarkLog::negotiate_schema_version(1), Ok(()));
    assert_eq!(
        RemarkLog::negotiate_schema_version(2),
        Err(RemarksSchemaMismatch {
            expected: 1,
            found: 2,
        })
    );
}

#[test]
fn counts_read_the_log() {
    let log = sample();
    assert_eq!(RemarkKind::ALL.map(|kind| log.count(kind)), [1, 1, 1]);
}
