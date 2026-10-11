use super::{archive, blob_identity, decode, gzip};
use crate::pr_pin::{metadata, retire_tests::fixture, tests::Repo};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn compressed_fixture(repo: &Repo, raw: &[u8]) -> (Value, BTreeMap<String, Vec<u8>>) {
    let (mut receipt, _, _, _) = fixture(repo);
    receipt["schema"] = json!("vize-unpublished-retirement-v2");
    receipt["originalEvidence"]["failureLogEncoding"] = json!("gzip");
    receipt["originalEvidence"]["failureLogSources"] = json!({
        "failure/123.log.gz": {"oid":blob_identity(raw, &repo.work).unwrap(),"bytes":raw.len()}
    });
    let logs = BTreeMap::from([("failure/123.log.gz".into(), gzip(raw).unwrap())]);
    (receipt, logs)
}

#[test]
fn complete_binary_log_roundtrips_and_corruption_or_extra_members_are_refused() {
    let raw = b"\xef\xbb\xbf\xff\x002026-10-11T15:04:40Z \x1b[31mfailed\x1b[0m\r\n\n";
    let stored = gzip(raw).unwrap();
    assert_eq!(decode(&stored, raw.len()).unwrap(), raw);
    assert_eq!(gzip(raw).unwrap(), stored);
    for expected in [
        0,
        raw.len() - 1,
        raw.len() + 1,
        archive::MAX_BYTES + 1,
        usize::MAX,
    ] {
        assert!(
            decode(&stored, expected).is_err(),
            "decoded length {expected}"
        );
    }
    for bytes in [
        Vec::new(),
        stored[..stored.len() - 1].to_vec(),
        {
            let mut corrupt = stored.clone();
            let crc = corrupt.len() - 8;
            corrupt[crc] ^= 1;
            corrupt
        },
        {
            let mut trailing = stored.clone();
            trailing.extend_from_slice(b"trailing whole private packet");
            trailing
        },
        [stored.clone(), stored.clone()].concat(),
    ] {
        assert!(decode(&bytes, raw.len()).is_err());
    }
    assert!(gzip(&[]).is_err());
}

#[test]
fn compressed_archive_retains_complete_original_bytes_above_raw_log_storage_bound() {
    let repo = Repo::new();
    let original = b"\xef\xbb\xbfprivate original failure\x1b[31m\r\n\n\xff\x00";
    let measured: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/retirement-log-compression-accounting.json"
    ))
    .unwrap();
    let jobs = measured["failedJobs"].as_array().unwrap();
    assert_eq!(jobs.len(), 26);
    assert_eq!(
        jobs.iter()
            .map(|job| job["bytes"].as_u64().unwrap())
            .sum::<u64>(),
        10_119_127
    );
    assert_eq!(
        jobs.iter()
            .map(|job| job["gzipBytes"].as_u64().unwrap())
            .sum::<u64>(),
        1_727_835
    );
    assert!(jobs.iter().all(|job| job["wholeRoundtrip"] == true));
    // An explicit storage accounting model, not a replacement for a public failure.
    // Original packet boundaries, binary bytes and terminal formatting are retained.
    let raw = original
        .iter()
        .copied()
        .cycle()
        .take(measured["rawBytes"].as_u64().unwrap() as usize)
        .collect::<Vec<_>>();
    assert!(raw.len() > archive::MAX_LOG_BYTES);
    let (mut receipt, logs) = compressed_fixture(&repo, &raw);
    receipt["completeAccountingModel"] = json!({"payload":"x".repeat(28_364_208)});
    let head = receipt["identity"]["head"].as_str().unwrap().to_owned();
    let pin = receipt["identity"]["pin"].as_str().unwrap().to_owned();
    let integration = receipt["identity"]["integrationHead"]
        .as_str()
        .unwrap()
        .to_owned();
    let original_refs =
        crate::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    let archived = archive::install(receipt.clone(), &logs, &repo.work).unwrap();
    let (_, mut saved) = archive::read("v0.8.0", &repo.work).unwrap().unwrap();
    let stored = metadata::bytes(
        &["show", &format!("{archived}:failure/123.log.gz")],
        &repo.work,
    )
    .unwrap();
    assert_eq!(stored, logs["failure/123.log.gz"]);
    assert_eq!(decode(&stored, raw.len()).unwrap(), raw);
    assert_eq!(
        blob_identity(&raw, &repo.work).unwrap(),
        saved["originalEvidence"]["failureLogSources"]["failure/123.log.gz"]["oid"]
    );
    saved.as_object_mut().unwrap().remove("failureLogObjects");
    assert_eq!(saved, receipt, "whole receipt and raw custody remain exact");
    assert_eq!(
        crate::pr_github::git(&["rev-list", "--parents", "-n", "1", &archived], &repo.work)
            .unwrap(),
        format!("{archived} {pin} {integration}")
    );
    let after = crate::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    assert!(
        original_refs
            .lines()
            .all(|row| after.lines().any(|actual| actual == row))
    );
    assert_eq!(
        archive::remote("refs/heads/release/v0.8.0", &repo.work).unwrap(),
        Some(head)
    );
}

#[test]
fn compressed_archive_rejects_forged_raw_custody_and_unsafe_paths_before_remote_mutation() {
    let repo = Repo::new();
    let raw = b"whole original failed build\r\n\n";
    let (receipt, logs) = compressed_fixture(&repo, raw);
    let before = crate::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap();
    for (pointer, value) in [
        ("/schema", json!("unknown-retirement-v3")),
        ("/originalEvidence/failureLogEncoding", json!("raw")),
        ("/originalEvidence/failureLogSources", json!({})),
        (
            "/originalEvidence/failureLogSources/failure~1123.log.gz/oid",
            json!("0".repeat(40)),
        ),
        (
            "/originalEvidence/failureLogSources/failure~1123.log.gz/bytes",
            json!(raw.len() - 1),
        ),
        (
            "/originalEvidence/failureLogSources/failure~1123.log.gz/bytes",
            json!(0),
        ),
        (
            "/originalEvidence/failureLogSources/failure~1123.log.gz/bytes",
            json!(archive::MAX_BYTES + 1),
        ),
    ] {
        let mut bad = receipt.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            archive::install(bad, &logs, &repo.work).is_err(),
            "{pointer}"
        );
        assert_eq!(
            crate::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
            before
        );
    }
    for name in [
        "failure/../escape.log.gz",
        "failure/123.log",
        "failure/123.log.gz.extra",
    ] {
        let bad = BTreeMap::from([(name.into(), logs["failure/123.log.gz"].clone())]);
        assert!(
            archive::install(receipt.clone(), &bad, &repo.work).is_err(),
            "{name}"
        );
    }
    let mut appended = logs["failure/123.log.gz"].clone();
    appended.extend_from_slice(b"unarchived trailing bytes");
    assert!(
        archive::install(
            receipt.clone(),
            &BTreeMap::from([("failure/123.log.gz".into(), appended)]),
            &repo.work
        )
        .is_err()
    );
    let mut decoded_overflow = receipt;
    decoded_overflow["completeAccountingModel"] = json!({
        "payload":"x".repeat(archive::MAX_BYTES - raw.len())
    });
    assert!(archive::install(decoded_overflow, &logs, &repo.work).is_err());
    assert_eq!(
        crate::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work).unwrap(),
        before
    );
}
