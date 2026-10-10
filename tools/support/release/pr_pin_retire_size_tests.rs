use super::{MAX_BYTES, MAX_LOG_BYTES, check_size};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[test]
fn retirement_archive_and_raw_log_bounds_are_independent_finite_and_overflow_safe() {
    assert_eq!(MAX_BYTES, 64 * 1024 * 1024);
    assert_eq!(MAX_LOG_BYTES, 8 * 1024 * 1024);
    for logs in [0, 1, MAX_LOG_BYTES] {
        let metadata = MAX_BYTES - logs;
        assert!(check_size(metadata - 1, logs).is_ok());
        assert!(check_size(metadata, logs).is_ok());
        let failure = check_size(metadata + 1, logs).unwrap_err();
        assert!(failure.contains("exceeds 64 MiB"));
        assert!(failure.contains(&format!("metadata={} bytes", metadata + 1)));
        assert!(failure.contains(&format!("logs={logs} bytes")));
    }
    assert!(check_size(0, MAX_LOG_BYTES - 1).is_ok());
    assert!(check_size(0, MAX_LOG_BYTES).is_ok());
    assert!(check_size(0, MAX_LOG_BYTES + 1).is_err());
    assert!(check_size(MAX_BYTES, MAX_LOG_BYTES + 1).is_err());
    assert!(check_size(usize::MAX, 1).is_err());
}

#[test]
fn retirement_archive_preserves_complete_accounting_model_above_old_storage_bound() {
    let measured: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/release/retirement-archive-accounting.json"
    ))
    .unwrap();
    let history = measured["historicalReleases"]["compactUtf8Bytes"]
        .as_u64()
        .unwrap() as usize;
    assert_eq!(history, 28_364_208);
    assert!(history > 16 * 1024 * 1024);
    assert_eq!(measured["historicalReleases"]["rows"], 392);
    assert_eq!(measured["historicalReleases"]["assets"], 16_126);
    assert_eq!(measured["originalEvidence"]["jobs"], 189);
    assert_eq!(measured["originalEvidence"]["artifacts"], 117);
    assert_eq!(measured["rawFailureLogs"]["bytes"], 764_449);
    assert_eq!(measured["exactFailedEncodedReceiptBytes"], Value::Null);
    let repo = super::super::tests::Repo::new();
    let (mut receipt, head, pin, integration) = super::super::retire_tests::fixture(&repo);
    // This complete inert extra field models the measured footprint without inventing
    // the failed in-memory receipt or discarding any original private fixture field.
    receipt["completeAccountingModel"] =
        json!({"measurement":measured,"payload":"x".repeat(history)});
    let raw = b"\xef\xbb\xbfcomplete private fixture\x1b[31m failure\x1b[0m\r\n\n".to_vec();
    let logs = BTreeMap::from([("failure/123.log".into(), raw.clone())]);
    let before =
        super::super::super::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work)
            .unwrap();
    let archived = super::install(receipt.clone(), &logs, &repo.work).unwrap();
    let (_, mut saved) = super::read("v0.8.0", &repo.work).unwrap().unwrap();
    let metadata = super::super::metadata::bytes(
        &["show", &format!("{archived}:retirement.json")],
        &repo.work,
    )
    .unwrap();
    assert!(metadata.len() + raw.len() > 16 * 1024 * 1024);
    assert!(metadata.len() + raw.len() <= MAX_BYTES);
    assert_eq!(
        super::super::metadata::bytes(
            &["show", &format!("{archived}:failure/123.log")],
            &repo.work
        )
        .unwrap(),
        raw
    );
    saved.as_object_mut().unwrap().remove("failureLogObjects");
    assert_eq!(
        saved, receipt,
        "all receipt fields, original identities and complete evidence survive storage"
    );
    assert_eq!(
        super::super::super::pr_github::git(
            &["rev-list", "--parents", "-n", "1", &archived],
            &repo.work
        )
        .unwrap(),
        format!("{archived} {pin} {integration}")
    );
    let after = super::super::super::pr_github::git(&["ls-remote", "--refs", "origin"], &repo.work)
        .unwrap();
    assert!(
        before
            .lines()
            .all(|row| after.lines().any(|actual| actual == row))
    );
    assert_eq!(
        super::remote("refs/heads/release/v0.8.0", &repo.work).unwrap(),
        Some(head)
    );
}
