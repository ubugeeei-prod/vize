//! Authenticate the complete new payload family before reading frozen history.
use super::support;
use serde_json::{Value, json};
use std::fs;

mod ref_factory;

const RECORD: &str = "malformed-membership/slot-provider-payload-update-11027.json";
const PATHS: [&str; 11] = [
    "npm/cli/virtual-ts.d.ts",
    "npm/cli/virtual-ts.d.mts",
    "npm/cli/virtual-ts.d.cts",
    "npm/cli/virtual-ts-globals.d.ts",
    "npm/cli/virtual-ts-patterns.d.ts",
    "npm/cli/content-mapper-helpers.d.ts",
    "npm/cli/content-mapper-helpers.d.mts",
    "npm/cli/content-mapper-helpers.d.cts",
    "npm/cli/generated/content-mapper-types.d.ts",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-payload.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-payload.d.ts.txt",
];

fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(
        support::hash(&bytes),
        "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c"
    );
    let update: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(update["version"], 1);
    assert_eq!(update["kind"], "exact-slot-provider-payload-source-inverse");
    assert_eq!(update["issue"], 11027);
    assert_eq!(
        update["sourceRevision"],
        "35c9246e926966cd98582401cb4f1a9281b07ce3"
    );
    assert_eq!(
        update["beforeSourceRevision"],
        "be8aa8108652c10b7eb02d963e8862fcae6b542c"
    );
    assert_eq!(update["nativeAcceptanceCredit"], false);
    assert_eq!(
        update["physicalFilesRole"],
        "actual-current-nine-file-public-package-and-two-canonical-assets"
    );
    assert_eq!(
        update["logicalPreambleRole"],
        "source-composition-prediction-checked-against-compiled-constant-by-native-tests"
    );
    assert_eq!(
        update["frozenConstructorRecordSha256"],
        "d46c4c1fc777809b9a82c3917020ac14f823f4ad894777043c54a2d55c342a02"
    );
    assert_eq!(
        update["frozenLayoutRecordSha256"],
        "fa2f6b2369276268c1c4be5ccdc68e5053a732166e8ad42198e77a2daccd7cdc"
    );
    let files = update["files"].as_array().unwrap();
    assert_eq!(files.len(), PATHS.len());
    for (index, (file, path)) in files.iter().zip(PATHS).enumerate() {
        assert_eq!(file["path"], path);
        assert_eq!(
            file["edits"].as_array().unwrap().len(),
            usize::from(index < 3 || index > 8)
        );
    }
    assert_eq!(
        update["logicalPreamble"]["path"],
        "vize_canon::virtual_ts::SHARED_PREAMBLE_DTS"
    );
    assert_eq!(update["logicalPreamble"]["beforeBytes"], 43_628);
    assert_eq!(
        update["logicalPreamble"]["beforeSha256"],
        "36165195867281533ce9cb085cc646e3dcac6aafe287e9fbc8f9aa71bf48908e"
    );
    update
}

fn previous(current: &str, record: &Value) -> String {
    assert_eq!(json!(current.len()), record["afterBytes"]);
    assert_eq!(
        support::hash(current.as_bytes()),
        record["afterSha256"].as_str().unwrap()
    );
    assert_eq!(current, record["afterWholeUtf8"].as_str().unwrap());
    let mut previous = current.to_owned();
    for edit in record["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_ne!(before, after);
        assert!(!after.is_empty());
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, before, 1);
    }
    assert_eq!(previous, record["beforeWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(previous.len()), record["beforeBytes"]);
    assert_eq!(
        support::hash(previous.as_bytes()),
        record["beforeSha256"].as_str().unwrap()
    );
    previous
}

/// Only the eleven named files can be projected; every other source-cone
/// member is read unchanged and remains subject to its existing frozen hash.
pub(super) fn source_before(path: &str) -> String {
    let actual = fs::read_to_string(support::repository().join(path)).unwrap();
    let actual = ref_factory::source_before(path, &actual);
    let Some(index) = PATHS.iter().position(|candidate| *candidate == path) else {
        return actual;
    };
    previous(&actual, &load()["files"][index])
}

pub(super) fn preamble_before(current: &str) -> String {
    previous(
        &ref_factory::preamble_before(current),
        &load()["logicalPreamble"],
    )
}

pub(super) fn retain_current(manifest: &mut Value, members: &mut [Value], current: &str) {
    let update = load();
    for path in PATHS {
        source_before(path);
    }
    let logical = &update["logicalPreamble"];
    preamble_before(current);
    let helper = members.get_mut(82).unwrap();
    assert_eq!(
        *helper,
        json!([
            "project/__vize_helpers.d.ts",
            logical["beforeBytes"],
            logical["beforeSha256"]
        ])
    );
    *helper = json!([
        "project/__vize_helpers.d.ts",
        current.len(),
        support::hash(current.as_bytes())
    ]);
    manifest["provenance"]["currentSlotProviderPayloadUpdate"] = update;
    manifest["provenance"]["currentRefFactoryUpdate"] = ref_factory::load();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_current_package_and_compiled_preamble_reconstruct_frozen_inputs() {
        let update = load();
        for (index, path) in PATHS.iter().enumerate() {
            assert_eq!(
                source_before(path),
                update["files"][index]["beforeWholeUtf8"].as_str().unwrap()
            );
        }
        assert_eq!(
            preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS),
            update["logicalPreamble"]["beforeWholeUtf8"]
                .as_str()
                .unwrap()
        );
    }

    #[test]
    fn missing_duplicate_foreign_and_extra_bytes_are_refused() {
        let update = load();
        for record in update["files"]
            .as_array()
            .unwrap()
            .iter()
            .chain(std::iter::once(&update["logicalPreamble"]))
        {
            let current = record["afterWholeUtf8"].as_str().unwrap();
            for changed in [
                format!("{current}\n"),
                format!("{current}{current}"),
                current.replacen("type ", "interface ", 1),
            ] {
                assert_ne!(changed, current);
                assert!(std::panic::catch_unwind(|| previous(&changed, record)).is_err());
            }
            for edit in record["edits"].as_array().unwrap() {
                let after = edit["after"].as_str().unwrap();
                for changed in [
                    current.replacen(after, "", 1),
                    current.replacen(after, edit["before"].as_str().unwrap(), 1),
                ] {
                    assert!(std::panic::catch_unwind(|| previous(&changed, record)).is_err());
                }
            }
        }
    }

    #[test]
    fn whole_body_hashes_and_exact_edit_inventory_are_required() {
        let update = load();
        let record = &update["logicalPreamble"];
        let current = record["afterWholeUtf8"].as_str().unwrap();
        for field in [
            "beforeBytes",
            "afterBytes",
            "beforeSha256",
            "afterSha256",
            "beforeWholeUtf8",
            "afterWholeUtf8",
        ] {
            let mut changed = record.clone();
            changed[field] = json!("unknown");
            assert!(std::panic::catch_unwind(|| previous(current, &changed)).is_err());
        }
        for edits in [
            json!([]),
            json!([record["edits"][0], record["edits"][0]]),
            json!([{ "before": "", "after": "" }]),
        ] {
            let mut changed = record.clone();
            changed["edits"] = edits;
            assert!(std::panic::catch_unwind(|| previous(current, &changed)).is_err());
        }
    }
}
