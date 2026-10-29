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
    let successor_receipt_text =
        include_str!("fixtures/public-instance-options-api-slots/successor-11064/receipt.json");
    assert_eq!(
        sha256(successor_receipt_text),
        "e8916bf8eff167bc482fa64de01be67f384284d400ef69e1f8d854694108127c"
    );
    let successor_receipt: serde_json::Value =
        serde_json::from_str(successor_receipt_text).expect("slot outlet alias successor receipt");
    assert_eq!(successor_receipt["version"], 1);
    assert_eq!(successor_receipt["pullRequest"], 11064);
    let predecessor =
        include_str!("fixtures/public-instance-options-api-slots/after/projection-record.txt");
    assert_eq!(
        successor_receipt["predecessor"]["bytes"].as_u64(),
        Some(predecessor.len() as u64)
    );
    let predecessor_sha = sha256(predecessor);
    assert_eq!(
        successor_receipt["predecessor"]["sha256"].as_str(),
        Some(predecessor_sha.as_str())
    );
    let successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11064/projection-record.txt"
    );
    assert_eq!(
        successor_receipt["successor"]["bytes"].as_u64(),
        Some(successor.len() as u64)
    );
    let successor_sha = sha256(successor);
    assert_eq!(
        successor_receipt["successor"]["sha256"].as_str(),
        Some(successor_sha.as_str())
    );
    let matrix = load_matrix();
    let fixture = matrix
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "options-api-slots")
        .expect("reviewed Options API slot fixture");
    assert_eq!(capture_fixture(fixture).render().as_str(), successor);
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
        assert_eq!(
            capture_fixture(fixture).render().as_str(),
            include_str!("fixtures/public-instance-mixed-vue2/after/projection-record.txt")
        );
    }
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
