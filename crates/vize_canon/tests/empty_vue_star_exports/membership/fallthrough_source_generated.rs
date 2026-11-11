//! Exact generated successors for #12793: a fallthrough-bearing export names
//! its raw surface in `__vizeFallthroughSource`.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/fallthrough-source-generated-12793.json";
const RECORD_SHA: &str = "ba6e84b393609f874431b00d4a8adce9d51d44190eb6a539d4260f27f6d1218a";
const MARKER: &str =
    " readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: ";
const UNOWNED: &str = "$UnownedComponentAttrs<";

fn record() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(
        record["kind"],
        "exact-fallthrough-source-generated-successor"
    );
    assert_eq!(record["issue"], 12431);
    assert_eq!(record["pullRequest"], 12793);
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

/// The raw surface the marker forwards: the first `$UnownedComponentAttrs`
/// argument, or the whole forwarded type.
fn source_of(forwarded: &str) -> &str {
    let Some(arguments) = forwarded.strip_prefix(UNOWNED) else {
        return forwarded;
    };
    let mut depth = 0_i32;
    for (at, byte) in arguments.bytes().enumerate() {
        match byte {
            b'<' | b'(' | b'[' | b'{' => depth += 1,
            b'>' | b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => return arguments.get(..at).unwrap(),
            _ => {}
        }
    }
    panic!("unterminated owned-key map: {forwarded}")
}

/// The one insertion on the authenticated predecessor bytes.
fn successor_body(current: &str, entry: &Value) -> String {
    assert_eq!(json!(current.len()), entry["before"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["before"][2]);
    assert_eq!(current.matches(MARKER).count(), 1);
    assert_eq!(current.matches("__vizeFallthroughSource").count(), 0);
    let at = current.find(MARKER).unwrap();
    let forwarded = current.get(at + MARKER.len()..).unwrap();
    let forwarded = forwarded.get(..forwarded.find(';').unwrap()).unwrap();
    let insertion = format!(
        " readonly __vizeFallthroughSource?: {};",
        source_of(forwarded)
    );
    let successor = format!(
        "{}{insertion}{}",
        current.get(..at).unwrap(),
        current.get(at..).unwrap()
    );
    assert_eq!(
        (
            json!(successor.len()),
            json!(support::hash(successor.as_bytes()))
        ),
        (entry["after"][1].clone(), entry["after"][2].clone()),
        "{}",
        entry["after"][0]
    );
    assert_eq!(json!(insertion.len()), entry["deltaBytes"]);
    successor
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let record = record();
    let entries = record["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    for (index, entry) in entries.iter().enumerate() {
        let member_index = index + 74;
        assert_eq!(entry["entry"], member_index);
        assert_eq!(entry["after"][0], entry["before"][0]);
        let member = members.get_mut(member_index).unwrap();
        assert_eq!(*member, entry["before"]);
        *member = entry["after"].clone();
    }
    manifest["provenance"]["currentFallthroughSourceGenerated12793"] = record;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_generated_successors_reconstruct_from_the_authenticated_predecessors() {
        let record = record();
        let bodies = super::super::setup_verbatim_generated::current_bodies();
        let entries = record["entries"].as_array().unwrap();
        assert_eq!(bodies.len(), entries.len());
        for (body, entry) in bodies.iter().zip(entries) {
            successor_body(body, entry);
            for changed in [format!("{body}\n"), body.replacen(MARKER, "", 1)] {
                assert!(std::panic::catch_unwind(|| successor_body(&changed, entry)).is_err());
            }
        }
    }
}
