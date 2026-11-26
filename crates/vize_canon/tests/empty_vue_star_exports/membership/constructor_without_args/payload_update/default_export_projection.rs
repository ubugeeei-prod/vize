//! Restore the exact generator before the default export projection changes.

use super::support;
use serde_json::{Value, json};
use std::{fs, path::Path, sync::OnceLock};

const RECORD_SHA: &str = "24d58a6b250d5ac39cde3db348235cff88c6517ae4d155b1b7fefda92888098f";

fn record() -> &'static Value {
    static RECORD: OnceLock<Value> = OnceLock::new();
    RECORD.get_or_init(|| {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/tooling/default-export-projection-13693.json"
        ));
        assert_eq!(support::hash(bytes), RECORD_SHA);
        let record: Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(record["schema"], "vize.defaultExportProjectionSource.v1");
        assert_eq!(record["pullRequest"], 13693);
        assert_eq!(
            record["credit"],
            json!({ "compiled": false, "native": false, "performance": false })
        );
        record
    })
}

fn is_after(current: &str, file: &Value) -> bool {
    json!(current.len()) == file["after"]["bytes"]
        && support::hash(current.as_bytes()) == file["after"]["sha256"]
}

fn restore(current: &str, file: &Value) -> String {
    assert!(is_after(current, file));
    let mut previous = current.to_owned();
    for hunk in file["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(previous.len()), file["before"]["bytes"]);
    assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
    previous
}

/// Only the complete authenticated successor can enter the historical chain.
pub(super) fn source_before(path: &str, current: String) -> String {
    source_before_in(&support::repository(), path, current)
}

fn source_before_in(root: &Path, path: &str, current: String) -> String {
    match record()["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
    {
        Some(file) if is_after(&current, file) => {
            for source in record()["supportingSources"].as_array().unwrap() {
                let path = support::relative(source["path"].as_str().unwrap());
                let path = root.join(path);
                assert!(fs::symlink_metadata(&path).unwrap().is_file());
                let bytes = fs::read(path).unwrap();
                assert_eq!(json!(bytes.len()), source["bytes"]);
                assert_eq!(support::hash(&bytes), source["sha256"]);
            }
            restore(&current, file)
        }
        _ => current,
    }
}

#[cfg(test)]
mod tests {
    use super::{is_after, record, restore, source_before, source_before_in, support};
    use serde_json::json;
    use std::fs;

    #[test]
    fn generator_restores_the_pinned_predecessor_and_round_trips_every_byte() {
        let record = record();
        assert_eq!(
            record["predecessor"],
            json!({
                "revision": "6bf40dbded97b320d607a446d036d77829cf761d",
                "gitBlob": "ef50d92d7dd5d920d4b72d6924ee7dca0a6f5119"
            })
        );
        let files = record["files"].as_array().unwrap();
        assert_eq!(files.len(), 1);
        let file = &files[0];
        let path = "crates/vize_canon/src/virtual_ts/generator.rs";
        assert_eq!(file["path"], path);
        assert_eq!(
            file["before"],
            json!({
                "bytes": 37936,
                "sha256": "0a348064a6fd157a9c4020f670c341da7a6d5a2122dcaf5736189d4621b9ba85"
            })
        );
        let current = fs::read_to_string(support::repository().join(path)).unwrap();
        assert!(is_after(&current, file));
        let previous = source_before(path, current.clone());
        let mut round_trip = previous.clone();
        for hunk in file["hunks"].as_array().unwrap() {
            let before = hunk["before"].as_str().unwrap();
            assert_eq!(round_trip.matches(before).count(), 1);
            round_trip = round_trip.replacen(before, hunk["after"].as_str().unwrap(), 1);
        }
        assert_eq!(round_trip, current);
        assert_eq!(source_before(path, previous.clone()), previous);
        assert_eq!(source_before("other.rs", current.clone()), current);
        let mut extra_byte = current.clone();
        extra_byte.push('\n');
        let first = file["hunks"][0]["after"].as_str().unwrap();
        for changed in [
            current.repeat(2),
            extra_byte,
            current.replacen(first, "", 1),
        ] {
            assert_eq!(source_before(path, changed.clone()), changed);
            assert!(std::panic::catch_unwind(|| restore(&changed, file)).is_err());
        }
    }

    #[test]
    fn actual_source_verifier_rejects_each_changed_missing_and_moved_callee() {
        let record = record();
        let sources = record["supportingSources"].as_array().unwrap();
        let paths = sources
            .iter()
            .map(|source| source["path"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            [
                "crates/vize_canon/src/virtual_ts/generator/irregular_default.rs",
                "crates/vize_canon/src/virtual_ts/generator/module_projection.rs",
                "crates/vize_canon/src/virtual_ts/generator/options_api.rs",
                "crates/vize_canon/src/virtual_ts/generator/options_api/default_export.rs",
                "crates/vize_canon/src/virtual_ts/generator/options_api_props_identifiers.rs",
                "crates/vize_canon/src/virtual_ts/generator/options_api_support.rs",
                "crates/vize_canon/src/virtual_ts/generator/spans.rs",
                "crates/vize_canon/src/virtual_ts/generator/standard_default.rs",
                "crates/vize_canon/src/virtual_ts/generator/template_ref_keys.rs",
                "crates/vize_canon/src/virtual_ts/generator/template_ref_keys/lines.rs",
                "crates/vize_canon/src/virtual_ts/generator/template_ref_keys/lines/tsx.rs",
            ]
        );
        let root = tempfile::tempdir().unwrap();
        for path in &paths {
            let destination = root.path().join(path);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(support::repository().join(path), destination).unwrap();
        }
        let path = "crates/vize_canon/src/virtual_ts/generator.rs";
        let current = fs::read_to_string(support::repository().join(path)).unwrap();
        let expected = source_before(path, current.clone());
        assert_eq!(
            source_before_in(root.path(), path, current.clone()),
            expected
        );
        for source in sources {
            let supporting = source["path"].as_str().unwrap();
            let target = root.path().join(supporting);
            let bytes = fs::read(&target).unwrap();
            let text = std::str::from_utf8(&bytes).unwrap();
            let callee = source["callee"].as_str().unwrap();
            assert_eq!(text.matches(callee).count(), 1);
            let offset = text.find(callee).unwrap() + 3;
            let mut changed = bytes.clone();
            *changed.get_mut(offset).unwrap() ^= 1;
            fs::write(&target, changed).unwrap();
            assert!(
                std::panic::catch_unwind(|| source_before_in(root.path(), path, current.clone()))
                    .is_err()
            );
            fs::remove_file(&target).unwrap();
            assert!(
                std::panic::catch_unwind(|| source_before_in(root.path(), path, current.clone()))
                    .is_err()
            );
            let moved = target.with_extension("rs.moved");
            fs::write(&moved, &bytes).unwrap();
            assert!(
                std::panic::catch_unwind(|| source_before_in(root.path(), path, current.clone()))
                    .is_err()
            );
            fs::rename(moved, target).unwrap();
            assert_eq!(
                source_before_in(root.path(), path, current.clone()),
                expected
            );
        }
    }
}
