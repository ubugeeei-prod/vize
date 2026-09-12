use super::{
    DomCompilerOptions, compile_template_with_options,
    compile_template_with_template_syntax_codegen_and_experimental_options,
};
use vize_atelier_core::{CodegenExperimentalOptions, CodegenOptions, TemplateSyntaxMode};
use vize_s0::Allocator;

#[test]
fn test_compile_experimental_patterned_template_when_branches() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="status"><p v-when="'ready'">Ready</p><p v-when="_">Other</p></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(full.contains("status"), "{full}");
    assert!(full.contains("ready"), "{full}");
    assert!(full.contains("? (_openBlock()"), "{full}");
    assert!(!full.contains("_resolveDirective(\"case\")"), "{full}");
    assert!(!full.contains("_resolveDirective(\"when\")"), "{full}");
    assert!(!full.contains("_resolveDirective(\"match\")"), "{full}");
}

#[test]
fn test_compile_experimental_patterned_template_or_when() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="status"><p v-when="'ready' | 'done'">Done</p></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(full.contains("__vize_match"), "{full}");
    assert!(full.contains("(__vize_match) === ('ready')"), "{full}");
    assert!(full.contains("(__vize_match) === ('done')"), "{full}");
    assert!(!full.contains("_resolveDirective(\"when\")"), "{full}");
}

#[test]
fn test_compile_experimental_patterned_template_object_const_binding_and_guard() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="entry"><article v-when="{ kind: 'article', data: const article } if (article.published)">{{ article.title }}</article></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(
        full.contains("_renderList([entry], (__vize_match)"),
        "{full}"
    );
    assert!(
        full.contains(r#"((__vize_match).kind) === ('article')"#),
        "{full}"
    );
    assert!(
        full.contains("const { data: article } = __vize_match; return (article.published);"),
        "{full}"
    );
    assert!(
        full.contains(
            "_renderList([{ __vize_value: __vize_match }], ({ __vize_value: { data: article } }) =>",
        ),
        "{full}"
    );
    assert!(full.contains("_toDisplayString(article.title)"), "{full}");
}

#[test]
fn test_compile_experimental_patterned_template_shorthand_rest_as_and_nan() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="entry"><article v-when="{ kind: 'article', const data, ... } as const article">{{ article.kind }}:{{ data.title }}</article><p v-when="NaN">NaN</p></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(
        full.contains("typeof (__vize_match) === \"object\""),
        "{full}"
    );
    assert!(
        full.contains(
            "_renderList([{ __vize_value: __vize_match, __vize_as0: __vize_match }], ({ __vize_value: { data: data }, __vize_as0: article }) =>",
        ),
        "{full}"
    );
    assert!(full.contains("Number.isNaN(__vize_match)"), "{full}");
}

#[test]
fn test_compile_experimental_patterned_template_array_rest_and_as_binding() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<template v-match="items"><p v-when="[const first, ...const rest] as const tuple">{{ tuple.length }}:{{ first }}:{{ rest.length }}</p></template>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(full.contains("Array.isArray(__vize_match)"), "{full}");
    assert!(full.contains("(__vize_match).length >= 1"), "{full}");
    assert!(
        full.contains(
            "_renderList([{ __vize_value: __vize_match, __vize_as0: __vize_match }], ({ __vize_value: [first, ...rest], __vize_as0: tuple }) =>",
        ),
        "{full}"
    );
    assert!(full.contains("_toDisplayString(tuple.length)"), "{full}");
    assert!(full.contains("_toDisplayString(first)"), "{full}");
    assert!(full.contains("_toDisplayString(rest.length)"), "{full}");
}

#[test]
fn test_compile_experimental_patterned_template_reports_fallback_order() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, _) = compile_template_with_options(
        &allocator,
        r#"<template v-match="status"><p v-when="_">Other</p><p v-when="'ready'">Ready</p><p v-when="(_)">Again</p></template>"#,
        options,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("must be the last branch")),
        "fallback order must be reported: {:?}",
        errors
    );
    assert!(
        errors.iter().any(|error| error.message.contains("unique")),
        "duplicate fallback must be reported: {:?}",
        errors
    );
}

#[test]
fn test_compile_experimental_patterned_template_rejects_let_binding() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, _) = compile_template_with_options(
        &allocator,
        r#"<template v-match="entry"><p v-when="{ data: let article }">{{ article }}</p></template>"#,
        options,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("must use `const`")),
        "let bindings must report a const-only diagnostic: {:?}",
        errors
    );
}

#[test]
fn test_compile_experimental_patterned_template_reports_orphan_when() {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    };

    let (_, errors, _) = compile_template_with_options(
        &allocator,
        r#"<section><p v-when="'ready'">Ready</p></section>"#,
        options,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("direct children of a `v-match`")),
        "orphan `v-when` must be reported: {:?}",
        errors
    );
}

#[test]
fn test_compile_experimental_patterned_template_requires_flag() {
    let allocator = Allocator::new();
    let (_, errors, _) = compile_template_with_options(
        &allocator,
        r#"<template v-match="status"><p v-when="'ready'">Ready</p></template>"#,
        DomCompilerOptions::default(),
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("experimentals.patternedTemplate")),
        "disabled patterned templates must report their opt-in flag: {:?}",
        errors
    );
}

#[test]
fn test_compile_experimental_self_component_resolves_current_component() {
    let allocator = Allocator::new();
    let codegen_experimental_options = CodegenExperimentalOptions {
        component_name: Some("TreeNode".into()),
        self_component: true,
    };

    let (_, errors, result) =
        compile_template_with_template_syntax_codegen_and_experimental_options(
            &allocator,
            r#"<Self />"#,
            DomCompilerOptions::default(),
            TemplateSyntaxMode::Standard,
            CodegenOptions::default(),
            codegen_experimental_options,
        );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = format!("{}\n{}", result.preamble, result.code);
    assert!(
        full.contains(r#"_resolveComponent("TreeNode", true)"#),
        "{full}"
    );
}
