//! #9361: root classification and runtime modules use the same authored cases.
use vize_atelier_core::{
    WhitespaceStrategy, options::CodegenMode, parser::with_whitespace_strategy,
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_l0::{Allocator, cstr};

#[test]
fn comment_root_fragments_keep_attribute_fallthrough_and_real_siblings() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/dev_root_fragment.json")).unwrap();
    // Unmodified source-qualified capture: Check 39026935005, job 117380340006,
    // artifact 12098089241; raw SHA256 78ee1e16af4c5d54f554c7a74c20b4061d0fb301ac54404e37f0d7c45e80e417.
    // The same complete modules passed the independent Vue runtime oracle.
    let expected: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/dev_root_fragment_output.json")).unwrap();
    let mut capture = Vec::new();
    for fixture in fixtures.as_array().unwrap() {
        let name = fixture["name"].as_str().unwrap();
        let source = fixture["template"].as_str().unwrap();
        let strategy = if fixture["whitespace"] == "preserve" {
            WhitespaceStrategy::Preserve
        } else {
            WhitespaceStrategy::Condense
        };
        let arena = Allocator::new();
        let (_, errors, output) = with_whitespace_strategy(strategy, || {
            compile_template_with_options(
                &arena,
                source,
                DomCompilerOptions {
                    mode: CodegenMode::Module,
                    prefix_identifiers: true,
                    comments: fixture["comments"].as_bool().unwrap(),
                    ..Default::default()
                },
            )
        });
        assert!(
            errors.iter().all(|error| error.is_compatibility_notice()),
            "{name}: {errors:?}",
        );
        let code = cstr!("{}\n{}", output.preamble, output.code);
        capture.push(serde_json::json!({"name":name,"code":code}));
    }
    assert_eq!(capture.len(), 18);
    assert_eq!(capture, expected);
    if let Some(path) = std::env::var_os("VIZE_DEV_ROOT_FRAGMENT_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&capture).unwrap()).unwrap();
    }
}
