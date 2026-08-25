//! Compound dynamic `v-bind` / `v-on` arguments must prefix every identifier.
//!
//! Codegen used to prepend `_ctx.` only to a simple token, or emit member /
//! call / operator expressions raw. SFC compile always sets
//! `prefix_identifiers`, so `:[prefix+suffix]`, `:[foo.bar]`, and
//! `:[keyOf(item)]` evaluated as `ReferenceError` at runtime.

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
fn prefixed_bind_concat_keys_prefix_both_identifiers() {
    assert_eq!(
        compile(r#"<div :[prefix+suffix]="value"></div>"#, true),
        concat!(
            "const { normalizeProps: _normalizeProps, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ ",
            "[_ctx.prefix+_ctx.suffix || \"\"]: _ctx.value }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_bind_member_keys_prefix_the_object() {
    assert_eq!(
        compile(r#"<div :[foo.bar]="value"></div>"#, true),
        concat!(
            "const { normalizeProps: _normalizeProps, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ ",
            "[_ctx.foo.bar || \"\"]: _ctx.value }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_bind_call_keys_prefix_callee_and_args() {
    assert_eq!(
        compile(r#"<div :[keyOf(item)]="value"></div>"#, true),
        concat!(
            "const { normalizeProps: _normalizeProps, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ ",
            "[_ctx.keyOf(_ctx.item) || \"\"]: _ctx.value }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_on_concat_keys_prefix_both_identifiers() {
    assert_eq!(
        compile(r#"<button @[prefix+suffix]="handler"></button>"#, true),
        concat!(
            "const { toHandlerKey: _toHandlerKey, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(), _createElementBlock(\"button\", { ",
            "[_toHandlerKey(_ctx.prefix+_ctx.suffix)]: _ctx.handler }, null, ",
            "16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_simple_bind_key_still_reads_through_ctx() {
    assert_eq!(
        compile(r#"<div :[attr]="value"></div>"#, true),
        concat!(
            "const { normalizeProps: _normalizeProps, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ ",
            "[_ctx.attr || \"\"]: _ctx.value }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_v_for_member_keys_keep_the_alias_local() {
    assert_eq!(
        compile(
            r#"<div v-for="item in items" :[item.id]="item.value"></div>"#,
            true,
        ),
        concat!(
            "const { openBlock: _openBlock, createElementBlock: _createElementBlock, ",
            "Fragment: _Fragment, renderList: _renderList } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(true), _createElementBlock(_Fragment, null, ",
            "_renderList(_ctx.items, (item) => {\n",
            "    return (_openBlock(), _createElementBlock(\"div\", { ",
            "[item.id || \"\"]: item.value }, null, 16 /* FULL_PROPS */))\n",
            "  }), 256 /* UNKEYED_FRAGMENT */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_v_for_concat_keys_prefix_only_the_outer_ident() {
    assert_eq!(
        compile(
            r#"<div v-for="item in items" :[item.id+suffix]="item.value"></div>"#,
            true,
        ),
        concat!(
            "const { openBlock: _openBlock, createElementBlock: _createElementBlock, ",
            "Fragment: _Fragment, renderList: _renderList } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(true), _createElementBlock(_Fragment, null, ",
            "_renderList(_ctx.items, (item) => {\n",
            "    return (_openBlock(), _createElementBlock(\"div\", { ",
            "[item.id+_ctx.suffix || \"\"]: item.value }, null, 16 /* FULL_PROPS */))\n",
            "  }), 256 /* UNKEYED_FRAGMENT */))\n",
            "}",
        )
    );
}

#[test]
fn unprefixed_compound_bind_keys_stay_as_authored() {
    assert_eq!(
        compile(r#"<div :[prefix+suffix]="value"></div>"#, false),
        concat!(
            "const { normalizeProps: _normalizeProps, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  return (_openBlock(), _createElementBlock(\"div\", _normalizeProps({ ",
            "[prefix+suffix || \"\"]: value }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}
