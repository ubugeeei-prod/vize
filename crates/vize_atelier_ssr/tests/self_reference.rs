use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_carton::{Bump, FxHashMap};

#[test]
fn experimental_self_reference_passes_maybe_self_reference() {
    let allocator = Bump::new();
    let (_, errors, result) = compile_ssr_with_options(
        &allocator,
        r#"<Self :value="value" />"#,
        SsrCompilerOptions {
            experimental_self_reference: true,
            ..Default::default()
        },
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    assert!(
        result.code.contains(r#"_resolveComponent("Self", true)"#),
        "{}",
        result.code
    );
}

#[test]
fn experimental_self_reference_ignores_setup_binding() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("Self".into(), BindingType::SetupConst);
    let (_, errors, result) = compile_ssr_with_options(
        &allocator,
        r#"<Self />"#,
        SsrCompilerOptions {
            experimental_self_reference: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                is_script_setup: true,
                ..Default::default()
            }),
            ..Default::default()
        },
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    assert!(
        result.code.contains(r#"_resolveComponent("Self", true)"#),
        "{}",
        result.code
    );
    assert!(!result.code.contains("$setup.Self"), "{}", result.code);
}
