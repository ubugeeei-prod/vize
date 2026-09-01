use vize_atelier_core::{
    CodegenOptions, CodegenResult, TransformOptions, generate, parse, transform,
};
use vize_s0::String;

fn result_output(result: &CodegenResult) -> String {
    let mut output = String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

fn compile(source: &str, prefix_identifiers: bool) -> String {
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers,
            ..Default::default()
        },
        None,
    );
    result_output(&generate(
        &root,
        CodegenOptions {
            prefix_identifiers,
            ..Default::default()
        },
    ))
}

fn assert_has_props_patch_flag(output: &str, key: &str) {
    assert!(
        output.contains("8 /* PROPS */") && output.contains(key),
        "bind must stay dynamic so Vue patches the attr. Got:\n{output}"
    );
}

#[test]
fn concatenated_quoted_bind_keeps_props_patch_flag() {
    let output = compile(r#"<div :title="'Hello, ' + name + '!'"></div>"#, false);
    assert_has_props_patch_flag(&output, "\"title\"");
}

#[test]
fn concatenated_template_bind_keeps_props_patch_flag() {
    let output = compile("<div :id=\"`item-` + id + `!`\"></div>", false);
    assert_has_props_patch_flag(&output, "\"id\"");
}

#[test]
fn concatenated_class_bind_keeps_class_patch_flag() {
    let output = compile(r#"<div :class="'card ' + variant"></div>"#, false);
    assert!(
        output.contains("2 /* CLASS */"),
        "class concat must keep CLASS so Vue patches className. Got:\n{output}"
    );
}

#[test]
fn static_quoted_bind_stays_patchless() {
    let output = compile(r#"<div :title="'hello'"></div>"#, false);
    assert!(
        !output.contains("/* PROPS */") && !output.contains("/* CLASS */"),
        "a single quoted literal bind should stay patchless. Got:\n{output}"
    );
}

#[test]
fn prefixed_concatenated_string_bind_keeps_props_patch_flag() {
    let output = compile(r#"<div :title="'Hello, ' + name + '!'"></div>"#, true);
    assert_has_props_patch_flag(&output, "\"title\"");
}
