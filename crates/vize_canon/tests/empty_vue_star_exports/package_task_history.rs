//! Reconstruct the exact manifest bytes used by the frozen provider and SDK records.
use crate::support;
use serde::Deserialize;
use std::{collections::BTreeMap, fs, io, sync::OnceLock};

mod vite_resolver_importer;
pub(crate) use vite_resolver_importer::before_vite_resolver_importer;

#[derive(Deserialize)]
struct PackageRevision {
    current_sha256: String,
    previous_sha256: String,
    edits: Vec<Edit>,
}

#[derive(Deserialize)]
struct Edit {
    current: String,
    previous: String,
}

pub(crate) fn read(path: &str) -> io::Result<String> {
    let current = fs::read_to_string(support::repository().join(path))?;
    Ok(restore(path, current))
}

fn revisions() -> &'static BTreeMap<String, Vec<PackageRevision>> {
    static REVISIONS: OnceLock<BTreeMap<String, Vec<PackageRevision>>> = OnceLock::new();
    REVISIONS.get_or_init(|| {
        serde_json::from_value(support::value(
            "malformed-membership/named-package-task-history.json",
        ))
        .unwrap()
    })
}

pub(crate) fn restore(path: &str, current: String) -> String {
    let current = before_vite_resolver_importer(path, current.as_bytes())
        .map(|body| String::from_utf8(body).expect("UTF-8 resolver importer predecessor"))
        .unwrap_or(current);
    revisions()[path]
        .iter()
        .fold(current, |body, revision| reverse(path, body, revision))
}

/// Only the current, byte-exact dependency graph has the translation-worker predecessor.
pub(crate) fn before_translation_worker(path: &str, body: &[u8]) -> Option<Vec<u8>> {
    let predecessor = before_vite_resolver_importer(path, body);
    let body = predecessor.as_deref().unwrap_or(body);
    let revision = revisions().get(path)?.first()?;
    if support::hash(body) != revision.current_sha256 {
        return None;
    }
    let text = std::str::from_utf8(body).ok()?.to_owned();
    Some(reverse(path, text, revision).into_bytes())
}

fn reverse(path: &str, mut current: String, revision: &PackageRevision) -> String {
    assert_eq!(
        support::hash(current.as_bytes()),
        revision.current_sha256,
        "{path}"
    );
    for edit in &revision.edits {
        assert_eq!(current.matches(&edit.current).count(), 1, "{path}");
        current = current.replacen(&edit.current, &edit.previous, 1);
    }
    assert_eq!(
        support::hash(current.as_bytes()),
        revision.previous_sha256,
        "{path}"
    );
    current
}

#[test]
fn altered_launchers_and_unrelated_bytes_cannot_reconstruct_a_frozen_manifest() {
    let path = "npm/cli/package.json";
    let current = fs::read_to_string(support::repository().join(path)).unwrap();
    for changed in [
        current.replace("package-tasks.cjs build", "package-tasks.cjs test"),
        format!("{current}\n"),
    ] {
        assert!(std::panic::catch_unwind(|| restore(path, changed)).is_err());
    }
}
