//! #9361: root classification and runtime modules use the same authored cases.
use vize_atelier_core::{
    WhitespaceStrategy, options::CodegenMode, parser::with_whitespace_strategy,
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_l0::{Allocator, PatchFlags, cstr};

#[test]
fn comment_root_fragments_keep_attribute_fallthrough_and_real_siblings() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/dev_root_fragment.json")).unwrap();
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
        let dev_root = PatchFlags::STABLE_FRAGMENT | PatchFlags::DEV_ROOT_FRAGMENT;
        let flag = cstr!(
            "{} /* STABLE_FRAGMENT, DEV_ROOT_FRAGMENT */",
            dev_root.bits()
        );
        assert_eq!(
            output.code.contains(flag.as_str()),
            fixture["devRoot"].as_bool().unwrap(),
            "{name}: {}",
            output.code,
        );
        if name == "preserve_space" {
            assert!(output.code.contains("_createTextVNode("), "{name}");
        }
        if name == "leading_space" || name == "trailing_space" {
            assert!(!output.code.contains("_createTextVNode("), "{name}");
        }
        if name == "production_strip" {
            assert!(!output.code.contains("createCommentVNode"));
            assert!(!output.code.contains("DEV_ROOT_FRAGMENT"));
        }
        let code = cstr!("{}\n{}", output.preamble, output.code);
        capture.push(serde_json::json!({"name":name,"code":code}));
    }
    assert_eq!(capture.len(), 18);
    if let Some(path) = std::env::var_os("VIZE_DEV_ROOT_FRAGMENT_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&capture).unwrap()).unwrap();
    }
}
