//! Bind the offline documentation translator to its exact pnpm importer and package snapshots.
use super::{Checked, Value, bound, fs, json, support};
use crate::package_task_history::before_translation_worker;

pub(super) fn successor(previous: &Value, previous_lock: &[u8], lock: &[u8]) -> Checked<Value> {
    if !bound(
        &previous["finalQualificationLock"],
        "pnpm-lock.yaml",
        previous_lock,
    ) {
        return Err("exact translation worker lock predecessor");
    }
    let path = "docs/package.json";
    let docs = fs::read(support::repository().join(path)).map_err(|_| "complete docs manifest")?;
    let previous_docs =
        before_translation_worker(path, &docs).ok_or("exact translation worker manifest")?;
    Ok(json!({
        "version": 1,
        "kind": "exact-provider-qualification-translation-worker-successor",
        "nativeAcceptanceCredit": false,
        "previousQualificationLock": previous["finalQualificationLock"],
        "finalQualificationLock": { "path": "pnpm-lock.yaml", "bytes": lock.len(), "sha256": support::hash(lock) },
        "finalQualificationPackage": previous["finalQualificationPackage"],
        "workspaceManifest": previous["workspaceManifest"],
        "docsManifest": { "path": path, "bytes": docs.len(), "sha256": support::hash(&docs) },
        "previousDocsManifest": { "path": path, "bytes": previous_docs.len(), "sha256": support::hash(&previous_docs) },
    }))
}

#[test]
fn changing_a_dependency_or_any_other_lock_byte_refuses_the_translation_worker() {
    let path = "pnpm-lock.yaml";
    let lock = fs::read_to_string(support::repository().join(path)).unwrap();
    assert!(before_translation_worker(path, lock.as_bytes()).is_some());
    for changed in [
        lock.replacen("pythonia@1.2.6", "pythonia@0.0.0", 1),
        format!("{lock}\n"),
    ] {
        assert!(before_translation_worker(path, changed.as_bytes()).is_none());
    }
}
