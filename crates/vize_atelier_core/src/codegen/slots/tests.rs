use super::super::helpers::is_valid_js_identifier;
use super::params::prefix_slot_defaults;
use crate::compile;

fn result_output(result: &super::super::CodegenResult) -> vize_s0::String {
    let mut output = vize_s0::String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

#[test]
fn test_is_valid_js_identifier_valid() {
    assert!(is_valid_js_identifier("foo"));
    assert!(is_valid_js_identifier("_bar"));
    assert!(is_valid_js_identifier("$baz"));
    assert!(is_valid_js_identifier("foo123"));
    assert!(is_valid_js_identifier("camelCase"));
    assert!(is_valid_js_identifier("PascalCase"));
}

#[test]
fn test_is_valid_js_identifier_invalid() {
    assert!(!is_valid_js_identifier("123foo")); // starts with number
    assert!(!is_valid_js_identifier("")); // empty
    assert!(!is_valid_js_identifier("foo-bar")); // contains hyphen
    assert!(!is_valid_js_identifier("foo.bar")); // contains dot
    assert!(!is_valid_js_identifier("foo bar")); // contains space
    assert!(!is_valid_js_identifier("item-header")); // hyphenated slot name
}

#[test]
fn test_hyphenated_slot_names_need_quotes() {
    assert!(!is_valid_js_identifier("item-header"));
    assert!(!is_valid_js_identifier("card-body"));
    assert!(!is_valid_js_identifier("main-content"));
    assert!(!is_valid_js_identifier("list-item"));
}

#[test]
fn test_regular_slot_names_are_valid_identifiers() {
    assert!(is_valid_js_identifier("default"));
    assert!(is_valid_js_identifier("header"));
    assert!(is_valid_js_identifier("footer"));
    assert!(is_valid_js_identifier("content"));
}

#[test]
fn test_prefix_slot_defaults() {
    // Default values should get _ctx. prefix
    assert_eq!(
        prefix_slot_defaults("{ item = defaultItem }"),
        "{ item = _ctx.defaultItem }"
    );
    assert_eq!(prefix_slot_defaults("{ count = 0 }"), "{ count = 0 }");
    assert_eq!(
        prefix_slot_defaults("{ name = 'test' }"),
        "{ name = 'test' }"
    );
    // Literals should not be prefixed
    assert_eq!(prefix_slot_defaults("{ x = true }"), "{ x = true }");
    assert_eq!(prefix_slot_defaults("{ x = false }"), "{ x = false }");
    assert_eq!(prefix_slot_defaults("{ x = null }"), "{ x = null }");
    assert_eq!(
        prefix_slot_defaults("{ x = undefined }"),
        "{ x = undefined }"
    );
    assert_eq!(
        prefix_slot_defaults("{ item = fallback.item }"),
        "{ item = _ctx.fallback.item }"
    );
    assert_eq!(
        prefix_slot_defaults("{ item, active = item.active }"),
        "{ item, active = item.active }"
    );
    assert_eq!(
        prefix_slot_defaults("{ label = makeLabel(seed) }"),
        "{ label = _ctx.makeLabel(_ctx.seed) }"
    );
    assert_eq!(
        prefix_slot_defaults("{ item = { label } }"),
        "{ item = { label: _ctx.label } }"
    );
    assert_eq!(
        prefix_slot_defaults("{ id = Math.random(), value = Number(seed) }"),
        "{ id = Math.random(), value = Number(_ctx.seed) }"
    );
    assert_eq!(
        prefix_slot_defaults("{ mapper = item => item.label, label = () => labelText }"),
        "{ mapper = item => item.label, label = () => _ctx.labelText }"
    );
}

#[test]
fn scoped_slot_default_expressions_prefix_context_only() {
    let result = compile!(
        r#"<Comp v-slot="{ item, active = item.active, label = fallback.label, meta = { text }, id = Math.random() }">{{ active }}{{ label }}</Comp>"#
    );
    let output = result_output(&result);

    assert!(output.contains("active = item.active"), "{output}");
    assert!(output.contains("label = _ctx.fallback.label"), "{output}");
    assert!(output.contains("meta = { text: _ctx.text }"), "{output}");
    assert!(output.contains("id = Math.random()"), "{output}");
    assert!(!output.contains("_ctx.item.active"), "{output}");
    assert!(!output.contains("_ctx.Math"), "{output}");
}

#[test]
fn v_slots_forwards_the_object_as_the_children_argument() {
    // `v-slots` is a compiler built-in (#3467), so it never reaches the custom
    // directive path: no `resolveDirective("slots")`, no `withDirectives`. With
    // nothing else contributing slots the forwarded value *is* the children
    // argument, matching `@vue/babel-plugin-jsx`.
    let result = compile!(r#"<Comp v-slots="slots" />"#);
    assert_eq!(
        result_output(&result).as_str(),
        "const { resolveComponent: _resolveComponent, openBlock: _openBlock, \
         createBlock: _createBlock } = Vue\n\n\
         function render(_ctx, _cache, $props, $setup, $data, $options) {\n  \
         const _component_Comp = _resolveComponent(\"Comp\")\n  \n  \
         return (_openBlock(), _createBlock(_component_Comp, null, slots, \
         1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn v_slots_spreads_after_the_authored_slots() {
    // The spread closes the object so a forwarded entry overrides an authored
    // one of the same name, and no `_` stability flag is emitted beside it.
    let result = compile!(r#"<Comp v-slots="slots"><span></span></Comp>"#);
    assert_eq!(
        result_output(&result).as_str(),
        "const { resolveComponent: _resolveComponent, createElementVNode: \
         _createElementVNode, openBlock: _openBlock, createBlock: _createBlock, \
         withCtx: _withCtx } = Vue\n\n\
         function render(_ctx, _cache, $props, $setup, $data, $options) {\n  \
         const _component_Comp = _resolveComponent(\"Comp\")\n  \n  \
         return (_openBlock(), _createBlock(_component_Comp, null, {\n    \
         default: _withCtx(() => [\n      \
         _createElementVNode(\"span\")\n    \
         ]),\n    \
         ...slots\n  \
         }, 1024 /* DYNAMIC_SLOTS */))\n}"
    );
}

#[test]
fn slot_outlet_vbind_object_preserves_optional_chaining() {
    let result = compile!(
        r#"<slot v-bind="external ? { isActive: undefined } : { isActive: scope?.isActive }" />"#
    );
    let output = result_output(&result);

    assert!(
        output.contains(r#"external ? { isActive: undefined } : { isActive: scope?.isActive }"#),
        "slot outlet ternary v-bind object must preserve optional chaining:\n{output}"
    );
    assert!(
        !output.contains(r#"{ isActive: scope.isActive }"#),
        "slot outlet ternary v-bind object must not emit an unguarded member access:\n{output}"
    );
}
