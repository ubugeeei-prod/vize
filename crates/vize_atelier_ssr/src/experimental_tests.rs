use crate::{
    SsrCompilerExperimentalOptions, SsrCompilerOptions,
    compile_ssr_with_template_syntax_and_experimental_options,
};
use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::Allocator;

#[test]
fn test_experimental_self_component_resolves_current_component() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_with_template_syntax_and_experimental_options(
        &allocator,
        r#"<Self />"#,
        SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        SsrCompilerExperimentalOptions {
            component_name: Some("TreeNode".into()),
            self_component: true,
        },
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    assert!(
        result
            .code
            .contains(r#"_resolveComponent("TreeNode", true)"#),
        "{}",
        result.code
    );
}
