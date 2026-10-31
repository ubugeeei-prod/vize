//! A finite current postimage layered before the frozen slot-factory inverse.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "malformed-membership/constructor-without-args-update-20261031.json";
const PATHS: [&str; 5] = [
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-resolver.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-resolver.d.ts.txt",
    "npm/cli/virtual-ts.d.ts",
    "npm/cli/virtual-ts.d.mts",
    "npm/cli/virtual-ts.d.cts",
];

pub(super) fn load() -> Value {
    let bytes = fs::read(support::corpus().join(RECORD)).unwrap();
    assert_eq!(
        support::hash(&bytes),
        "d46c4c1fc777809b9a82c3917020ac14f823f4ad894777043c54a2d55c342a02"
    );
    let update: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(update["version"], 1);
    assert_eq!(update["pullRequest"], 11153);
    assert_eq!(update["kind"], "exact-constructor-source-finite-inverse");
    assert_eq!(update["nativeAcceptanceCredit"], false);
    assert_eq!(update["entry"], 82);
    assert_eq!(update["deltaBytes"], 94);
    assert_eq!(
        update["sourceRevision"],
        "edf01c1e7f6c4933a08dd69db64817a33bedb6c1"
    );
    assert_eq!(
        update["productionRevision"],
        "edf01c1e7f6c4933a08dd69db64817a33bedb6c1"
    );
    assert_eq!(
        update["frozenSlotFactoryRecordSha256"],
        "9dc2caf4c5f2c4855ca5070e10f10efddcfafa99d305c6cd601a01246d41fe65"
    );
    let capture = &update["compiledCapture"];
    assert_eq!(
        capture["sourceRevision"],
        "edf01c1e7f6c4933a08dd69db64817a33bedb6c1"
    );
    assert_eq!(capture["genuineExecutionStatus"], 101);
    assert_eq!(capture["snapshotBodyBytes"], update["helper"]["afterBytes"]);
    assert_eq!(
        capture["snapshotBodySha256"],
        update["helper"]["afterSha256"]
    );
    let cone = update["preambleSourceCone"].as_array().unwrap();
    assert_eq!(cone.len(), 19);
    for source in cone {
        let path = support::repository().join(support::relative(source["path"].as_str().unwrap()));
        let body = fs::read(path).unwrap();
        assert_eq!(json!(body.len()), source["bytes"]);
        assert_eq!(support::hash(&body), source["sha256"].as_str().unwrap());
    }
    let declarations = update["declarations"].as_array().unwrap();
    assert_eq!(declarations.len(), PATHS.len());
    for ((record, path), delta) in declarations.iter().zip(PATHS).zip([94, 94, 68, 68, 68]) {
        assert_eq!(record["path"], path);
        assert_eq!(record["deltaBytes"], delta);
        let body = fs::read_to_string(support::repository().join(path)).unwrap();
        previous(&body, record);
    }
    previous(
        vize_canon::virtual_ts::SHARED_PREAMBLE_DTS,
        &update["helper"],
    );
    update
}

/// Exact whole bodies authenticate both sides; only the recorded alias may differ.
pub(super) fn previous(current: &str, record: &Value) -> String {
    assert_eq!(json!(current.len()), record["afterBytes"]);
    assert_eq!(
        support::hash(current.as_bytes()),
        record["afterSha256"].as_str().unwrap()
    );
    assert_eq!(current, record["afterWholeUtf8"].as_str().unwrap());
    let before = record["edit"]["before"].as_str().unwrap();
    let after = record["edit"]["after"].as_str().unwrap();
    assert_ne!(before, after);
    assert_eq!(current.matches(after).count(), 1);
    let historical = current.replacen(after, before, 1);
    assert_eq!(historical, record["beforeWholeUtf8"].as_str().unwrap());
    assert_eq!(json!(historical.len()), record["beforeBytes"]);
    assert_eq!(
        support::hash(historical.as_bytes()),
        record["beforeSha256"].as_str().unwrap()
    );
    assert_eq!(
        record["afterBytes"].as_u64().unwrap() - record["beforeBytes"].as_u64().unwrap(),
        record["deltaBytes"].as_u64().unwrap()
    );
    historical
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value], update: Value) {
    let current = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
    previous(current, &update["helper"]);
    let helper = members.get_mut(82).unwrap();
    assert_eq!(*helper, update["before"]);
    assert_eq!(
        update["after"],
        json!([
            "project/__vize_helpers.d.ts",
            current.len(),
            support::hash(current.as_bytes())
        ])
    );
    *helper = update["after"].clone();
    manifest["provenance"]["currentConstructorWithoutArgumentsUpdate"] = update;
}

#[test]
fn constructor_postimage_rejects_missing_duplicate_unknown_and_extra_bytes() {
    let update = load();
    for record in update["declarations"]
        .as_array()
        .unwrap()
        .iter()
        .chain(std::iter::once(&update["helper"]))
    {
        let current = record["afterWholeUtf8"].as_str().unwrap();
        let before = record["edit"]["before"].as_str().unwrap();
        let after = record["edit"]["after"].as_str().unwrap();
        assert_eq!(
            previous(current, record),
            record["beforeWholeUtf8"].as_str().unwrap()
        );
        for mutated in [
            current.replacen(after, "", 1),
            format!("{current}{after}"),
            current.replacen(after, before, 1),
            current.replacen("$slots: never", "$slots: unknown", 1),
            format!("{current}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| previous(&mutated, record)).is_err());
        }
        let mut changed = record.clone();
        changed["beforeSha256"] = json!("0".repeat(64));
        assert!(std::panic::catch_unwind(|| previous(current, &changed)).is_err());
    }
}
