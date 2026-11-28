//! The lock re-resolution after the builders stopped depending on oxc-parser at
//! runtime; every byte of the qualified lock predecessor stays bound.
use crate::support;
use serde_json::{Value, json};
use std::{fs, sync::OnceLock};

const UPDATE: &str = "malformed-membership/provider-parser-free-builders-update.json";
const UPDATE_HASH: &str = "e530a5d7178e0030acc0066f9fcbbbd3db9c3f9b9ac94bec2c8d02dc217eb9c0";
const KIND: &str = "exact-provider-qualification-parser-free-builders-successor";
const LOCK: &str = "pnpm-lock.yaml";

fn update() -> &'static Value {
    static UPDATE_VALUE: OnceLock<Value> = OnceLock::new();
    UPDATE_VALUE.get_or_init(|| {
        let bytes = fs::read(support::corpus().join(UPDATE)).expect("complete lock successor");
        assert_eq!(support::hash(&bytes), UPDATE_HASH, "{UPDATE}");
        let update: Value = serde_json::from_slice(&bytes).expect("JSON lock successor");
        assert_eq!(update["kind"], KIND);
        assert_eq!(update["nativeAcceptanceCredit"], false);
        update
    })
}

fn bound(binding: &Value, body: &[u8]) -> bool {
    binding["path"] == LOCK
        && binding["bytes"] == json!(body.len())
        && binding["sha256"] == support::hash(body)
}

/// The exact previous lock, when `body` is exactly the re-resolved lock.
pub(crate) fn before_parser_free_builders(path: &str, body: &[u8]) -> Option<Vec<u8>> {
    let update = update();
    if path != LOCK || !bound(&update["finalQualificationLock"], body) {
        return None;
    }
    let mut previous = Vec::with_capacity(body.len());
    let mut cursor = 0;
    for replacement in update["lockReplacements"].as_array()? {
        let before = replacement["before"].as_str()?.as_bytes();
        let after = replacement["after"].as_str()?.as_bytes();
        let offset = usize::try_from(replacement["offset"].as_u64()?).ok()?;
        let end = offset.checked_add(after.len())?;
        if before.is_empty() || after.is_empty() || offset < cursor {
            return None;
        }
        if body.get(offset..end) != Some(after) {
            return None;
        }
        previous.extend_from_slice(&body[cursor..offset]);
        previous.extend_from_slice(before);
        cursor = end;
    }
    previous.extend_from_slice(&body[cursor..]);
    bound(&update["previousQualificationLock"], &previous).then_some(previous)
}

/// The qualification record of the re-resolved lock, given its qualified predecessor.
pub(crate) fn qualification_successor(
    predecessor: &Value,
    previous_lock: &[u8],
    lock: &[u8],
) -> Result<Value, &'static str> {
    let update = update();
    if !bound(&predecessor["finalQualificationLock"], previous_lock)
        || predecessor["finalQualificationLock"] != update["previousQualificationLock"]
    {
        return Err("exact parser-free builders qualification predecessor");
    }
    Ok(json!({
        "version": 1,
        "kind": KIND,
        "nativeAcceptanceCredit": false,
        "previousQualificationLock": predecessor["finalQualificationLock"],
        "finalQualificationLock": { "path": LOCK, "bytes": lock.len(), "sha256": support::hash(lock) },
        "finalQualificationPackage": predecessor["finalQualificationPackage"],
        "workspaceManifest": predecessor["workspaceManifest"],
        "scope": update["scope"],
    }))
}

/// The current lock as the successors before this one recorded it.
#[cfg(test)]
pub(crate) fn current_lock_before_parser_free_builders() -> String {
    let current = fs::read(support::repository().join(LOCK)).unwrap();
    String::from_utf8(before_parser_free_builders(LOCK, &current).unwrap_or(current)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_inverse_reconstructs_the_whole_previous_lock() {
        let current = fs::read(support::repository().join(LOCK)).unwrap();
        let previous = before_parser_free_builders(LOCK, &current).unwrap();
        assert!(bound(&update()["previousQualificationLock"], &previous));
        assert!(before_parser_free_builders(LOCK, &previous).is_none());
        assert!(super::super::before_vite_resolver_importer(LOCK, &previous).is_some());
        assert_eq!(
            super::super::restore(LOCK, String::from_utf8(current.clone()).unwrap()),
            super::super::restore(LOCK, String::from_utf8(previous.clone()).unwrap()),
        );
        assert_eq!(
            super::super::before_translation_worker(LOCK, &current),
            super::super::before_translation_worker(LOCK, &previous),
        );
    }

    #[test]
    fn foreign_and_changed_lock_bytes_cannot_use_the_successor() {
        let current = fs::read_to_string(support::repository().join(LOCK)).unwrap();
        assert!(before_parser_free_builders("foreign-lock.yaml", current.as_bytes()).is_none());
        for changed in [
            current.replacen("pythonia@1.2.6", "pythonia@0.0.0", 1),
            current.replacen("  npm/builder/vite:\n", "  npm/builder/other:\n", 1),
            format!("{current}\n"),
        ] {
            assert_ne!(changed, current);
            assert!(before_parser_free_builders(LOCK, changed.as_bytes()).is_none());
        }
    }
}
