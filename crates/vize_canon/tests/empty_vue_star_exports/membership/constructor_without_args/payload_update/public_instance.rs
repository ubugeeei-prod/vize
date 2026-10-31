//! Actual ten-file public source authority before the immutable C3 projection.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/public-slot-instance-update-11023.json";
const RECORD_SHA: &str = "d71d1c6dc7f050a2aa1d6ee19b37ae9cc7ede393682ab4ce4df0e4083b36c71e";
const C3_RECORD: &str = "malformed-membership/slot-provider-payload-update-11027.json";
const C3_SHA: &str = "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c";
const PATHS: [&str; 10] = [
    "npm/cli/virtual-ts.d.ts",
    "npm/cli/virtual-ts.d.mts",
    "npm/cli/virtual-ts.d.cts",
    "npm/cli/virtual-ts-globals.d.ts",
    "npm/cli/virtual-ts-patterns.d.ts",
    "npm/cli/content-mapper-helpers.d.ts",
    "npm/cli/content-mapper-helpers.d.mts",
    "npm/cli/content-mapper-helpers.d.cts",
    "npm/cli/generated/content-mapper-types.d.ts",
    "npm/cli/virtual-ts-slots.d.ts",
];
const OWNERS: [&str; 4] = [
    "npm/cli/scripts/generate-content-mapper-types.ts",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-resolver.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-resolver.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
];

fn metadata(update: &Value) {
    assert_eq!(update["version"], 1);
    assert_eq!(
        update["kind"],
        "exact-public-slot-module-and-instance-source-inverse"
    );
    assert_eq!(update["issue"], 11023);
    assert_eq!(
        update["sourceRevision"],
        "ed08f2e7990dd0ee0f5d918829acb68ea66425f9"
    );
    assert_eq!(
        update["beforeSourceRevision"],
        "1a65bbd5e95da036bd7e6a0fee7f8b214023a6b0"
    );
    assert_eq!(update["nativeAcceptanceCredit"], false);
    assert_eq!(
        update["physicalFilesRole"],
        "actual-current-ten-file-public-package"
    );
    assert_eq!(
        update["historicalProjectionRole"],
        "three-complete-C3-entry-inputs-only-not-current-physical-files"
    );
    assert_eq!(
        update["logicalPreambleRole"],
        "unchanged-by-this-update-owned-by-existing-C3-adapter"
    );
    assert_eq!(update["frozenC3RecordSha256"], C3_SHA);
    assert!(update.get("logicalPreamble").is_none());
    assert_eq!(
        update["publicExports"],
        json!([
            "$SlotHostSlots",
            "$SlotsFactory",
            "$SlotConstructor",
            "$SlotConstructorWithoutArguments",
            "$SlotConstructorWithArguments",
            "$SlotConstructorArgs",
            "$SlotConstructorInstance",
            "$SlotHostSelector"
        ])
    );
    for (key, expected) in [
        ("files", PATHS.as_slice()),
        ("sourceOwners", OWNERS.as_slice()),
    ] {
        let records = update[key].as_array().unwrap();
        assert_eq!(records.len(), expected.len());
        for (record, path) in records.iter().zip(expected) {
            assert_eq!(record["path"], *path);
        }
    }
    assert_eq!(update["packageManifest"]["path"], "npm/cli/package.json");
    assert_eq!(update["slotModule"]["path"], PATHS[9]);
    assert_eq!(
        update["slotModule"]["noHistoricalPhysicalCounterpart"],
        true
    );
    assert_eq!(
        update["slotModule"]["declarationNames"],
        json!([
            "$SlotHostSlots",
            "__vizeFunctionalSlots",
            "$NativeSlotsFactory",
            "$SlotsFactory",
            "$HasSlotResolver",
            "$DirectSlotConstructor",
            "$SlotConstructor",
            "$SlotConstructorArgumentLists",
            "$SlotConstructorHasArguments",
            "$SlotConstructorWithoutArguments",
            "$SlotConstructorWithArguments",
            "$SlotConstructorArgs",
            "$SlotContractIsUsable",
            "$SlotContractHasKeys",
            "$SlotConstructorFallback",
            "$SlotConstructorInstance",
            "$SlotHostSelector"
        ])
    );
    let entries = update["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["path"], PATHS[index]);
        assert_eq!(entry["currentFileIndex"], json!(index));
        assert_eq!(entry["c3FileIndex"], json!(index));
    }
}

fn decode(bytes: &[u8]) -> Value {
    assert_eq!(support::hash(bytes), RECORD_SHA);
    let update = serde_json::from_slice(bytes).unwrap();
    metadata(&update);
    update
}

fn load() -> Value {
    decode(&fs::read(support::corpus().join(RECORD)).unwrap())
}

fn c3() -> Value {
    let bytes = fs::read(support::corpus().join(C3_RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), C3_SHA);
    serde_json::from_slice(&bytes).unwrap()
}

fn physical() -> Vec<String> {
    PATHS
        .iter()
        .map(|path| fs::read_to_string(support::repository().join(path)).unwrap())
        .collect()
}

fn image(body: &str, record: &Value) {
    assert_eq!(json!(body.len()), record["bytes"]);
    assert_eq!(
        support::hash(body.as_bytes()),
        record["sha256"].as_str().unwrap()
    );
    assert_eq!(body, record["wholeUtf8"].as_str().unwrap());
}

