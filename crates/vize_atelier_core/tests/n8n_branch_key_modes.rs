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
const PARSE_NOTICES: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/retained-parse-notices.json"
);
const CASE_NAMES: [&str; 12] = [
    "simple-global",
    "compound-global",
    "literal-complex",
    "global-Math-member",
    "loop-local-simple",
    "loop-local-compound",
    "quoted-literal",
    "static-vs-binding",
    "literal-object-member",
    "lambda-binding",
    "local-shorthand",
    "erased-template-wrapper",
];

#[test]
fn branch_key_shape_and_scope_match_pinned_official_modes() {
    let oracle: Value = serde_json::from_str(OFFICIAL).expect("full official compiler controls");
    assert_eq!(oracle["compiler"]["version"], "3.5.26");
    assert_eq!(
        oracle["compiler"]["sha256"],
        "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec"
    );
    let notice_oracle: Value = serde_json::from_str(PARSE_NOTICES).expect("complete parse notices");
    let cases = oracle["cases"].as_array().expect("whole cases");
    let notice_cases = notice_oracle["cases"]
        .as_array()
        .expect("whole notice cases");
    assert_eq!(cases.len(), CASE_NAMES.len());
    assert_eq!(notice_cases.len(), CASE_NAMES.len());
    let mut packets = Vec::new();
    for ((case, notices), name) in cases.iter().zip(notice_cases).zip(CASE_NAMES) {
        assert_eq!(case["name"], name);
        assert_eq!(notices["name"], name);
        assert_eq!(notices["source"], case["source"]);
        let source = case["source"].as_str().expect("authored source");
        let modes = case["modes"].as_array().expect("both official modes");
        assert_eq!(modes.len(), 2);
        for (mode, expected_prefixed) in modes.iter().zip([false, true]) {
            let prefixed = mode["prefixed"].as_bool().expect("mode");
            assert_eq!(prefixed, expected_prefixed);
            let allocator = Allocator::new();
            let (mut root, parse_errors) = parse(&allocator, source);
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
                "official":mode,"parseOracle":notices,"legacy":{"preamble":generated.preamble,"code":generated.code,
                    "parseErrors":parse_errors.iter().map(|e|json!({"code":format!("{:?}",e.code),
                        "message":e.message,"location":e.loc})).collect::<Vec<_>>(),
                    "errors":errors.iter().map(|e|json!({"code":format!("{:?}",e.code),
                        "message":e.message,"location":e.loc})).collect::<Vec<_>>()}}),
            );
        }
    }
    assert_eq!(packets.len(), 24);
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    let directory = match std::env::var("NEXTEST_PROFILE") {
        Ok(profile) => target
            .join("nextest")
            .join(profile)
            .join("compiler-fixtures"),
        Err(_) => target.join("test-receipts"),
    };
    let receipt = directory.join("n8n-retained-branch-key-modes.json");
    std::fs::create_dir_all(receipt.parent().expect("receipt directory")).expect("directory");
    std::fs::write(
        receipt,
        serde_json::to_vec_pretty(&packets).expect("full packets"),
    )
    .expect("retain all complete parse, transform and module packets before assertions");
    for packet in &packets {
        assert_eq!(
            packet["legacy"]["parseErrors"], packet["parseOracle"]["parseErrors"],
            "{} prefixed={}",
            packet["name"], packet["prefixed"]
        );
        let expected = packet["official"]["errors"]
            .as_array()
            .expect("complete official errors");
        let actual = packet["legacy"]["errors"]
            .as_array()
            .expect("complete legacy errors");
        assert_eq!(
            actual.len(),
            expected.len(),
            "{} prefixed={}",
            packet["name"],
            packet["prefixed"]
        );
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(expected["code"], 29);
            assert_eq!(
                actual,
                &json!({"code":"VIfSameKey",
                "message":"v-if/v-else-if branches must use unique keys.",
                "location":{"span":{"start":expected["loc"]["start"]["offset"],
                    "end":expected["loc"]["end"]["offset"]}}})
            );
        }
    }
}
