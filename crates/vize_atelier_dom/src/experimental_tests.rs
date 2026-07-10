use super::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_carton::Bump;
use vize_carton::FxHashMap;

#[test]
fn test_compile_experimental_patterned_template_cases() {
    let allocator = Bump::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="status"><p v-case="'ready'">Ready</p><p v-case.default>Other</p></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(full.contains("status"), "{full}");
    assert!(full.contains("ready"), "{full}");
    assert!(full.contains("? (_openBlock()"), "{full}");
    assert!(!full.contains("_resolveDirective(\"case\")"), "{full}");
    assert!(!full.contains("_resolveDirective(\"match\")"), "{full}");
}

#[test]
fn test_compile_experimental_patterned_template_array_case() {
    let allocator = Bump::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="status"><p v-case="['ready', 'done']">Done</p></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(full.contains("includes(status)"), "{full}");
    assert!(!full.contains("_resolveDirective(\"case\")"), "{full}");
}

#[test]
fn test_compile_experimental_self_reference_resolves_reserved_self() {
    let allocator = Bump::new();
    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<Self :value="value" />"#,
        DomCompilerOptions {
            experimental_self_reference: true,
            ..Default::default()
        },
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(
        full.contains(r#"_resolveComponent("Self", true)"#),
        "{full}"
    );
}

#[test]
fn test_compile_experimental_self_reference_ignores_setup_binding() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("Self".into(), BindingType::SetupConst);
    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<Self />"#,
        DomCompilerOptions {
            mode: vize_atelier_core::options::CodegenMode::Module,
            prefix_identifiers: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                is_script_setup: true,
                ..Default::default()
            }),
            experimental_self_reference: true,
            ..Default::default()
        },
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(
        full.contains(r#"_resolveComponent("Self", true)"#),
        "{full}"
    );
    assert!(!full.contains("$setup.Self"), "{full}");
}
