//! Inert HTTP/plan metadata controls; private Git fixtures never claim live registry execution.
use super::super::archive;
use super::{Repo, github};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn fixture(repo: &Repo, head: &str, cut: &str) -> Value {
    let observed = "2026-10-10T00:00:00Z";
    let mut plan = json!({"head":head,"parentCut":cut,"version":"0.8.0",
        "npm":[{"name":"@vizejs/native","version":"0.8.0"},{"name":"vize","version":"0.8.0"}],
        "crates":[{"name":"vize","version":"0.8.0"}],
        "editor":{"publisher":"vize","name":"vize","version":"0.8.0"},"githubAssets":[],
        "authority":{"commitSha256":"a".repeat(64),"tree":github::git(&["rev-parse",&format!("{head}^{{tree}}")],&repo.work).unwrap(),
            "blobs":[{"path":"Cargo.toml","oid":github::git(&["rev-parse",&format!("{head}:Cargo.toml")],&repo.work).unwrap(),"sha256":"b".repeat(64)}]}});
    let duplicate = plan["authority"]["blobs"][0].clone();
    plan["authority"]["blobs"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let mut rows = Vec::new();
    for (channel, name, url, status, evidence, body) in [
        (
            "npm",
            "@vizejs/native",
            "https://registry.npmjs.org/%40vizejs%2Fnative/0.8.0",
            404,
            "typed-exact-version-missing",
            json!({"error":"version not found: 0.8.0"}),
        ),
        (
            "npm",
            "vize",
            "https://registry.npmjs.org/vize/0.8.0",
            404,
            "typed-exact-version-missing",
            json!({"error":"Not found"}),
        ),
        (
            "crates",
            "vize",
            "https://crates.io/api/v1/crates/vize/0.8.0",
            404,
            "typed-exact-version-missing",
            json!({"errors":[{"detail":"crate `vize` does not have a version `0.8.0`"}]}),
        ),
        (
            "marketplace",
            "vize.vize",
            "https://marketplace.visualstudio.com/_apis/gallery/publishers/vize/extensions/vize?flags=1&api-version=7.2-preview.2",
            200,
            "complete-marketplace-version-inventory",
            json!({"extensionName":"vize","extensionId":"inert-extension-id", "flags":512|65536,
                "publisher":{"publisherName":"vize","publisherId":"inert-publisher-id"},
                "versions":[{"version":"0.7.0"},{"version":"0.7.0","targetPlatform":"linux-x64"}]}),
        ),
        (
            "openvsx",
            "vize.vize",
            "https://open-vsx.org/api/vize/vize/0.8.0",
            404,
            "typed-exact-version-missing",
            json!({"error":"Extension not found: vize.vize 0.8.0"}),
        ),
    ] {
        rows.push(json!({"channel":channel,"name":name,"version":"0.8.0","url":url,"status":status,"evidence":evidence,
            "responseBytes":serde_json::to_vec(&body).unwrap().len(),"response":body,"responseSha256":"c".repeat(64),"date":null,"observedAt":observed}));
    }
    json!({"schema":"vize-retired-unpublished-registry-absence-v1","head":head,"tag":"v0.8.0","observedAt":observed,
        "plan":plan,"observations":rows,"scope":"Inert retained metadata schema controls; no original HTTP byte custody or live absence claim"})
}

pub(super) fn controls(receipt: &Value, logs: &BTreeMap<String, Vec<u8>>, repo: &Repo) {
    let before = github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for (pointer, value) in [
        ("/registryAbsence", Value::Null),
        ("/registryAbsence/observations", json!([])),
        ("/registryAbsence/head", receipt["identity"]["cut"].clone()),
        (
            "/registryAbsence/plan/parentCut",
            receipt["identity"]["head"].clone(),
        ),
        ("/registryAbsence/plan/npm", json!([])),
        ("/registryAbsence/plan/npm/0/version", json!("0.9.0")),
        ("/registryAbsence/plan/authority/blobs", json!([])),
        (
            "/registryAbsence/plan/authority/blobs/1/sha256",
            json!("d".repeat(64)),
        ),
        ("/registryAbsence/observations/0/name", json!("foreign")),
        (
            "/registryAbsence/observations/0/url",
            json!("https://registry.npmjs.org/foreign/0.8.0"),
        ),
        ("/registryAbsence/observations/0/status", json!(403)),
        ("/registryAbsence/observations/0/evidence", json!("assumed")),
        (
            "/registryAbsence/observations/0/responseSha256",
            json!("unknown"),
        ),
        ("/registryAbsence/observations/0/responseBytes", json!(0)),
        ("/registryAbsence/observations/0/date", json!(123)),
        (
            "/registryAbsence/observations/0/response/error",
            json!("version not found: 0.9.0"),
        ),
        (
            "/registryAbsence/observations/3/responseBytes",
            json!(2_097_153),
        ),
        (
            "/registryAbsence/observations/3/response/versions",
            json!([]),
        ),
        (
            "/registryAbsence/observations/3/response/versions/1/version",
            json!("0.7.0-"),
        ),
        (
            "/registryAbsence/observations/3/response/versions/1/version",
            json!("0.8.0"),
        ),
    ] {
        let mut bad = receipt.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            archive::install(bad, logs, &repo.work).is_err(),
            "{pointer}"
        );
        assert_eq!(
            github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    let mut duplicate = receipt.clone();
    duplicate["registryAbsence"]["observations"][1] =
        duplicate["registryAbsence"]["observations"][0].clone();
    assert!(archive::install(duplicate, logs, &repo.work).is_err());
    assert_eq!(
        github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
        before
    );
}
