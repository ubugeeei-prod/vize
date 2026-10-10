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
    for name in ["AfsStepperDialog.stories", "native-nested-controls"] {
        let source = std::fs::read_to_string(fixture.join(format!("{name}.tsx.txt"))).unwrap();
        for (label, mode) in [
            ("vdom", JsxOutputMode::Vdom),
            ("vapor", JsxOutputMode::Vapor),
        ] {
            let expected: Value = serde_json::from_slice(
                &std::fs::read(fixture.join(format!("{name}.{label}.runtime.json.txt"))).unwrap(),
            )
            .unwrap();
            assert_eq!(
                complete_runtime_packet(&source, mode),
                expected,
                "{name}/{label}"
            );
        }
    }
}
