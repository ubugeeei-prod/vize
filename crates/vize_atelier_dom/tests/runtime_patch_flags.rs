//! Runtime-sensitive patch-flag regressions reported in #10168.

use vize_atelier_core::options::CodegenMode;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_l0::{Allocator, cstr};

#[test]
fn static_root_props_keep_text_patch_flags() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/runtime_patch_flags.json")).unwrap();
    let mut capture = Vec::new();

    for fixture in fixtures.as_array().unwrap() {
        let name = fixture["name"].as_str().unwrap();
        let source = fixture["template"].as_str().unwrap();
        let allocator = Allocator::new();
        let (_, errors, output) = compile_template_with_options(
            &allocator,
            source,
            DomCompilerOptions {
                mode: CodegenMode::Module,
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            errors.iter().all(|error| error.is_compatibility_notice()),
            "{name}: {errors:?}",
        );
        assert_eq!(
            output.code.matches("1 /* TEXT */").count(),
            1,
            "{name}: dynamic text requires the TEXT patch flag:\n{}",
            output.code,
        );
        capture.push(serde_json::json!({
            "name": name,
            "template": source,
            "code": cstr!("{}\n{}", output.preamble, output.code),
        }));
    }

    assert_eq!(capture.len(), 2);
    if let Some(path) = std::env::var_os("VIZE_RUNTIME_PATCH_FLAGS_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&capture).unwrap()).unwrap();
    }
}
