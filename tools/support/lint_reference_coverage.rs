//! Exact reviewed findings which the pinned reference cannot inspect.
//!
//! This does not infer a rule-wide exemption. Changed sources, settings,
//! runtimes, revisions, messages, severities or ranges remain scored.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

const CONTRACT: &str =
    include_str!("../../tests/_fixtures/differential/linter/reference-coverage-8849/contract.json");

pub(crate) fn classify(
    project: &str,
    context: &Value,
    finding: &Value,
    source: &str,
    has_reference_finding: impl FnOnce() -> bool,
) -> Option<Value> {
    let case = reviewed()["cases"]
        .as_array()?
        .iter()
        .find(|case| settings_match(case, project, context) && case["finding"] == *finding)?;
    let source_hash = format!("{:x}", Sha256::digest(source.as_bytes()));
    if case["sourceSha256"] != source_hash || has_reference_finding() {
        return None;
    }
    Some(json!({
        "case": case["id"], "category": "reference-coverage-gap",
        "issue": case["issue"], "reason": case["reason"], "revision": case["revision"],
        "sourceSha256": source_hash, "runtimeVersions": case["runtimeVersions"],
        "options": case["options"], "upstreamOptions": case["upstreamOptions"],
        "packetSha256": case["packetSha256"], "nativePacketSha256": case["nativePacketSha256"],
    }))
}

fn reviewed() -> &'static Value {
    static REVIEWED: OnceLock<Value> = OnceLock::new();
    REVIEWED.get_or_init(|| {
        serde_json::from_str(CONTRACT).expect("reviewed reference coverage contract")
    })
}

fn settings_match(case: &Value, project: &str, context: &Value) -> bool {
    let Some(rule) = case["finding"]["ruleId"].as_str() else {
        return false;
    };
    let severity = match case["finding"]["severity"].as_str() {
        Some("warning") => "warn",
        Some("error") => "error",
        _ => return false,
    };
    case["project"] == project
        && case["revision"] == context["revision"]
        && case["preset"] == context["preset"]
        && case["runtimeVersions"] == context["runtimeVersions"]
        && context["rules"][rule] == severity
}

/// Discover eligible expected findings independently of the native answer.
/// Only measured files participate. An absent native row must not become a
/// passing comparison merely because the reference also cannot see it.
pub(crate) fn eligible(
    project: &str,
    context: &Value,
    mut source_for: impl FnMut(&str) -> Option<String>,
) -> Vec<Value> {
    reviewed()["cases"]
        .as_array()
        .expect("reviewed case list")
        .iter()
        .filter(|case| {
            settings_match(case, project, context)
                && context["files"]
                    .as_array()
                    .is_some_and(|files| files.contains(&case["finding"]["file"]))
                && case["finding"]["file"]
                    .as_str()
                    .and_then(&mut source_for)
                    .is_some_and(|source| {
                        case["sourceSha256"] == format!("{:x}", Sha256::digest(source.as_bytes()))
                    })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "lint_reference_coverage/tests.rs"]
mod tests;
