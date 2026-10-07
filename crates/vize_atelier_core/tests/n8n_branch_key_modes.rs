//! Raw versus processed branch keys, independently pinned to Vue 3.5.26.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "complete authored compiler packets and independent stock diagnostics"
)]
use serde_json::{Value, json};
use vize_atelier_core::{
    CodegenMode, CodegenOptions, TransformOptions, generate, lane::transform, parse,
};
use vize_l0::Allocator;

const OFFICIAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/retained-controls.json"
);

#[test]
fn branch_key_shape_and_scope_match_pinned_official_modes() {
    let oracle: Value = serde_json::from_str(OFFICIAL).expect("full official compiler controls");
    assert_eq!(oracle["compiler"]["version"], "3.5.26");
    assert_eq!(
        oracle["compiler"]["sha256"],
        "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec"
    );
    let mut packets = Vec::new();
    for case in oracle["cases"].as_array().expect("whole cases") {
        let source = case["source"].as_str().expect("authored source");
        for mode in case["modes"].as_array().expect("both official modes") {
            let prefixed = mode["prefixed"].as_bool().expect("mode");
            let allocator = Allocator::new();
            let (mut root, parse_errors) = parse(&allocator, source);
            assert!(parse_errors.is_empty(), "{parse_errors:?}");
            let errors = transform(
                &allocator,
                &mut root,
                TransformOptions {
                    prefix_identifiers: prefixed,
                    hoist_static: false,
                    ..Default::default()
                },
                None,
            );
            let generated = generate(
                &root,
                CodegenOptions {
                    prefix_identifiers: prefixed,
                    mode: if prefixed {
                        CodegenMode::Module
                    } else {
                        CodegenMode::Function
                    },
                    ..Default::default()
                },
            );
            packets.push(
                json!({"name":case["name"],"source":source,"prefixed":prefixed,
                "official":mode,"legacy":{"preamble":generated.preamble,"code":generated.code,
                    "errors":errors.iter().map(|e|json!({"code":format!("{:?}",e.code),
                        "message":e.message,"location":e.loc})).collect::<Vec<_>>()}}),
            );
            let receipt = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/test-receipts/n8n-retained-branch-key-modes.json");
            std::fs::create_dir_all(receipt.parent().expect("receipt directory"))
                .expect("directory");
            std::fs::write(
                receipt,
                serde_json::to_vec_pretty(&packets).expect("full packets"),
            )
            .expect("retain complete modules before assertions");
            let expected = mode["errors"].as_array().expect("complete official errors");
            assert_eq!(
                errors.len(),
                expected.len(),
                "{} prefixed={prefixed}",
                case["name"]
            );
            for (actual, expected) in errors.iter().zip(expected) {
                assert_eq!(format!("{:?}", actual.code), "VIfSameKey");
                assert_eq!(
                    actual.message.as_str(),
                    "v-if/v-else-if branches must use unique keys."
                );
                assert_eq!(expected["code"], 29);
                let loc = actual.loc.as_ref().expect("actual raw-new key location");
                assert_eq!(json!(loc.span.start), expected["loc"]["start"]["offset"]);
                assert_eq!(json!(loc.span.end), expected["loc"]["end"]["offset"]);
            }
        }
    }
    assert_eq!(packets.len(), 24);
}
