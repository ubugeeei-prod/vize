use super::support;
use serde_json::{Value, json};
use std::fs;

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let generated = fs::read(
        support::corpus().join("malformed-membership/generated-contract-update-11153.json"),
    )
    .unwrap();
    assert_eq!(
        support::hash(&generated),
        "4ed2c20757f3cbb830771a7235dc2e7696a208bddb0f08a2a18e4b7629ceea88"
    );
    let generated: Value = serde_json::from_slice(&generated).unwrap();
    assert_eq!(generated["version"], 1);
    assert_eq!(generated["pullRequest"], 11153);
    assert_eq!(
        generated["sourceRevision"],
        "8862213fc88ec98ea51a52bbd0809668b9cd0dce"
    );
    assert_eq!(
        generated["testedMerge"],
        "a311a7da3cd1390b19b69b658929ca6d8627b179"
    );
    assert_eq!(generated["observedRun"], 41043029472_i64);
    assert_eq!(generated["observedJob"], 124021180957_i64);
    assert_eq!(generated["artifact"], 12971895560_i64);
    assert_eq!(
        generated["artifactArchiveSha256"],
        "b8e7e11735c2e8b8bd9de42c38371585f8b1c32d22f5750cdb731aa81c44814f"
    );
    assert_eq!(
        generated["extractedPacketSha256"],
        "56ece424e5858328feac89f9ed30a25de7738d3a429d56545966cb327a2b848f"
    );
    let entries = generated["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    for (update, expected_index) in entries.iter().zip([74usize, 75, 82]) {
        let index = update["entry"].as_u64().unwrap() as usize;
        assert_eq!(index, expected_index);
        let member = members.get_mut(index).unwrap();
        assert_eq!(*member, update["before"]);
        assert_eq!(update["before"][0], update["after"][0]);
        assert_eq!(
            update["after"][1].as_i64().unwrap() - update["before"][1].as_i64().unwrap(),
            update["deltaBytes"].as_i64().unwrap()
        );
        *member = update["after"].clone();
    }
    manifest["provenance"]["currentGeneratedContractUpdate"] = generated;

    let resolver =
        fs::read(support::corpus().join("malformed-membership/slot-resolver-update-11153.json"))
            .unwrap();
    assert_eq!(
        support::hash(&resolver),
        "dd97c8e3361b9782d3d63a97e8831d03645e70bf0261406d7c81ef1bdac9374e"
    );
    let resolver: Value = serde_json::from_slice(&resolver).unwrap();
    assert_eq!(resolver["version"], 1);
    assert_eq!(resolver["pullRequest"], 11153);
    assert_eq!(
        resolver["sourceRevision"],
        "ebe4557da5c5598d746fca6a1f5fec4655f9c7f5"
    );
    assert_eq!(
        resolver["testedMerge"],
        "f04d35bba69dcf5b860496b95097b115e6bc7a9b"
    );
    assert_eq!(resolver["observedRun"], 41049205214_i64);
    assert_eq!(resolver["observedJob"], 124041861371_i64);
    assert_eq!(resolver["artifact"], 12974471198_i64);
    assert_eq!(
        resolver["artifactArchiveSha256"],
        "bab980089fe5260acb1356d2a87655743aed8ca21d1fdcdbfba91f058c45059e"
    );
    assert_eq!(
        resolver["extractedPacketSha256"],
        "5ab08fea04db6ef67c50cde5bcf1b347b3ec719efade6b420e00cf2016b7ecfa"
    );
    assert_eq!(resolver["entry"], 82);
    assert_eq!(resolver["deltaBytes"], 547);
    let helper = members.get_mut(82).unwrap();
    assert_eq!(*helper, resolver["before"]);
    assert_eq!(resolver["before"][0], resolver["after"][0]);
    assert_eq!(
        resolver["after"][1].as_u64().unwrap() - resolver["before"][1].as_u64().unwrap(),
        547
    );
    *helper = resolver["after"].clone();
    manifest["provenance"]["currentSlotResolverUpdate"] = resolver;

    let resolver_default = fs::read(
        support::corpus().join("malformed-membership/slot-resolver-default-update-11153.json"),
    )
    .unwrap();
    assert_eq!(
        support::hash(&resolver_default),
        "4b83d67b63eca20a1fe7b65b9e8e657e4f1cdb524de3ede56164a82e6d21c49d"
    );
    let resolver_default: Value = serde_json::from_slice(&resolver_default).unwrap();
    assert_eq!(resolver_default["version"], 1);
    assert_eq!(resolver_default["pullRequest"], 11153);
    assert_eq!(
        resolver_default["sourceRevision"],
        "06180e7d273c72d507dc2590d6eb5518455ea457"
    );
    assert_eq!(
        resolver_default["testedMerge"],
        "1087711502f84c72430d269c0a1f529164f52dd5"
    );
    assert_eq!(resolver_default["observedRun"], 41065960975_i64);
    assert_eq!(resolver_default["observedJob"], 124098349503_i64);
    assert_eq!(resolver_default["artifact"], 12983250250_i64);
    assert_eq!(
        resolver_default["artifactArchiveSha256"],
        "e3d8aa526f2fc0eb975fe20fadb7f1223a674390b673af8600f0f668be8c5766"
    );
    assert_eq!(
        resolver_default["extractedPacketSha256"],
        "1c3c2d1153012c2da6159e089cb89ffeffce0812ec619c343a6cd3b11bf8a7ef"
    );
    assert_eq!(resolver_default["entry"], 82);
    assert_eq!(resolver_default["deltaBytes"], 56);
    let helper = members.get_mut(82).unwrap();
    assert_eq!(*helper, resolver_default["before"]);
    assert_eq!(resolver_default["before"][0], resolver_default["after"][0]);
    assert_eq!(
        resolver_default["after"][1].as_u64().unwrap()
            - resolver_default["before"][1].as_u64().unwrap(),
        56
    );
    *helper = resolver_default["after"].clone();
    manifest["provenance"]["currentSlotResolverDefaultUpdate"] = resolver_default;
    apply_slot_factory_union(manifest, members);
}

