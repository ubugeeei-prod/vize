#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

use std::path::{Path, PathBuf};
use vize_canon::batch::generate_vue_content_mapper_transform;

#[test]
fn public_slot_type_maps_only_complete_authored_keys_and_keeps_payload_scope() {
    let cases = [
        (
            "original",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/slot-rename/8011/original/Card.vue.txt"
            ),
            vec![("header", 41)],
        ),
        (
            "quoted-setup-payload",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/slot-rename/8011/controls/QuotedCard.vue.txt"
            ),
            vec![("\"item-row\"", 108), ("header", 175)],
        ),
        (
            "repeated-intersection",
            "<script setup lang=\"ts\">defineSlots<{ header(): void } & { header(): void }>();</script><template><slot name=\"header\" /></template>",
            vec![("header", 38), ("header", 59)],
        ),
        (
            "comment-type-lookalike",
            "<script setup lang=\"ts\">defineSlots /* <{ header(): void }> */ <{ header(): void }>();</script><template><slot name=\"header\" /></template>",
            vec![("header", 66)],
        ),
    ];
    for (name, source, expected_keys) in cases {
        let result = generate_vue_content_mapper_transform(Path::new("Card.vue"), source)
            .expect("whole transform");
        // Retain the entire module, wire mapping vectors and errors before judging.
        if let Some(root) = capture_root() {
            let root = root.join("original-slot-key-mappings");
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(
                root.join(name).with_extension("json"),
                serde_json::to_vec_pretty(&serde_json::json!({
                    "name": name, "sourceSha": std::env::var("SOURCE_SHA").ok(),
                    "original": source, "transform": result,
                }))
                .unwrap(),
            )
            .unwrap();
        }
        let alias = result.text.find("type __VizeSlots = ").expect("alias");
        let end = result
            .text
            .find("export type { __VizeSlots as Slots };")
            .expect("whole alias end");
        let rows: Vec<_> = result
            .mappings
            .iter()
            .filter(|row| alias <= row.0[0] && row.0[0] < end)
            .map(|row| {
                let [generated, length, original, original_length, kind, features] = row.0;
                (
                    result.text.get(generated..generated + length).unwrap(),
                    source.get(original..original + original_length).unwrap(),
                    length == original_length,
                    original,
                    kind,
                    features,
                )
            })
            .collect();
        let wanted: Vec<_> = expected_keys
            .iter()
            .map(|(key, start)| (*key, *key, true, *start, 0, 1_048_575))
            .collect();
        assert_eq!(rows, wanted, "complete public alias mapping for {name}");
        assert!(result.diagnostics.is_empty(), "{result:#?}");
    }
}

fn capture_root() -> Option<PathBuf> {
    std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let profile = std::env::var_os("NEXTEST_PROFILE")?;
            Some(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()?
                    .parent()?
                    .join("target/nextest")
                    .join(profile),
            )
        })
}
