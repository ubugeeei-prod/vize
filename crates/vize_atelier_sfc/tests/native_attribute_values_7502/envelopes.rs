//! Independently authored source envelopes confer no styled-output authority.
use super::{
    Test,
    capture::{self, Target},
    check,
    inputs::{self, Inputs},
};
use serde_json::{Value, json};
use vize_l0::cstr;
use vize_l1::container::vue::ScriptRole;
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind;

pub fn packet(inputs: &Inputs) -> Value {
    let mut rows = Vec::new();
    for fixture in &inputs.fixtures {
        for (id, suffix, expected) in [
            (
                "ordinary",
                "<script></script>",
                NativeSelectedSfcIssueKind::Script(ScriptRole::Ordinary),
            ),
            (
                "setup",
                "<script setup></script>",
                NativeSelectedSfcIssueKind::Script(ScriptRole::Setup),
            ),
            (
                "style",
                "<style></style>",
                NativeSelectedSfcIssueKind::Style,
            ),
            (
                "scoped-style",
                "<style scoped>.x{color:red}</style>",
                NativeSelectedSfcIssueKind::Style,
            ),
        ] {
            for target in [Target::Dom, Target::Ssr, Target::Vapor] {
                for source_map in [true, false] {
                    let source = cstr!("<template>{}</template>{suffix}", fixture.source);
                    let mut row =
                        capture::compile_row(fixture, target, source_map, &source, Some(expected));
                    if let Some(object) = row.as_object_mut() {
                        object.insert("envelopeId".into(), json!(id));
                        object.insert("envelopeSuffix".into(), json!(suffix));
                        object.insert("expectedIssue".into(), json!(cstr!("{expected:?}")));
                    }
                    rows.push(row);
                }
            }
        }
    }
    json!({"schema":"vize.native-attribute-values-7502.envelopes", "version":1,
        "custody":"once-selected-original", "fixtureSha256":inputs::PACK_SHA256,
        "ledgerSha256":inputs.ledger_sha256,
        "authority":"independently authored script/style carrier refusal controls; no styled runtime credit",
        "carrier":"<template>{source}</template>{envelopeSuffix}",
        "summary":{"fixtures":inputs.fixtures.len(),"envelopes":4,"outcomes":rows.len()},"rows":rows})
}

fn field<'a>(value: &'a Value, name: &str) -> Test<&'a Value> {
    value
        .get(name)
        .ok_or_else(|| cstr!("envelope field {name}"))
}

pub fn validate(packet: &Value) -> Test {
    let rows = field(packet, "rows")?.as_array().ok_or("envelope rows")?;
    check(
        rows.len() == 336,
        "every independently authored envelope/target/link mode",
    )?;
    for row in rows {
        check(
            field(row, "envelopeIssueMatches")? == &json!(true),
            "actual typed Script Ordinary/Setup or Style issue",
        )?;
        let observation = field(row, "observation")?;
        check(
            field(observation, "descriptorSameSource")? == &json!(true)
                && field(observation, "admitted")? == &json!(false)
                && field(observation, "selected")?.is_null()
                && field(observation, "file")?.is_null()
                && field(observation, "rejectedCreationDebug")?.is_null(),
            "source envelope refuses before selected template creation",
        )?;
        let source_issues = field(observation, "sourceIssues")?
            .as_array()
            .ok_or("original source issues")?;
        check(source_issues.len() == 1, "one actual source envelope issue")?;
        let issue = source_issues
            .first()
            .ok_or("original source envelope issue")?;
        check(
            field(issue, "kind")? == field(row, "expectedIssue")?,
            "complete typed original source issue",
        )?;
        let result = field(row, "result")?;
        check(
            field(result, "classification")? == &json!("lower-refusal")
                && field(result, "publicError")?.is_string()
                && field(result, "code")?.is_null()
                && field(result, "mapText")?.is_null()
                && field(result, "map")?.is_null()
                && field(result, "links")?
                    .as_array()
                    .is_some_and(Vec::is_empty),
            "no partial carrier output",
        )?;
    }
    Ok(())
}
