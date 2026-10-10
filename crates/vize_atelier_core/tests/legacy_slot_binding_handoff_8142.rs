//! Whole raw observations precede laws; one test owns the global profiler.
#![cfg(feature = "legacy")]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "authored integration fixtures retain complete observations before asserting"
)]

mod support {
    pub(super) mod legacy_slot_binding_handoff_8142;
}

use serde_json::{Value, json};
use std::path::Path;
use support::legacy_slot_binding_handoff_8142::{compile, judge};
use vize_atelier_core::CodegenMode;
use vize_l0::config::VueVersion;
use vize_relief::{AttributeNode, SimpleExpressionNode, TextNode};

#[test]
fn complete_legacy_slot_binding_handoff() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/legacy-slot-binding-handoff-8142/cases.json"
    ))
    .expect("authored complete fixture");
    let structures: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/legacy-slot-binding-handoff-8142/structure.json"
    ))
    .expect("independent whole structural laws");
    let cases = fixture["cases"].as_array().expect("authored cases");
    let mut packets = Vec::new();
    for case in cases {
        for dialect in [VueVersion::V2, VueVersion::V2_7, VueVersion::V3] {
            if case["kind"] == "v3-only" && dialect != VueVersion::V3 {
                continue;
            }
            for mode in [CodegenMode::Function, CodegenMode::Module] {
                for variant in ["legacy", "modern"] {
                    packets.push(compile(case, variant, dialect, mode));
                }
            }
        }
    }
    let layout = json!({"AttributeNode": size_of::<AttributeNode<'static>>(), "TextNode": size_of::<TextNode<'static>>(),
        "SimpleExpressionNode": size_of::<SimpleExpressionNode<'static>>(), "pointerWidth": usize::BITS});
    let receipt = json!({"schema": 1, "fixture": fixture, "structureFixture": structures, "pid": std::process::id(),
        "executable": std::env::current_exe().expect("actual executable"), "githubSha": std::env::var("GITHUB_SHA").ok(),
        "layout": layout, "packetCount": packets.len(), "sourceMapPolicy": "false: original physical spans are retained separately; encoded/decoded declaration mapping is not claimed", "packets": packets});
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/legacy-slot-binding-handoff-8142/whole-packets.json");
    std::fs::create_dir_all(output.parent().expect("artifact parent")).expect("create raw output");
    std::fs::write(
        &output,
        serde_json::to_vec_pretty(&receipt).expect("complete raw serialization"),
    )
    .expect("retain all raw before laws");
    assert_eq!(cases.len(), 17);
    assert_eq!(packets.len(), 196);
    if usize::BITS == 64 {
        assert_eq!(
            layout,
            json!({"AttributeNode": 56, "TextNode": 24, "SimpleExpressionNode": 88, "pointerWidth": 64})
        );
    }
    for packet in &packets {
        let case = cases
            .iter()
            .find(|case| case["id"] == packet["id"])
            .expect("authored case");
        judge(case, packet, &structures);
        if packet["variant"] != "legacy" || packet["dialect"] == "V3" {
            continue;
        }
        let modern = packets
            .iter()
            .find(|other| {
                other["id"] == packet["id"]
                    && other["dialect"] == packet["dialect"]
                    && other["mode"] == packet["mode"]
                    && other["variant"] == "modern"
            })
            .expect("whole equivalent source observation");
        if case["role"] != "error" {
            assert_eq!(
                packet["generated"], modern["generated"],
                "{}: whole Function/Module code+preamble+map",
                case["id"]
            );
        } else {
            assert_eq!(
                packet["first"]["root"]["slots"][0]["diagnosticsDebug"],
                modern["first"]["root"]["slots"][0]["diagnosticsDebug"],
                "{}: whole original parameter diagnostics",
                case["id"]
            );
        }
    }
}
