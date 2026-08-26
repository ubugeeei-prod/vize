//! `v-model:[arg]` on a component (#3391).
//!
//! Codegen emits the dynamic argument through the shared expression emitter
//! rather than hard-prefixing it with `_ctx.`, so the argument has to be
//! prefixed by the element transform exactly like the bound value is. These
//! tests pin the template lane's output so that stays true: with
//! `prefix_identifiers`, both the key and the update-listener name read
//! `_ctx.arg`; without it — the shape the JSX lane compiles with — they stay the
//! raw closure identifiers.

use vize_atelier_core::{
    CodegenOptions, CodegenResult, TransformOptions, generate, parse, transform,
};
use vize_s0::String;

const SOURCE: &str = r#"<Comp v-model:[arg]="value"/>"#;
const SOURCE_WITH_MODIFIERS: &str = r#"<Comp v-model:[arg].trim="value"/>"#;

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
fn prefixed_templates_render_the_argument_through_ctx() {
    assert_eq!(
        compile(SOURCE, true),
        concat!(
            "const { resolveComponent: _resolveComponent, normalizeProps: _normalizeProps, ",
            "openBlock: _openBlock, createBlock: _createBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  const _component_Comp = _resolveComponent(\"Comp\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_Comp, _normalizeProps({ ",
            "[_ctx.arg]: _ctx.value,\n",
            "  [\"onUpdate:\" + _ctx.arg]: $event => ((_ctx.value) = $event) }), null, ",
            "16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn prefixed_templates_render_the_modifiers_key_through_ctx() {
    assert_eq!(
        compile(SOURCE_WITH_MODIFIERS, true),
        concat!(
            "const { resolveComponent: _resolveComponent, normalizeProps: _normalizeProps, ",
            "openBlock: _openBlock, createBlock: _createBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  const _component_Comp = _resolveComponent(\"Comp\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_Comp, _normalizeProps({ ",
            "[_ctx.arg]: _ctx.value,\n",
            "  [\"onUpdate:\" + _ctx.arg]: $event => ((_ctx.value) = $event),\n",
            "  [_ctx.arg + \"Modifiers\"]: { trim: true } }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}

#[test]
fn unprefixed_templates_keep_the_argument_as_authored() {
    assert_eq!(
        compile(SOURCE, false),
        concat!(
            "const { resolveComponent: _resolveComponent, normalizeProps: _normalizeProps, ",
            "openBlock: _openBlock, createBlock: _createBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  const _component_Comp = _resolveComponent(\"Comp\")\n",
            "  \n",
            "  return (_openBlock(), _createBlock(_component_Comp, _normalizeProps({ [arg]: value,\n",
            "  [\"onUpdate:\" + arg]: $event => ((value) = $event) }), null, 16 /* FULL_PROPS */))\n",
            "}",
        )
    );
}
