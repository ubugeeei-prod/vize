//! A finite provider-path successor; frozen roster and helper history stay intact.

use super::{Checked, Roots, support};
use serde_json::{Value, json};
use std::{fs, path::Path};

mod qualification_graph;

fn load() -> Value {
    let bytes =
        fs::read(support::corpus().join("malformed-membership/provider-peer-update-20261031.json"))
            .unwrap();
    assert_eq!(
        support::hash(&bytes),
        "ad428607e450f807b15e9301d429fcbee7fe67729b52130f2d7e63c2f0cf1e0f"
    );
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["version"], 1);
    assert_eq!(record["kind"], "exact-provider-roster-path-successor");
    assert_eq!(record["nativeAcceptanceCredit"], false);
    record
}

fn roster_hash(members: &[Value]) -> String {
    support::hash(&serde_json::to_vec(members).unwrap())
}

fn successor(members: &[Value], record: &Value) -> Checked<Vec<Value>> {
    if members.len() != 86 || record["wholeBeforeRosterSha256"] != roster_hash(members) {
        return Err("complete frozen predecessor roster");
    }
    let changes = record["changes"].as_array().ok_or("six literal changes")?;
    if changes.len() != 6 {
        return Err("six literal changes");
    }
    let mut current = members.to_vec();
    for (entry, change) in (76..82).zip(changes) {
        if change["entry"] != entry || change["before"] != members[entry] {
            return Err("exact ordered predecessor entry");
        }
        let before = change["before"].as_array().ok_or("whole triplets")?;
        let after = change["after"].as_array().ok_or("whole triplets")?;
        if before.len() != 3 || after.len() != 3 {
            return Err("whole triplets");
        }
        if before[0] == after[0] || before[1..] != after[1..] {
            return Err("only the declared path may change");
        }
        current[entry] = change["after"].clone();
    }
    if record["wholeAfterRosterSha256"] != roster_hash(&current) {
        return Err("complete frozen successor roster");
    }
    Ok(current)
}

fn final_graph(record: &Value, lock: &[u8], package: &[u8]) -> Checked<()> {
    for (key, path, body) in [
        ("finalQualificationLock", "pnpm-lock.yaml", lock),
        ("finalQualificationPackage", "package.json", package),
    ] {
        let binding = &record[key];
        if !binding.is_object() {
            return Err("final qualification graph pending");
        }
        if binding["path"] != path
            || binding["bytes"] != json!(body.len())
            || binding["sha256"] != support::hash(body)
        {
            return Err("exact final qualification graph bytes");
        }
    }
    Ok(())
}

pub(super) fn apply(manifest: &mut Value, members: &mut Vec<Value>) {
    let record = load();
    let lock = fs::read(support::repository().join("pnpm-lock.yaml")).unwrap();
    let package = fs::read(support::repository().join("package.json")).unwrap();
    let qualification = qualification_graph::authorize(&record, &lock, &package)
        .expect("the captured graph is never a substitute for the reviewed final graph");
    *members = successor(members, &record).expect("only the six complete declared paths relocate");
    manifest["provenance"]["currentProviderRoster"] = record;
    if let Some(qualification) = qualification {
        manifest["provenance"]["currentProviderQualificationGraph"] = qualification;
    }
}

