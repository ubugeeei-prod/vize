//! Whole generated programs and diagnostic-only ownership for listener laws.

use crate::{ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options};
use std::path::Path;

#[test]
fn contextual_listener_complete_projection_and_mapping_packets() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/loose-listener-context-11600");
    let cases: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/loose-listener-context-11600/cases.json"
    )))
    .unwrap();
    let mut packets = Vec::new();
    for shared_module in [false, true] {
        for case in cases.as_array().unwrap() {
            let file = case["file"].as_str().unwrap();
            let source =
                std::fs::read_to_string(directory.join(vize_carton::cstr!("{file}.txt"))).unwrap();
            for preserve_unused in [false, true] {
                let options = ContentMapperTransformOptions::default()
                    .with_preserve_unused_diagnostics(preserve_unused)
                    .with_shared_virtual_module(shared_module);
                let transform = generate_vue_content_mapper_transform_with_options(
                    Path::new(file),
                    &source,
                    options,
                )
                .unwrap();
                let mut packet = serde_json::json!({
                    "file":file,
                    "source":source,
                    "preserveUnused":preserve_unused,
                    "contract":case["contract"],
                    "transform":transform,
                });
                if shared_module {
                    packet["sharedModule"] = true.into();
                }
                packets.push(packet);
            }
        }
    }
    if let Some(directory) = std::env::var_os("VIZE_LOOSE_LISTENER_PACKET_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("listener-packets.json"),
            serde_json::to_vec_pretty(&packets).unwrap(),
        )
        .unwrap();
    }
    // Missing expectations remain RED until exact-source host diagnostics and
    // every mapped range have been reviewed; capture alone cannot qualify this.
    insta::assert_snapshot!(
        "loose_listener_complete_source_and_diagnostic_mappings",
        serde_json::to_string_pretty(&packets).unwrap()
    );
}