fn authenticate(bodies: &[String], update: &Value) {
    metadata(update);
    assert_eq!(bodies.len(), PATHS.len());
    for (body, record) in bodies.iter().zip(update["files"].as_array().unwrap()) {
        image(body, record);
    }
    for record in update["sourceOwners"]
        .as_array()
        .unwrap()
        .iter()
        .chain(std::iter::once(&update["packageManifest"]))
    {
        let path = support::relative(record["path"].as_str().unwrap());
        image(
            &fs::read_to_string(support::repository().join(path)).unwrap(),
            record,
        );
    }
}

fn previous(path: &str, current: &str, update: &Value, historical: &Value) -> String {
    metadata(update);
    let index = PATHS[..3]
        .iter()
        .position(|candidate| *candidate == path)
        .unwrap();
    let entry = &update["entries"][index];
    let frozen = &historical["files"][index];
    assert_eq!(frozen["path"], path);
    assert_eq!(entry["c3Bytes"], frozen["afterBytes"]);
    assert_eq!(entry["c3Sha256"], frozen["afterSha256"]);
    image(current, &update["files"][index]);
    let before = entry["edit"]["before"].as_str().unwrap();
    let after = entry["edit"]["after"].as_str().unwrap();
    assert!(!before.is_empty());
    assert!(!after.is_empty());
    assert_ne!(before, after);
    assert_eq!(current.matches(after).count(), 1);
    assert_eq!(current.matches(before).count(), 0);
    let result = current.replacen(after, before, 1);
    assert_eq!(json!(result.len()), frozen["afterBytes"]);
    assert_eq!(
        support::hash(result.as_bytes()),
        frozen["afterSha256"].as_str().unwrap()
    );
    assert_eq!(result, frozen["afterWholeUtf8"].as_str().unwrap());
    result
}

/// This result is an old C3 input, never a current physical public entry.
pub(super) fn source_before(path: &str, current: &str) -> String {
    let update = load();
    authenticate(&physical(), &update);
    previous(path, current, &update, &c3())
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let update = load();
    let bodies = physical();
    authenticate(&bodies, &update);
    let historical = c3();
    for (path, body) in PATHS[..3].iter().zip(&bodies) {
        previous(path, body, &update, &historical);
    }
    manifest["provenance"]["currentPublicSlotInstanceUpdate"] = update;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_ten_file_package_reconstructs_all_complete_immutable_c3_entries() {
        let update = load();
        let bodies = physical();
        authenticate(&bodies, &update);
        let historical = c3();
        for (index, path) in PATHS[..3].iter().enumerate() {
            assert_eq!(
                previous(path, &bodies[index], &update, &historical),
                historical["files"][index]["afterWholeUtf8"]
                    .as_str()
                    .unwrap()
            );
        }
        let mut manifest = json!({"provenance": {}});
        retain_provenance(&mut manifest);
        assert_eq!(
            manifest["provenance"]["currentPublicSlotInstanceUpdate"],
            update
        );
        assert_eq!(
            manifest["provenance"]["currentPublicSlotInstanceUpdate"]["files"]
                .as_array()
                .unwrap()
                .len(),
            10
        );
    }

    #[test]
    fn changed_body_reference_order_missing_duplicate_and_extra_bytes_are_refused() {
        let update = load();
        let bodies = physical();
        for (index, body) in bodies.iter().enumerate() {
            let mut altered = bodies.clone();
            altered[index].push('\n');
            assert!(std::panic::catch_unwind(|| authenticate(&altered, &update)).is_err());
            altered[index] = format!("{body}{body}");
            assert!(std::panic::catch_unwind(|| authenticate(&altered, &update)).is_err());
        }
        for changed in [
            bodies[0].replace("./virtual-ts-slots.js", "./foreign-slots.js"),
            bodies[0].replacen("$SlotHostSlots,", "$SlotsFactory,", 1),
            bodies[0].replacen("$SlotHostSlots,", "", 1),
            bodies[0].replace(
                "\"resolution-mode\": \"import\"",
                "\"resolution-mode\": \"require\"",
            ),
        ] {
            assert_ne!(changed, bodies[0]);
            let mut altered = bodies.clone();
            altered[0] = changed;
            assert!(std::panic::catch_unwind(|| authenticate(&altered, &update)).is_err());
        }
        let mut reversed = bodies.clone();
        reversed.swap(0, 9);
        assert!(std::panic::catch_unwind(|| authenticate(&reversed, &update)).is_err());
        assert!(std::panic::catch_unwind(|| authenticate(&bodies[..9], &update)).is_err());
        let mut extra = bodies.clone();
        extra.push("unknown".to_owned());
        assert!(std::panic::catch_unwind(|| authenticate(&extra, &update)).is_err());
    }

    #[test]
    fn record_tamper_unknown_path_and_changed_projection_are_refused() {
        let update = load();
        let bodies = physical();
        let historical = c3();
        assert!(
            std::panic::catch_unwind(|| previous("unknown.d.ts", &bodies[0], &update, &historical))
                .is_err()
        );
        let mut record = update.clone();
        record["unexpected"] = json!(true);
        assert!(
            std::panic::catch_unwind(|| decode(&serde_json::to_vec(&record).unwrap())).is_err()
        );
        for field in [
            "path",
            "c3Bytes",
            "c3Sha256",
            "currentFileIndex",
            "c3FileIndex",
        ] {
            let mut record = update.clone();
            record["entries"][0][field] = json!("unknown");
            assert!(
                std::panic::catch_unwind(|| previous(PATHS[0], &bodies[0], &record, &historical))
                    .is_err()
            );
        }
        for side in ["before", "after"] {
            let mut record = update.clone();
            record["entries"][0]["edit"][side] = json!("unknown");
            assert!(
                std::panic::catch_unwind(|| previous(PATHS[0], &bodies[0], &record, &historical))
                    .is_err()
            );
        }
    }
}
