use super::{DomCompilerOptions, compile_template_with_options};
use vize_s0::Allocator;

#[test]
fn test_compile_experimental_patterned_template_cases() {
    let allocator = Allocator::new();
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
    let allocator = Allocator::new();
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
