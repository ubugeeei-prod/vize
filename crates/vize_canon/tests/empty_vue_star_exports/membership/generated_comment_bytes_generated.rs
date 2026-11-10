//! Exact generated successors for the generated-comment byte cut: the debug-only
//! `@vize-map` comments and the reasons after `@ts-ignore` leave each body.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/generated-comment-bytes-generated.json";
const RECORD_SHA: &str = "dfa919a6c1a6d8c2a9f3d1c93afc2a25810e79950640376031c8f0dabe10c30e";

/// `@ts-ignore` reasons the content mapper still reads back.
const KEPT_REASONS: [&str; 2] = [
    "Navigation-only references; no diagnostic owner.",
    "Named prop diagnostics are owned by the per-prop check.",
];

fn record() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-generated-comment-bytes-successor");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

/// A source-only map comment: it carries no generated offsets.
fn is_debug_map(line: &str) -> bool {
    let trimmed = line.trim_start();
    ["expr", "prop", "handler", "component"]
        .iter()
        .any(|kind| trimmed.starts_with(&format!("// @vize-map: {kind} -> ")))
}

/// Whether `after` is `before` without its debug map lines and with each
/// changed `@ts-ignore` line reduced to the bare directive.
fn only_comment_cut(before: &str, after: &str) -> bool {
    let mut kept = before
        .split_inclusive('\n')
        .filter(|line| !is_debug_map(line));
    let mut cut = after.split_inclusive('\n');
    loop {
        match (kept.next(), cut.next()) {
            (None, None) => return true,
            (Some(old), Some(new)) if old == new => {}
            (Some(old), Some(new)) => {
                let Some((head, reason)) = old.split_once("// @ts-ignore ") else {
                    return false;
                };
                let text = reason.trim_end_matches(['\r', '\n']);
                let ending = &reason[text.len()..];
                if KEPT_REASONS.contains(&text) || new != format!("{head}// @ts-ignore{ending}") {
                    return false;
                }
            }
            _ => return false,
        }
    }
}

/// The entry's hunks on the authenticated predecessor bytes; each only cuts comment bytes.
fn successor_body(current: &str, entry: &Value) -> String {
    assert_eq!(json!(current.len()), entry["before"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["before"][2]);
    let mut successor = current.to_owned();
    for hunk in entry["hunks"].as_array().unwrap() {
        let before = hunk["before"].as_str().unwrap();
        let after = hunk["after"].as_str().unwrap();
        assert_ne!(before, after);
        assert!(
            only_comment_cut(before, after),
            "only comment bytes may leave the body"
        );
        assert_eq!(successor.matches(before).count(), 1);
        successor = successor.replacen(before, after, 1);
    }
    assert_eq!(json!(successor.len()), entry["after"][1]);
    assert_eq!(support::hash(successor.as_bytes()), entry["after"][2]);
    assert_eq!(
        json!(successor.len() as i64 - current.len() as i64),
        entry["deltaBytes"]
    );
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
    manifest["provenance"]["currentGeneratedCommentBytesGenerated"] = record;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_generated_successors_reconstruct_from_the_authenticated_predecessors() {
        let record = record();
        for (body, entry) in super::super::setup_verbatim_generated::current_bodies()
            .iter()
            .zip(record["entries"].as_array().unwrap())
        {
            successor_body(body, entry);
            let first = entry["hunks"][0]["before"].as_str().unwrap();
            for changed in [format!("{body}\n"), body.replacen(first, "", 1)] {
                assert!(std::panic::catch_unwind(|| successor_body(&changed, entry)).is_err());
            }
        }
    }

    #[test]
    fn the_cut_keeps_authored_and_owner_marker_comments() {
        let before = "  // @vize-map: expr -> 1:2\n  // @ts-ignore Inference-only guard.\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  // authored\n";
        let after = "  // @ts-ignore\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  // authored\n";
        assert!(only_comment_cut(before, after));
        for wrong in [
            "  // @vize-map: expr -> 1:2\n  // @ts-ignore\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  // authored\n",
            "  // @ts-ignore\n  // @ts-ignore\n  // authored\n",
            "  // @ts-ignore\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n",
        ] {
            assert!(!only_comment_cut(before, wrong), "{wrong}");
        }
    }
}
