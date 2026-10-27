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
    let matrix = load_matrix();
    let fixture = matrix
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "options-api-slots")
        .expect("reviewed Options API slot fixture");
    assert_eq!(
        capture_fixture(fixture).render().as_str(),
        include_str!("fixtures/public-instance-options-api-slots/after/projection-record.txt")
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