pub(super) fn validate(packet: &Value, roots: &Roots, record: &Value) {
    let actual = packet["genuineQualifiedProviders"].as_array().unwrap();
    let expected = record["providerRoles"].as_array().unwrap();
    assert_eq!(actual.len(), 5);
    assert_eq!(expected.len(), 5);
    for (provider, binding) in actual.iter().zip(expected) {
        assert_eq!(provider["role"], binding["role"]);
        let manifest = &provider["wholeManifestRead"];
        let text = manifest["wholeUtf8"].as_str().unwrap();
        assert_eq!(manifest["bytes"], json!(text.len()));
        assert_eq!(manifest["sha256"], support::hash(text.as_bytes()));
        assert_eq!(manifest["bytes"], binding["manifest"]["bytes"]);
        assert_eq!(manifest["sha256"], binding["manifest"]["sha256"]);
        let body: Value = serde_json::from_str(text).unwrap();
        assert_eq!(body["name"], binding["manifest"]["name"]);
        assert_eq!(body["version"], binding["manifest"]["version"]);
        let actual_path = Path::new(manifest["path"].as_str().unwrap())
            .canonicalize()
            .unwrap();
        let expected_path = roots[2].1.join(support::relative(
            binding["manifestRelativePath"].as_str().unwrap(),
        ));
        assert_eq!(actual_path, expected_path);
        let source = &provider["actual"];
        let source_text = source["wholeUtf8"].as_str().unwrap();
        assert_eq!(source["bytes"], json!(source_text.len()));
        assert_eq!(source["sha256"], support::hash(source_text.as_bytes()));
        assert_eq!(source["bytes"], binding["role"]["body"]["bytes"]);
        assert_eq!(source["sha256"], binding["role"]["body"]["sha256"]);
        let source_path = Path::new(source["path"].as_str().unwrap())
            .canonicalize()
            .unwrap();
        let expected_source = actual_path
            .parent()
            .unwrap()
            .join(support::relative(
                binding["role"]["relativePath"].as_str().unwrap(),
            ))
            .canonicalize()
            .unwrap();
        assert_eq!(source_path, expected_source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frozen() -> Vec<Value> {
        let mut first: Vec<Value> =
            serde_json::from_value(support::value("malformed-membership/part-00.json")).unwrap();
        let second: Vec<Value> =
            serde_json::from_value(support::value("malformed-membership/part-01.json")).unwrap();
        first.extend(second);
        first
    }

    #[test]
    fn exact_six_path_successor_keeps_every_other_frozen_triplet() {
        let record = load();
        let before = frozen();
        let after = successor(&before, &record).unwrap();
        let mut inverse = after.clone();
        for change in record["changes"].as_array().unwrap() {
            let entry = change["entry"].as_u64().unwrap() as usize;
            assert_eq!(after[entry], change["after"]);
            inverse[entry] = change["before"].clone();
        }
        assert_eq!(inverse, before);
        assert_eq!(json!(roster_hash(&after)), record["wholeAfterRosterSha256"]);
    }

    #[test]
    fn incomplete_reordered_foreign_or_changed_rosters_are_refused() {
        let record = load();
        let before = frozen();
        let mut drop = before.clone();
        drop.pop();
        let mut add = before.clone();
        add.push(before[0].clone());
        let mut duplicate = before.clone();
        duplicate[1] = duplicate[0].clone();
        let mut reorder = before.clone();
        reorder.swap(76, 77);
        let mut foreign = before.clone();
        foreign[76][0] = json!("providers/unreviewed-peer/hmrPayload.d.ts");
        let mut length = before.clone();
        length[76][1] = json!(1399);
        let mut body = before.clone();
        body[76][2] = json!(support::hash(b"changed"));
        for changed in [drop, add, duplicate, reorder, foreign, length, body] {
            assert_eq!(
                successor(&changed, &record),
                Err("complete frozen predecessor roster")
            );
        }
        let mut unknown = record.clone();
        unknown["changes"][0]["after"][0] = json!("providers/unreviewed-peer/hmrPayload.d.ts");
        assert_eq!(
            successor(&before, &unknown),
            Err("complete frozen successor roster")
        );
        let mut changed_body = record.clone();
        changed_body["changes"][0]["after"][2] = json!(support::hash(b"changed"));
        assert_eq!(
            successor(&before, &changed_body),
            Err("only the declared path may change")
        );
    }

    #[test]
    fn a_captured_graph_cannot_authorize_an_unbound_final_graph() {
        let mut record = load();
        record["finalQualificationLock"] = Value::Null;
        record["finalQualificationPackage"] = Value::Null;
        let lock = b"model final lock";
        let package = b"model final package";
        assert_eq!(
            final_graph(&record, lock, package),
            Err("final qualification graph pending")
        );
        for (key, path, body) in [
            ("finalQualificationLock", "pnpm-lock.yaml", lock.as_slice()),
            (
                "finalQualificationPackage",
                "package.json",
                package.as_slice(),
            ),
        ] {
            record[key] = json!({"path":path,"bytes":body.len(),"sha256":support::hash(body)});
        }
        assert_eq!(final_graph(&record, lock, package), Ok(()));
        assert_eq!(
            final_graph(&record, b"model other lock", package),
            Err("exact final qualification graph bytes")
        );
    }
}
