//! Inert provider packets backed by actual private Git objects and immutable archive refs.
use super::super::pr_github as github;
use super::{metadata, retire, retire_archive as archive, retire_log_storage as storage};
use super::{retire_tests::fixture as release_fixture, tests::Repo};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const RAW: &[u8] = b"\xef\xbb\xbf\xff\x00original qualification failure\x1b[31m\r\n\n";

fn refs(repo: &Repo) -> String {
    github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap()
}

fn compressed(receipt: &mut Value, repo: &Repo) -> BTreeMap<String, Vec<u8>> {
    receipt["originalEvidence"]["failureLogEncoding"] = json!("gzip");
    receipt["originalEvidence"]["failureLogSources"] = json!({
        "failure/123.log.gz": {"oid":storage::blob_identity(RAW, &repo.work).unwrap(),"bytes":RAW.len()}
    });
    BTreeMap::from([("failure/123.log.gz".into(), storage::gzip(RAW).unwrap())])
}

fn expected_archive(mut receipt: Value, logs: &BTreeMap<String, Vec<u8>>, repo: &Repo) -> Value {
    receipt["failureLogObjects"] = json!(logs.iter().map(|(path, bytes)| {
        (path.clone(), json!({"oid":storage::blob_identity(bytes, &repo.work).unwrap(),"bytes":bytes.len()}))
    }).collect::<BTreeMap<_, _>>());
    receipt
}

fn message(repo: &Repo, head: &str) -> String {
    let bytes = metadata::bytes(&["cat-file", "commit", head], &repo.work).unwrap();
    String::from_utf8(bytes)
        .unwrap()
        .split_once("\n\n")
        .unwrap()
        .1
        .into()
}

#[test]
fn legacy_archive_receipts_logs_and_commit_messages_remain_byte_exact() {
    for schema in [
        "vize-unpublished-retirement-v1",
        "vize-unpublished-retirement-v2",
    ] {
        let repo = Repo::new();
        let (mut receipt, head, pin, integration) = release_fixture(&repo);
        receipt["schema"] = json!(schema);
        let logs = if schema.ends_with("-v2") {
            compressed(&mut receipt, &repo)
        } else {
            BTreeMap::from([("failure/123.log".into(), RAW.to_vec())])
        };
        let expected = expected_archive(receipt.clone(), &logs, &repo);
        let before = refs(&repo);
        let archived = archive::install(receipt, &logs, &repo.work).unwrap();
        let (actual, saved) = archive::read("v0.8.0", &repo.work).unwrap().unwrap();
        assert_eq!(actual, archived);
        assert_eq!(saved, expected);
        assert_eq!(
            metadata::bytes(
                &["show", &format!("{archived}:retirement.json")],
                &repo.work
            )
            .unwrap(),
            serde_json::to_vec_pretty(&expected).unwrap()
        );
        assert_eq!(
            message(&repo, &archived),
            format!(
                "Vize retired unpublished release v0.8.0\n\nSource-PR: #42\nSource-head: {head}\nRelease-run: 7\nOriginal objects and artifact metadata remain diagnostic evidence only.\n"
            )
        );
        assert_eq!(
            github::git(&["rev-list", "--parents", "-n", "1", &archived], &repo.work).unwrap(),
            format!("{archived} {pin} {integration}")
        );
        for (path, bytes) in &logs {
            assert_eq!(
                metadata::bytes(&["show", &format!("{archived}:{path}")], &repo.work).unwrap(),
                *bytes
            );
        }
        for original in before.lines() {
            assert!(refs(&repo).lines().any(|row| row == original));
        }
    }
}

fn qualification_identity(repo: &Repo) -> Value {
    let (mut receipt, _, _, _) = release_fixture(repo);
    receipt["schema"] = json!("vize-unpublished-retirement-v3");
    let identity = receipt["identity"].as_object_mut().unwrap();
    identity.remove("releaseRun");
    identity.remove("releaseAttempt");
    identity.insert("origin".into(), json!("pre-release-qualification"));
    identity.insert("qualificationRun".into(), json!(8));
    identity.insert("qualificationAttempt".into(), json!(1));
    json!({"schema":"vize-unpublished-retirement-v3","identity":identity})
}

