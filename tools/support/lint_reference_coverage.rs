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
    let rule = finding["ruleId"].as_str()?;
    static REVIEWED: OnceLock<Value> = OnceLock::new();
    let contract = REVIEWED.get_or_init(|| {
        serde_json::from_str(CONTRACT).expect("reviewed reference coverage contract")
    });
    let configured_severity = match finding["severity"].as_str()? {
        "warning" => "warn",
        "error" => "error",
        _ => return None,
    };
    let case = contract["cases"].as_array()?.iter().find(|case| {
        case["project"] == project
            && case["revision"] == context["revision"]
            && case["preset"] == context["preset"]
            && case["runtimeVersions"] == context["runtimeVersions"]
            && case["finding"] == *finding
            && context["rules"][rule] == configured_severity
    })?;
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

#[cfg(test)]
#[path = "lint_reference_coverage/tests.rs"]
mod tests;
