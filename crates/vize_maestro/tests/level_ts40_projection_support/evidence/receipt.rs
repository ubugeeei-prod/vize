//! Authenticate linear and joined projection successor receipts.

use super::super::normalize::sha256;

pub(super) fn authenticate_packets(
    directory: &str,
    manifest_text: &str,
    expected_files: usize,
    expected_changed: &[&str],
    expected_retained: usize,
) {
    let manifest: serde_json::Value =
        serde_json::from_str(manifest_text).expect("fixed packet manifest");
    assert_eq!(manifest["version"], 1);
    let files = manifest["files"]
        .as_array()
        .expect("complete packet census");
    assert_eq!(files.len(), expected_files);
    let changed: Vec<_> = files
        .iter()
        .filter(|file| file["before"] != file["after"])
        .map(|file| file["path"].as_str().expect("packet relative path"))
        .collect();
    assert_eq!(changed, expected_changed);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/level_ts40_projection_support/fixtures")
        .join(directory);
    let retained = manifest["retainedFiles"]
        .as_array()
        .expect("retained packets");
    assert_eq!(retained.len(), expected_retained);
    for entry in retained {
        let path = entry["path"].as_str().expect("retained relative path");
        let text = std::fs::read_to_string(root.join(path)).expect("complete retained packet");
        assert_eq!(Some(text.len() as u64), entry["bytes"].as_u64(), "{path}");
        assert_eq!(
            Some(sha256(&text).as_str()),
            entry["sha256"].as_str(),
            "{path}"
        );
    }
}

pub(super) fn authenticate_join_receipt(
    receipt_text: &str,
    expected_receipt_sha: &str,
    main_predecessor: &str,
    tsx_predecessor: &str,
    successor: &str,
    expected_snapshot: &str,
    expected_snapshot_sha: &str,
) {
    assert_eq!(sha256(receipt_text), expected_receipt_sha);
    let receipt: serde_json::Value =
        serde_json::from_str(receipt_text).expect("projection join receipt");
    assert_eq!(receipt["version"], 1);
    assert_eq!(receipt["pullRequest"], 11964);
    assert_eq!(
        receipt["sourcePullRequests"],
        serde_json::json!([11919, 11951])
    );
    assert_eq!(receipt["nativeAcceptanceCredit"].as_bool(), Some(false));
    authenticate_record_identity(&receipt["predecessors"]["main"], main_predecessor);
    authenticate_record_identity(&receipt["predecessors"]["tsx"], tsx_predecessor);
    authenticate_record_identity(&receipt["successor"], successor);
    authenticate_join_delta(&receipt["deltaFromMain"], main_predecessor, successor);
    authenticate_join_delta(&receipt["deltaFromTsx"], tsx_predecessor, successor);

    let qualification = &receipt["qualification"];
    assert_eq!(
        qualification["head"].as_str(),
        Some("d5e55331c62d304e8bab2d5fcbe2097064512d0e")
    );
    assert_eq!(qualification["run"].as_u64(), Some(42089460359));
    let artifact = &qualification["artifact"];
    assert_eq!(artifact["id"].as_u64(), Some(13444742138));
    assert_eq!(
        artifact["name"].as_str(),
        Some("component-input-packets-head-d5e55331c62d304e8bab2d5fcbe2097064512d0e-42089460359")
    );
    assert_eq!(artifact["snapshot"].as_str(), Some(expected_snapshot));
    assert_eq!(artifact["sha256"].as_str(), Some(expected_snapshot_sha));
}

fn authenticate_record_identity(identity: &serde_json::Value, record: &str) {
    assert_eq!(identity["bytes"].as_u64(), Some(record.len() as u64));
    assert_eq!(identity["sha256"].as_str(), Some(sha256(record).as_str()));
}

fn authenticate_join_delta(delta: &serde_json::Value, predecessor: &str, successor: &str) {
    for (field, section) in [
        ("canonGeneratedTextBytes", "[canon]\n"),
        ("contentMapperGeneratedTextBytes", "[content-mapper]\n"),
        ("maestroGeneratedTextBytes", "[maestro]\n"),
    ] {
        assert_eq!(
            delta[field].as_i64(),
            Some(
                generated_text_bytes(successor, section)
                    - generated_text_bytes(predecessor, section)
            ),
            "{field} must authenticate the joined projection delta"
        );
    }
    for (field, key) in [
        ("contentMapperMappings", "mappings="),
        ("contentMapperAuthoredHits", "authored-hits="),
    ] {
        assert_eq!(
            delta[field].as_i64(),
            Some(content_mapper_count(successor, key) - content_mapper_count(predecessor, key)),
            "{field} must authenticate the joined projection delta"
        );
    }
}