#[test]
fn pre_release_archive_identity_requires_explicit_origin_and_never_accepts_a_release_sentinel() {
    let repo = Repo::new();
    let receipt = qualification_identity(&repo);
    assert!(archive::identity(&receipt).is_ok());
    assert_eq!(archive::log_suffix(&receipt), ".log.gz");
    let before = refs(&repo);
    for (key, value) in [
        ("origin", Value::Null),
        ("origin", json!("release")),
        ("qualificationRun", json!(0)),
        ("qualificationRun", Value::Null),
        ("qualificationAttempt", json!(0)),
        ("qualificationAttempt", json!("1")),
        ("releaseRun", Value::Null),
        ("releaseRun", json!(0)),
        ("releaseRun", json!(7)),
        ("releaseAttempt", Value::Null),
        ("releaseAttempt", json!(0)),
        ("releaseAttempt", json!(1)),
    ] {
        let mut bad = receipt.clone();
        bad["identity"][key] = value;
        assert!(archive::identity(&bad).is_err(), "{key}");
        assert!(
            archive::install(bad, &BTreeMap::new(), &repo.work).is_err(),
            "{key}"
        );
        assert_eq!(refs(&repo), before);
    }
    for key in ["origin", "qualificationRun", "qualificationAttempt"] {
        let mut bad = receipt.clone();
        bad["identity"].as_object_mut().unwrap().remove(key);
        assert!(archive::identity(&bad).is_err(), "missing {key}");
    }
    let mut unknown = receipt;
    unknown["schema"] = json!("unknown-retirement-v4");
    assert!(archive::identity(&unknown).is_err());
    assert_eq!(refs(&repo), before);
}

/// New inert Q/O packets retain the existing fixture's genuine C/H/pin/M Git objects.
/// No R key, R row, invented publication job, or pending qualification survives.
pub(super) fn prequalification_fixture(repo: &Repo) -> (Value, String, String, String) {
    let (mut receipt, head, pin, integration) = release_fixture(repo);
    receipt["schema"] = json!("vize-unpublished-retirement-v3");
    let identity = receipt["identity"].as_object_mut().unwrap();
    identity.remove("releaseRun");
    identity.remove("releaseAttempt");
    identity.insert("origin".into(), json!("pre-release-qualification"));
    identity.insert("qualificationRun".into(), json!(8));
    identity.insert("qualificationAttempt".into(), json!(1));
    let mut qualification = receipt["originalEvidence"]["HRunPages"][0]["workflow_runs"][1].clone();
    qualification["run_started_at"] = json!("2026-10-17T01:00:00Z");
    qualification["status"] = json!("completed");
    qualification["conclusion"] = json!("failure");
    qualification["display_title"] = json!(format!("Check full @ {head}"));
    let names = [
        "test-scripts",
        "test-js-packages",
        "build-js-packages",
        "full-js-report",
        "Full Rust workspace and doctests",
        "clippy-and-test",
        "full-rust-report",
        "cargo-semver-checks (vize_armature)",
        "cargo-semver-checks (vize_atelier_core)",
        "cargo-semver-checks (vize_atelier_dom)",
        "cargo-semver-checks (vize_atelier_sfc)",
        "cargo-semver-checks (vize_atelier_ssr)",
        "cargo-semver-checks (vize_atelier_vapor)",
        "cargo-semver-checks (vize_carton)",
        "cargo-semver-checks (vize_croquis)",
        "cargo-semver-checks (vize_fresco)",
        "cargo-semver-checks (vize_musea)",
        "cargo-semver-checks (vize_relief)",
        "cargo-semver-checks (vize_l1_to_l2)",
    ];
    let jobs: Vec<_> = names.iter().enumerate().map(|(index, name)| {
        json!({"id":if index == 0 { 123 } else { 1001 + index },
            "run_id":8,"run_attempt":1,"head_sha":head,"name":name,
            "status":"completed","conclusion":if index == 0 {"failure"} else {"success"},"steps":[]})
    }).collect();
    let job_pages = json!([{"total_count":jobs.len(),"jobs":jobs}]);
    let mut q_observation = receipt["originalEvidence"]["observations"][0].clone();
    q_observation["run"] = qualification.clone();
    q_observation["jobPages"] = job_pages.clone();
    q_observation["artifactPages"][0]["artifacts"][0]["workflow_run"]["id"] = json!(8);
    let operator_observation = receipt["originalEvidence"]["observations"][2].clone();
    let operator_pages = receipt["guards"]["originalFailureJobPages"]["operator"].clone();
    let guards = receipt["guards"].as_object_mut().unwrap();
    guards.remove("releaseRun");
    guards.remove("publicationJobs");
    guards.insert("qualificationRun".into(), qualification.clone());
    guards.insert("qualificationJobs".into(), json!(jobs));
    guards.insert(
        "originalFailureJobPages".into(),
        json!({"qualification":job_pages,"operator":operator_pages}),
    );
    guards.insert(
        "releaseRunAbsence".into(),
        json!({"pages":[{"total_count":0,"workflow_runs":[]}],
            "hRunPages":[{"total_count":1,"workflow_runs":[qualification]}]}),
    );
    receipt["guards"]["identity"] = receipt["identity"].clone();
    receipt["originalEvidence"]["HRunPages"] =
        json!([{"total_count":1,"workflow_runs":[qualification]}]);
    receipt["originalEvidence"]["observations"] = json!([q_observation, operator_observation]);
    // rust-script runs Cargo tests from its cache project, not the tooling checkout.
    let source = std::path::Path::new(file!()).canonicalize().unwrap();
    let tooling = source
        .ancestors()
        .find(|directory| {
            directory
                .join("tools/support/release/retirement_qualification.ts")
                .is_file()
        })
        .expect("actual retirement tooling checkout must own the Node policy");
    for file in [
        "tools/support/release/retirement_qualification.ts",
        "tools/support/compat/github/release-preflight-bootstrap.mjs",
        "tools/support/compat/github/release-preflight-evidence.mjs",
        "tools/support/compat/github/real-project-matrix-plan.ts",
    ] {
        let target = repo.work.join(file);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(tooling.join(file), target).unwrap();
    }
    (receipt, head, pin, integration)
}

