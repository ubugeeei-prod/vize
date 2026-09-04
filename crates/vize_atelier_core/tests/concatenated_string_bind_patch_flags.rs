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

#[test]
fn concatenated_quoted_bind_keeps_props_patch_flag() {
    let output = compile(r#"<div :title="'Hello, ' + name + '!'"></div>"#, false);
    assert_eq!(
        output,
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { title: 'Hello, ' + name + '!' }, null, 8 /* PROPS */, [\"title\"]))
}"
    );
}

#[test]
fn concatenated_template_bind_keeps_props_patch_flag() {
    let output = compile("<div :id=\"`item-` + id + `!`\"></div>", false);
    assert_eq!(
        output,
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { id: `item-` + id + `!` }, null, 8 /* PROPS */, [\"id\"]))
}"
    );
}

#[test]
fn concatenated_class_bind_keeps_class_patch_flag() {
    let output = compile(r#"<div :class="'card ' + variant"></div>"#, false);
    assert_eq!(
        output,
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass('card ' + variant)
  }, null, 2 /* CLASS */))
}"
    );
}

#[test]
fn static_quoted_bind_stays_patchless() {
    let output = compile(r#"<div :title="'hello'"></div>"#, false);
    assert_eq!(
        output,
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { title: 'hello' }))
}"
    );
}

#[test]
fn prefixed_concatenated_string_bind_keeps_props_patch_flag() {
    let output = compile(r#"<div :title="'Hello, ' + name + '!'"></div>"#, true);
    assert_eq!(
        output,
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { title: 'Hello, ' + _ctx.name + '!' }, null, 8 /* PROPS */, [\"title\"]))
}"
    );
}
