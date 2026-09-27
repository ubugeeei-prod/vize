//! Exhaustive downstream experimental-option literals from published 0.429.0.
use vize_atelier_core::{TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_ssr::{
    Allocator, SsrCompilerExperimentalOptions, SsrCompilerOptions,
    compile_ssr_with_custom_elements_template_syntax_and_experimental_options,
    compile_ssr_with_options,
};

#[test]
fn published_experimental_options_literal_keeps_raw_template_slotted_default() {
    let allocator = Allocator::new();
    let experimental = SsrCompilerExperimentalOptions {
        component_name: None,
        self_component: false,
        source_map: false,
        source_map_filename: None,
    };
    let (_, errors, result) =
        compile_ssr_with_custom_elements_template_syntax_and_experimental_options(
            &allocator,
            "<slot />",
            SsrCompilerOptions {
                scope_id: Some("data-v-test".into()),
                ..Default::default()
            },
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            experimental,
        );
    let (_, default_errors, default_result) = compile_ssr_with_options(
        &allocator,
        "<slot />",
        SsrCompilerOptions {
            scope_id: Some("data-v-test".into()),
            ..Default::default()
        },
    );
    assert!(errors.is_empty());
    assert!(default_errors.is_empty());
    assert_eq!(result.code, default_result.code);
    assert_eq!(result.preamble, default_result.preamble);
    assert_eq!(result.map, default_result.map);
}