#[test]
fn pre_release_archive_roundtrips_whole_q_custody_and_actual_binary_failure_bytes() {
    let repo = Repo::new();
    let (mut receipt, head, pin, integration) = prequalification_fixture(&repo);
    let logs = compressed(&mut receipt, &repo);
    let expected = expected_archive(receipt.clone(), &logs, &repo);
    let before = refs(&repo);
    let archived = archive::install(receipt.clone(), &logs, &repo.work).unwrap();
    let (actual, saved) = archive::read("v0.8.0", &repo.work).unwrap().unwrap();
    assert_eq!(actual, archived);
    assert_eq!(saved, expected);
    assert_eq!(
        metadata::bytes(
            &["show", &format!("{archived}:retirement.json")],
            &repo.work
        )
        .unwrap(),
        serde_json::to_vec_pretty(&expected).unwrap()
    );
    for key in ["releaseRun", "releaseAttempt"] {
        assert!(saved["identity"].get(key).is_none());
    }
    assert!(saved["guards"].get("releaseRun").is_none());
    assert!(saved["guards"].get("publicationJobs").is_none());
    assert_eq!(saved["identity"]["qualificationRun"], 8);
    assert_eq!(
        saved["guards"]["qualificationJobs"]
            .as_array()
            .unwrap()
            .len(),
        19
    );
    assert_eq!(
        message(&repo, &archived),
        format!(
            "Vize retired unpublished pre-release qualification v0.8.0\n\nSource-PR: #42\nSource-head: {head}\nQualification-run: 8\nOriginal objects and artifact metadata remain diagnostic evidence only.\n"
        )
    );
    assert_eq!(
        github::git(&["rev-list", "--parents", "-n", "1", &archived], &repo.work).unwrap(),
        format!("{archived} {pin} {integration}")
    );
    let stored = metadata::bytes(
        &["show", &format!("{archived}:failure/123.log.gz")],
        &repo.work,
    )
    .unwrap();
    assert_eq!(stored, logs["failure/123.log.gz"]);
    assert_eq!(storage::decode(&stored, RAW.len()).unwrap(), RAW);
    assert_eq!(
        storage::blob_identity(RAW, &repo.work).unwrap(),
        saved["originalEvidence"]["failureLogSources"]["failure/123.log.gz"]["oid"]
    );
    assert_eq!(
        archive::install(receipt, &logs, &repo.work).unwrap(),
        archived
    );
    assert_eq!(github::tag_target("v0.8.0", &repo.work).unwrap(), None);
    assert!(retire::reject_resume("v0.8.0", &repo.work).is_err());
    let after = refs(&repo);
    assert_eq!(
        after
            .lines()
            .filter(|row| !row.ends_with(&archive::reference("v0.8.0")))
            .collect::<Vec<_>>(),
        before.lines().collect::<Vec<_>>()
    );
    assert_eq!(
        archive::remote(&archive::reference("v0.8.0"), &repo.work).unwrap(),
        Some(archived)
    );
}

#[test]
fn pre_release_archive_refuses_schema_confusion_and_corrupt_q_logs_before_remote_mutation() {
    let repo = Repo::new();
    let (mut receipt, _, _, _) = prequalification_fixture(&repo);
    let logs = compressed(&mut receipt, &repo);
    let before = refs(&repo);
    for (pointer, value) in [
        ("/schema", json!("vize-unpublished-retirement-v2")),
        ("/identity/origin", json!("release")),
        ("/identity/qualificationAttempt", json!(0)),
        ("/originalEvidence/failureLogEncoding", json!("raw")),
        (
            "/originalEvidence/failureLogSources/failure~1123.log.gz/bytes",
            json!(RAW.len() - 1),
        ),
        ("/originalEvidence/observations", json!([])),
    ] {
        let mut bad = receipt.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            archive::install(bad, &logs, &repo.work).is_err(),
            "{pointer}"
        );
        assert_eq!(refs(&repo), before);
    }
    for bad_logs in [
        BTreeMap::new(),
        BTreeMap::from([("failure/123.log".into(), RAW.to_vec())]),
        BTreeMap::from([(
            "failure/123.log.gz".into(),
            logs["failure/123.log.gz"][..logs["failure/123.log.gz"].len() - 1].to_vec(),
        )]),
        BTreeMap::from([(
            "failure/123.log.gz".into(),
            [
                logs["failure/123.log.gz"].clone(),
                b"trailing bytes".to_vec(),
            ]
            .concat(),
        )]),
    ] {
        assert!(archive::install(receipt.clone(), &bad_logs, &repo.work).is_err());
        assert_eq!(refs(&repo), before);
    }
}