fn content_mapper_count(record: &str, key: &str) -> i64 {
    record
        .split_once("[content-mapper]\n")
        .expect("content mapper section")
        .1
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .expect("content mapper count")
        .split_once(':')
        .expect("count/hash separator")
        .0
        .parse()
        .expect("content mapper count")
}

pub(super) fn authenticate_successor_receipt(
    receipt_text: &str,
    expected_receipt_sha: &str,
    pull_request: u64,
    predecessor: &str,
    successor: &str,
) {
    assert_eq!(sha256(receipt_text), expected_receipt_sha);
    let receipt: serde_json::Value =
        serde_json::from_str(receipt_text).expect("projection successor receipt");
    assert_eq!(receipt["version"], 1);
    assert_eq!(receipt["pullRequest"].as_u64(), Some(pull_request));
    assert_eq!(
        receipt["predecessor"]["bytes"].as_u64(),
        Some(predecessor.len() as u64)
    );
    let predecessor_sha = sha256(predecessor);
    assert_eq!(
        receipt["predecessor"]["sha256"].as_str(),
        Some(predecessor_sha.as_str())
    );
    assert_eq!(
        receipt["successor"]["bytes"].as_u64(),
        Some(successor.len() as u64)
    );
    let successor_sha = sha256(successor);
    assert_eq!(
        receipt["successor"]["sha256"].as_str(),
        Some(successor_sha.as_str())
    );
    for (field, section) in [
        ("canonGeneratedTextBytes", "[canon]\n"),
        ("contentMapperGeneratedTextBytes", "[content-mapper]\n"),
        ("maestroGeneratedTextBytes", "[maestro]\n"),
    ] {
        if let Some(expected) = receipt["delta"][field].as_i64() {
            assert_eq!(
                expected,
                generated_text_bytes(successor, section)
                    - generated_text_bytes(predecessor, section),
                "{field} must equal the authenticated predecessor-to-successor delta"
            );
        }
    }
    if let Some(expected) = receipt["delta"]["generatedTextBytesPerProjection"].as_i64() {
        for section in ["[canon]\n", "[content-mapper]\n", "[maestro]\n"] {
            assert_eq!(
                expected,
                generated_text_bytes(successor, section)
                    - generated_text_bytes(predecessor, section),
                "generatedTextBytesPerProjection must match {section}"
            );
        }
    }
    for (field, key) in [
        ("contentMapperMappings", "mappings="),
        ("contentMapperAuthoredHits", "authored-hits="),
    ] {
        if let Some(expected) = receipt["delta"][field].as_i64() {
            let count = |record: &str| -> i64 {
                record
                    .split_once("[content-mapper]\n")
                    .expect("content mapper section")
                    .1
                    .lines()
                    .find_map(|line| line.strip_prefix(key))
                    .expect("content mapper count")
                    .split_once(':')
                    .expect("count/hash separator")
                    .0
                    .parse()
                    .expect("content mapper count")
            };
            assert_eq!(count(successor) - count(predecessor), expected, "{field}");
        }
    }
    if receipt["delta"]["onlyContentMapperGeometryChanged"].as_bool() == Some(true) {
        let without_geometry = |record: &str| {
            let mut in_mapper = false;
            record
                .split_inclusive('\n')
                .filter(|line| {
                    if line.starts_with('[') {
                        in_mapper = *line == "[content-mapper]\n";
                    }
                    !(in_mapper
                        && (line.starts_with("mappings=") || line.starts_with("authored-hits=")))
                })
                .collect::<Vec<_>>()
                .join("")
        };
        assert_eq!(
            without_geometry(successor),
            without_geometry(predecessor),
            "only content-mapper geometry may differ from the complete predecessor"
        );
    }
    if receipt["delta"]["canonAndContentMapperUnchanged"].as_bool() == Some(true) {
        for section in ["[canon]\n", "[content-mapper]\n"] {
            assert_eq!(
                generated_text_record(successor, section),
                generated_text_record(predecessor, section),
                "canonAndContentMapperUnchanged must hold for {section}"
            );
        }
    }
}

fn generated_text_record<'a>(record: &'a str, section: &str) -> &'a str {
    let section = record.split_once(section).expect("projection section").1;
    section
        .lines()
        .find_map(|line| line.strip_prefix("text="))
        .expect("generated text record")
}

fn generated_text_bytes(record: &str, section: &str) -> i64 {
    generated_text_record(record, section)
        .split_once(':')
        .expect("generated text byte/hash separator")
        .0
        .parse()
        .expect("generated text byte count")
}
