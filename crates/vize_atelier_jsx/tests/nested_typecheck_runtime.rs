#![expect(clippy::disallowed_macros, reason = "complete runtime fixture packets")]

use serde_json::{Value, json};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, VdomCompileOptions, compile_jsx};
use vize_l0::Allocator;

pub fn complete_runtime_packet(source: &str, mode: JsxOutputMode) -> Value {
    let allocator = Allocator::new();
    let output = compile_jsx(
        &allocator,
        source,
        JsxLang::Tsx,
        &JsxCompileConfig {
            default_mode: mode,
            vdom: VdomCompileOptions {
                source_map: true,
                ..VdomCompileOptions::default()
            },
            ..JsxCompileConfig::default()
        },
    );
    json!({
        "module": output.module_code().as_str(), "moduleMap": output.source_map(),
        "diagnostics": format!("{:?}", output.diagnostics),
        "components": output.components.iter().map(|component| json!({
            "name": component.component_name(), "setup": format!("{:?}", component.component_setup()),
            "mode": format!("{:?}", component.mode()), "code": component.code(),
            "preamble": component.preamble(), "map": component.map(),
            "style": format!("{:?}", component.scoped_style())
        })).collect::<Vec<_>>()
    })
}

#[test]
fn nested_analysis_keeps_complete_normal_vdom_and_vapor_packets() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("tests/_fixtures/differential/typecheck/jsx-nested-native-expressions-8441");
    let successors: Value = serde_json::from_slice(
        &std::fs::read(fixture.join("authored-module-refusal-10701-successor.json")).unwrap(),
    )
    .unwrap();
    // Compile every case before assertions, so the first changed refusal does
    // not hide later producer, style or module packets from Actions review.
    let mut cases = Vec::new();
    for name in ["AfsStepperDialog.stories", "native-nested-controls"] {
        let source = std::fs::read_to_string(fixture.join(format!("{name}.tsx.txt"))).unwrap();
        for (label, mode) in [
            ("vdom", JsxOutputMode::Vdom),
            ("vapor", JsxOutputMode::Vapor),
        ] {
            // The frozen #8441 packet still records the former nested-root bug.
            // #8625 retains the root-fallthrough successor; #9455 adds slot stability.
            let output = if name == "AfsStepperDialog.stories" && label == "vapor" {
                "AfsStepperDialog.stories.vapor.forwarded-slot-boundaries-9455.runtime.json.txt"
                    .to_owned()
            } else {
                format!("{name}.{label}.runtime.json.txt")
            };
            let historical: Value =
                serde_json::from_slice(&std::fs::read(fixture.join(&output)).unwrap()).unwrap();
            // Refusal successors retain the original complete producer packet.
            // VDOM continues to compare the original whole module unchanged.
            let refusal_packet = if label == "vapor" {
                successors["cases"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|case| case["name"] == name && case["mode"] == label)
                    .unwrap()["successorPacket"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            } else {
                output.clone()
            };
            let refusal: Value =
                serde_json::from_slice(&std::fs::read(fixture.join(&refusal_packet)).unwrap())
                    .unwrap();
            let previous_expected_packet = if name == "AfsStepperDialog.stories" && label == "vapor"
            {
                "AfsStepperDialog.stories.vapor.static-values-10701.runtime.json.txt".to_owned()
            } else {
                refusal_packet.clone()
            };
            let previous_expected: Value = serde_json::from_slice(
                &std::fs::read(fixture.join(&previous_expected_packet)).unwrap(),
            )
            .unwrap();
            let expected_packet = if label == "vapor" {
                format!("{name}.vapor.child-values-6890.inert-template.runtime.json.txt")
            } else {
                previous_expected_packet.clone()
            };
            let expected: Value =
                serde_json::from_slice(&std::fs::read(fixture.join(&expected_packet)).unwrap())
                    .unwrap();
            cases.push(json!({
                "name": name, "mode": label, "source": source,
                "historicalPacket": output, "historical": historical,
                "refusalPacket": refusal_packet, "refusal": refusal,
                "previousExpectedPacket": previous_expected_packet, "previousExpected": previous_expected,
                "expectedPacket": expected_packet, "expected": expected,
                "current": complete_runtime_packet(&source, mode),
            }));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_JSX_NESTED_RUNTIME_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&json!({ "schemaVersion": 3, "cases": cases })).unwrap(),
        )
        .unwrap();
    }
    for case in cases {
        assert_eq!(
            case["current"], case["expected"],
            "{}/{}",
            case["name"], case["mode"]
        );
    }
}
