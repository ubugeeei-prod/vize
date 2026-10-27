//! Whole SFC consumers share the same IR operation; VDOM/SSR stay controls.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "complete SFC packets"
)]

use serde_json::{Value, json};
use vize_atelier_sfc::{SfcCompileOptions, compile_sfc, parse_sfc};

fn compile(source: &str, vapor: bool, ssr: bool) -> Value {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let mut options = SfcCompileOptions {
        vapor,
        ..Default::default()
    };
    options.template.ssr = ssr;
    let output = compile_sfc(&descriptor, options).unwrap();
    assert!(output.errors.is_empty(), "{:?}", output.errors);
    serde_json::to_value(output).unwrap()
}

#[test]
fn original_sfc_literal_style_reaches_the_shared_static_template() {
    let old = std::env::var_os("VIZE_LITERAL_STYLE_EXPECT_OLD").is_some();
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/App.vue.txt"
    );
    let reference = source.replace(
        ":style=\"{fontWeight:'700'}\"",
        "style=\"font-weight:700;\"",
    );
    let mut packets = Vec::new();
    for vapor in [false, true] {
        for ssr in [false, true] {
            let output = compile(source, vapor, ssr);
            if !old && vapor && !ssr {
                assert_eq!(
                    output,
                    compile(&reference, vapor, ssr),
                    "whole static SFC DTO"
                );
            }
            packets.push(
                json!({"source":source,"vapor":vapor,"ssr":ssr,"fold":vapor&&!ssr,"output":output}),
            );
        }
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_SFC_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}
