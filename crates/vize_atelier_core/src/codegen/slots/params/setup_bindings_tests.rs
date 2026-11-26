use super::prefix_slot_defaults_with_context;
use crate::codegen::context::CodegenContext;
use crate::{BindingMetadata, CodegenOptions};
use serde_json::{Value, json};

#[test]
fn inline_immutable_setup_defaults_keep_exact_authored_parameter_packets() {
    let controls: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/slot-default-setup-bindings-8142/controls.json"
    )))
    .expect("independently authored controls");
    let mut packets = Vec::new();
    for case in controls["cases"].as_array().expect("cases") {
        let metadata: Option<BindingMetadata> = if case["bindings"].is_null() {
            None
        } else {
            Some(
                serde_json::from_value(json!({
                    "bindings": case["bindings"],
                    "propsAliases": {},
                    "isScriptSetup": true,
                }))
                .expect("authored existing metadata kinds"),
            )
        };
        let ctx = CodegenContext::new(CodegenOptions {
            inline: case["inline"].as_bool().expect("inline"),
            binding_metadata: metadata,
            ..CodegenOptions::default()
        });
        let output =
            prefix_slot_defaults_with_context(case["source"].as_str().expect("whole source"), &ctx);
        packets.push(json!({
            "control": case,
            "actualInline": ctx.options.inline,
            "actualBindingMetadata": ctx.options.binding_metadata,
            "actualWholeParameters": output.as_str(),
        }));
    }
    // Retain every complete context/input/output before judging any product law.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../target/differential/slot-default-setup-bindings-8142/core-parameter-packets.json",
    );
    std::fs::create_dir_all(path.parent().expect("output directory"))
        .expect("create raw directory");
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&json!({
            "schema": 1,
            "plane": "source-core-parameter-unit",
            "producerPid": std::process::id(),
            "githubSha": std::env::var("GITHUB_SHA").ok(),
            "controls": controls,
            "packets": packets,
        }))
        .expect("whole packet serialization"),
    )
    .expect("retain every raw observation");

    assert_eq!(packets.len(), 24, "frozen complete control count");
    for packet in packets {
        assert_eq!(
            packet["actualWholeParameters"], packet["control"]["expectedWholeParameters"],
            "complete parameters for {}",
            packet["control"]["id"]
        );
    }
}