fn slot_factory_union() -> Value {
    let bytes = fs::read(
        support::corpus().join("malformed-membership/slot-factory-union-update-20261031.json"),
    )
    .unwrap();
    assert_eq!(
        support::hash(&bytes),
        "9dc2caf4c5f2c4855ca5070e10f10efddcfafa99d305c6cd601a01246d41fe65"
    );
    serde_json::from_slice(&bytes).unwrap()
}

/// Invert only the complete certified lines, then authenticate every other byte.
fn inverse_slot_factory_union(current: &str, record: &Value) -> String {
    assert_eq!(json!(current.len()), record["afterBytes"]);
    assert_eq!(
        support::hash(current.as_bytes()),
        record["afterSha256"].as_str().unwrap()
    );
    assert_eq!(
        record["afterBytes"].as_u64().unwrap() - record["beforeBytes"].as_u64().unwrap(),
        8
    );
    let mut historical = current.to_owned();
    for edit in record["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_eq!(historical.matches(after).count(), 1);
        historical = historical.replacen(after, before, 1);
    }
    assert_eq!(json!(historical.len()), record["beforeBytes"]);
    assert_eq!(
        support::hash(historical.as_bytes()),
        record["beforeSha256"].as_str().unwrap()
    );
    historical
}

fn apply_slot_factory_union(manifest: &mut Value, members: &mut [Value]) {
    let update = slot_factory_union();
    assert_eq!(update["version"], 1);
    assert_eq!(update["pullRequest"], 11153);
    assert_eq!(update["kind"], "exact-source-finite-inverse");
    assert_eq!(update["nativeAcceptanceCredit"], false);
    assert_eq!(update["entry"], 82);
    assert_eq!(update["deltaBytes"], 8);
    let declarations = update["declarations"].as_array().unwrap();
    let paths = [
        "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-slot-resolver.d.ts.txt",
        "crates/vize_canon/src/virtual_ts/helpers/declarations/slot-resolver.d.ts.txt",
        "npm/cli/virtual-ts.d.ts",
        "npm/cli/virtual-ts.d.mts",
        "npm/cli/virtual-ts.d.cts",
    ];
    assert_eq!(declarations.len(), paths.len());
    for (declaration, expected_path) in declarations.iter().zip(paths) {
        assert_eq!(declaration["path"], expected_path);
        let body = fs::read_to_string(support::repository().join(expected_path)).unwrap();
        inverse_slot_factory_union(&body, declaration);
    }
    let current = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
    inverse_slot_factory_union(current, &update["helper"]);
    let helper = members.get_mut(82).unwrap();
    assert_eq!(*helper, update["before"]);
    assert_eq!(update["before"][0], update["after"][0]);
    assert_eq!(
        update["after"],
        json!([
            "project/__vize_helpers.d.ts",
            current.len(),
            support::hash(current.as_bytes())
        ])
    );
    *helper = update["after"].clone();
    manifest["provenance"]["currentSlotFactoryUnionUpdate"] = update;
}

#[test]
fn slot_factory_union_postimage_rejects_unrelated_or_missing_bytes() {
    let update = slot_factory_union();
    let current = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
    inverse_slot_factory_union(current, &update["helper"]);
    let line = update["helper"]["edits"][0]["after"].as_str().unwrap();
    for mutated in [
        current.replace(line, ""),
        format!("{current}{line}"),
        format!("{current}\n"),
    ] {
        assert!(
            std::panic::catch_unwind(|| inverse_slot_factory_union(&mutated, &update["helper"]))
                .is_err()
        );
    }
}
