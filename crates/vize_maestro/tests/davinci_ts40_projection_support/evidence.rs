//! Authenticate the complete packets behind the reviewed #10933 transition.

use super::{capture_fixture, load_matrix, normalize::sha256};

#[test]
fn public_instance_slot_packets_remain_authenticated() {
    authenticate_packets(
        "public-instance-options-api-slots",
        include_str!("fixtures/public-instance-options-api-slots/manifest.json"),
        194,
        &[
            "options-api-slots/maestro/authored-hits.txt",
            "options-api-slots/maestro/mappings.txt",
            "options-api-slots/maestro/text.txt",
            "options-api-slots/maestro-document-0/text.ts",
        ],
        14,
    );
    let historical_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11064/receipt.json");
    let predecessor =
        include_str!("fixtures/public-instance-options-api-slots/after/projection-record.txt");
    let historical_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11064/projection-record.txt"
    );
    authenticate_successor_receipt(
        historical_receipt,
        "e8916bf8eff167bc482fa64de01be67f384284d400ef69e1f8d854694108127c",
        11064,
        predecessor,
        historical_successor,
    );
    let previous_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11069/receipt.json");
    let previous_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11069/projection-record.txt"
    );
    authenticate_successor_receipt(
        previous_receipt,
        "afc2f211f4b841b1385f09924a3593dadad1fb280b694d233250639a7020be7f",
        11069,
        historical_successor,
        previous_successor,
    );
    let current_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11142/receipt.json");
    let current_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11142/projection-record.txt"
    );
    authenticate_successor_receipt(
        current_receipt,
        "d3eae1337abcc6904773256c46069a9745ffc2e533a79b2c8ad186d433e412e7",
        11142,
        previous_successor,
        current_successor,
    );
    let latest_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11142-2/receipt.json");
    let latest_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11142-2/projection-record.txt"
    );
    authenticate_successor_receipt(
        latest_receipt,
        "682ce848022ce787fbc10c787fef3d23e3f1ee384ce185c50e4889e7488910b1",
        11142,
        current_successor,
        latest_successor,
    );
    let exact_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11142-3/receipt.json");
    let exact_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11142-3/projection-record.txt"
    );
    authenticate_successor_receipt(
        exact_receipt,
        "62f509940ffb70ccb90868f53c74a3a8e69fe4452d4b8072967e4b1bb5c2c51d",
        11142,
        latest_successor,
        exact_successor,
    );
    let external_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11148/receipt.json");
    let external_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11148/projection-record.txt"
    );
    authenticate_successor_receipt(
        external_receipt,
        "e2a6dd88ae954c9ff77bb969fa8ebc4ce562850972a2a29b10b1c42d943e7555",
        11148,
        exact_successor,
        external_successor,
    );
    let collision_safe_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11153/receipt.json");
    let collision_safe_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11153/projection-record.txt"
    );
    authenticate_successor_receipt(
        collision_safe_receipt,
        "cd58fde4a43fbb181c31f21f2b2c56d80dcd504b943e2370350b1db5894474f0",
        11153,
        external_successor,
        collision_safe_successor,
    );
    let matrix = load_matrix();
    let fixture = matrix
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "options-api-slots")
        .expect("reviewed Options API slot fixture");
    assert_eq!(
        capture_fixture(fixture).render().as_str(),
        collision_safe_successor
    );
}

#[test]
fn public_instance_mixed_vue2_packets_remain_authenticated() {
    authenticate_packets(
        "public-instance-mixed-vue2",
        include_str!("fixtures/public-instance-mixed-vue2/manifest.json"),
        205,
        &[
            "options-api-slots/maestro/authored-hits.txt",
            "options-api-slots/maestro/mappings.txt",
            "options-api-slots/maestro/text.txt",
            "options-api-slots/maestro-document-0/text.ts",
            "vue2-native-event/maestro/authored-hits.txt",
            "vue2-native-event/maestro/mappings.txt",
            "vue2-native-event/maestro/text.txt",
            "vue2-native-event/maestro-document-0/text.ts",
        ],
        36,
    );
    if cfg!(feature = "legacy") {
        let matrix = load_matrix();
        let fixture = matrix
            .fixtures
            .iter()
            .find(|fixture| fixture.id == "vue2-native-event")
            .expect("reviewed mixed Vue 2 fixture");
        assert_eq!((fixture.options_api, fixture.legacy_vue2), (true, true));
        // Canon opts into Vue 2; Maestro's existing generator uses Vue 3
        // defaults. The feature enables this mixed fixture's capture only.
        let predecessor =
            include_str!("fixtures/public-instance-mixed-vue2/after/projection-record.txt");
        let successor = include_str!(
            "fixtures/public-instance-mixed-vue2/successor-11069/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("fixtures/public-instance-mixed-vue2/successor-11069/receipt.json"),
            "3c486d9b75a1a25cc6e1815fb478f00a89334c48627621ab552e47e0e05d6330",
            11069,
            predecessor,
            successor,
        );
        let latest_successor = include_str!(
            "fixtures/public-instance-mixed-vue2/successor-11142/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("fixtures/public-instance-mixed-vue2/successor-11142/receipt.json"),
            "ac91a6f06d1d22359daf89c5ccea8076a60bd9ab637951ccfd76f0fb45cbf253",
            11142,
            successor,
            latest_successor,
        );
        let collision_safe_successor = include_str!(
            "fixtures/public-instance-mixed-vue2/successor-11153/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("fixtures/public-instance-mixed-vue2/successor-11153/receipt.json"),
            "84c1c49f0874cb1cb0b3d5227af293e378cfd72be2957232c46dbed7a38aa00e",
            11153,
            latest_successor,
            collision_safe_successor,
        );
        assert_eq!(
            capture_fixture(fixture).render().as_str(),
            collision_safe_successor
        );
    }
}

fn authenticate_successor_receipt(
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

fn authenticate_packets(
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
        .join("tests/davinci_ts40_projection_support/fixtures")
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
