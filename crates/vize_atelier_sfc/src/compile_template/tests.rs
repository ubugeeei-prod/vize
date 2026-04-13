//! Tests for template compilation utilities.

use super::extraction::{extract_template_parts, extract_template_parts_full};
use super::string_tracking::{
    count_braces_outside_strings, count_braces_with_state, StringTrackState,
};
use super::vapor::{add_scope_id_to_template, transform_vapor_template_output};
use crate::types::{BlockLocation, SfcTemplateBlock};
use std::borrow::Cow;

#[test]
fn test_add_scope_id_to_template() {
    let input = r#"const t0 = _template("<div class='container'>Hello</div>")"#;
    let result = add_scope_id_to_template(input, "data-v-abc123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_transform_vapor_template_output_current_render_format() {
    let template = SfcTemplateBlock {
        content: Cow::Borrowed("<div>{{ msg }}</div>"),
        loc: BlockLocation {
            start: 0,
            end: 0,
            tag_start: 0,
            tag_end: 0,
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        },
        lang: None,
        src: None,
        attrs: Default::default(),
    };

    let vapor_code = r#"import { template as _template } from 'vue/vapor';
const t0 = _template("<div> </div>", true)

export function render(_ctx) {
  const n0 = t0()
  return n0
}"#;

    let result = transform_vapor_template_output(vapor_code, None, &template, None)
        .expect("current Vapor output should be transformed");

    insta::assert_snapshot!(result.as_str());
}

// --- count_braces_outside_strings tests ---

#[test]
fn test_count_braces_normal() {
    assert_eq!(count_braces_outside_strings("{ a: 1 }"), 0);
    assert_eq!(count_braces_outside_strings("{"), 1);
    assert_eq!(count_braces_outside_strings("}"), -1);
    assert_eq!(count_braces_outside_strings("{ { }"), 1);
}

#[test]
fn test_count_braces_ignores_string_braces() {
    assert_eq!(
        count_braces_outside_strings("_toDisplayString(isArray.value ? ']' : '}')"),
        0
    );
    assert_eq!(count_braces_outside_strings(r#"var x = "{";"#), 0);
    assert_eq!(count_braces_outside_strings("var x = `{`;"), 0);
}

#[test]
fn test_count_braces_mixed_string_and_code() {
    assert_eq!(count_braces_outside_strings("if (x) { var s = '}'"), 1);
}

#[test]
fn test_count_braces_escaped_quotes() {
    assert_eq!(count_braces_outside_strings(r"var x = '\'' + '}'"), 0);
}

#[test]
fn test_extract_template_parts_full_brace_in_string() {
    let template_code = r#"import { toDisplayString as _toDisplayString } from 'vue'

export function render(_ctx, _cache) {
  return _toDisplayString(isArray.value ? ']' : '}')
}"#;

    let (imports, _hoisted, render_fn, render_fn_name) = extract_template_parts_full(template_code);

    assert_eq!(render_fn_name, "render");
    insta::assert_debug_snapshot!((&imports, &render_fn));
    let trimmed = render_fn.trim();
    assert!(
        trimmed.ends_with('}'),
        "Render function should end with closing brace. Got:\n{}",
        render_fn
    );
}

#[test]
fn test_extract_template_parts_basic() {
    let template_code = r#"import { createVNode as _createVNode } from 'vue'

const _hoisted_1 = { class: "test" }

export function render(_ctx, _cache) {
  return _createVNode("div", _hoisted_1, "Hello")
}"#;

    let (imports, hoisted, _preamble, render_body, render_fn_name) =
        extract_template_parts(template_code);

    assert_eq!(render_fn_name, "render");
    insta::assert_debug_snapshot!((&imports, &hoisted, &render_body));
}

#[test]
fn test_extract_template_parts_vapor_template_declarations() {
    let template_code = r#"import { template as _template, renderEffect as _renderEffect, setText as _setText } from 'vue'

const t0 = _template("<div> </div>", true)

function render(_ctx, $props, $emit, $attrs, $slots) {
  const n0 = t0()
  _renderEffect(() => _setText(n0, _ctx.msg))
  return n0
}"#;

    let (_imports, hoisted, _preamble, render_body, render_fn_name) =
        extract_template_parts(template_code);

    assert_eq!(render_fn_name, "render");
    insta::assert_debug_snapshot!((&hoisted, &render_body));
}

#[test]
fn test_extract_template_parts_full_preserves_vapor_top_level_side_effects() {
    let template_code = r#"import { delegateEvents as _delegateEvents, template as _template } from 'vue'

const t0 = _template("<button>ok</button>", true)
_delegateEvents("click")

function render(_ctx, $props, $emit, $attrs, $slots) {
  const n0 = t0()
  return n0
}"#;

    let (_imports, hoisted, render_fn, render_fn_name) = extract_template_parts_full(template_code);

    assert_eq!(render_fn_name, "render");
    insta::assert_debug_snapshot!((&hoisted, &render_fn));
}

#[test]
fn test_extract_template_parts_full_ssr_render_function() {
    let template_code = r#"import { ssrRenderComponent as _ssrRenderComponent } from "vue/server-renderer"

export function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(_ssrRenderComponent(_ctx.Foo, null, null, _parent))
}"#;

    let (imports, _hoisted, render_fn, render_fn_name) = extract_template_parts_full(template_code);

    assert_eq!(render_fn_name, "ssrRender");
    insta::assert_debug_snapshot!((&imports, &render_fn));
}

#[test]
fn test_extract_template_parts_ssr_preserves_render_name_without_inline_body() {
    let template_code = r#"import { ssrRenderComponent as _ssrRenderComponent } from "vue/server-renderer"

export function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(_ssrRenderComponent(_ctx.Foo, null, null, _parent))
}"#;

    let (_imports, _hoisted, _preamble, render_body, render_fn_name) =
        extract_template_parts(template_code);

    assert_eq!(render_fn_name, "ssrRender");
    assert!(
        render_body.is_empty(),
        "SSR render functions should stay separate instead of being inlined. Got:\n{}",
        render_body
    );
}

// --- Multiline template literal tests ---

#[test]
fn test_count_braces_multiline_template_literal() {
    let mut state = StringTrackState::default();

    let line1 = r#"}, _toDisplayString(`${t("key")}: v${ver.major}.${"#;
    let count1 = count_braces_with_state(line1, &mut state);
    assert_eq!(count1, -1, "Line 1 brace count");
    assert!(
        !state.template_expr_brace_stack.is_empty(),
        "Should be inside template expression after line 1"
    );

    let line2 = "            ver.minor";
    let count2 = count_braces_with_state(line2, &mut state);
    assert_eq!(count2, 0, "Line 2 brace count");

    let line3 = r##"          }`) + "\n      ", 1 /* TEXT */)))"##;
    let count3 = count_braces_with_state(line3, &mut state);
    assert_eq!(count3, 0, "Line 3 brace count");
    assert!(!state.in_string, "Should be outside string after line 3");
    assert!(
        state.template_expr_brace_stack.is_empty(),
        "Template expression stack should be empty"
    );

    assert_eq!(count1 + count2 + count3, -1);
}

#[test]
fn test_extract_template_parts_multiline_template_literal() {
    let template_code = r#"import { openBlock as _openBlock, createElementBlock as _createElementBlock, toDisplayString as _toDisplayString, createCommentVNode as _createCommentVNode } from "vue"

export function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (show.value)
    ? (_openBlock(), _createElementBlock("div", {
      key: 0,
      class: "outer"
    }, [
      _createElementVNode("div", { class: "inner" }, [
        (ver.value)
          ? (_openBlock(), _createElementBlock("span", { key: 0 }, "\n        " + _toDisplayString(`${t("key")}: v${ver.value.major}.${
            ver.value.minor
          }`) + "\n      ", 1 /* TEXT */))
          : (_openBlock(), _createElementBlock("span", { key: 1 }, "no"))
      ])
    ]))
    : _createCommentVNode("v-if", true)
}"#;

    let (_imports, _hoisted, _preamble, render_body, _render_fn_name) =
        extract_template_parts(template_code);

    insta::assert_snapshot!(render_body.as_str());
}

#[test]
fn test_extract_template_parts_full_multiline_template_literal() {
    let template_code = r#"import { toDisplayString as _toDisplayString } from 'vue'

export function render(_ctx, _cache) {
  return _toDisplayString(`${t("key")}: v${ver.major}.${
    ver.minor
  }`)
}"#;

    let (_imports, _hoisted, render_fn, _render_fn_name) =
        extract_template_parts_full(template_code);

    insta::assert_snapshot!(render_fn.as_str());
    let trimmed = render_fn.trim();
    assert!(
        trimmed.ends_with('}'),
        "Render function should end with closing brace. Got:\n{}",
        render_fn
    );
}

// --- Generalized template literal / string tracking tests ---

#[test]
fn test_count_braces_template_literal_with_nested_object() {
    let mut state = StringTrackState::default();
    let line = r#"x = `result: ${fn({a: 1, b: {c: 2}})}`"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(
        count, 0,
        "Braces inside template expression should be balanced"
    );
    assert!(!state.in_string, "Template literal should be closed");
}

#[test]
fn test_count_braces_nested_template_literals() {
    let mut state = StringTrackState::default();
    let line = r#"x = `outer ${`inner ${x}`} end`"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(
        count, 0,
        "Nested template literals should not affect brace count"
    );
    assert!(!state.in_string, "All template literals should be closed");
}

#[test]
fn test_count_braces_multiline_template_expr_with_object() {
    let mut state = StringTrackState::default();

    let line1 = r#"x = `value: ${fn({"#;
    let c1 = count_braces_with_state(line1, &mut state);
    assert_eq!(
        c1, 1,
        "Line 1: object literal brace inside template expression"
    );

    let line2 = r#"  key: val"#;
    let c2 = count_braces_with_state(line2, &mut state);
    assert_eq!(c2, 0, "Line 2: no braces");

    let line3 = r#"})}`"#;
    let c3 = count_braces_with_state(line3, &mut state);
    assert_eq!(c3, -1, "Line 3: closing object brace");
    assert!(!state.in_string, "Template literal should be closed");
    assert_eq!(c1 + c2 + c3, 0, "Total should be balanced");
}

#[test]
fn test_count_braces_template_literal_with_arrow_function() {
    let mut state = StringTrackState::default();
    let line = r#"x = `${items.map(x => ({ name: x })).join()}`"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(count, 0);
    assert!(!state.in_string);
}

#[test]
fn test_count_braces_state_across_many_lines() {
    let mut state = StringTrackState::default();

    let c1 = count_braces_with_state("function render() {", &mut state);
    assert_eq!(c1, 1);

    let c2 = count_braces_with_state(r#"  return _toDisplayString(`${fn({"#, &mut state);
    assert_eq!(c2, 1, "Object literal brace inside template expression");

    let c3 = count_braces_with_state("    key: val,", &mut state);
    assert_eq!(c3, 0);

    let c4 = count_braces_with_state("    nested: {", &mut state);
    assert_eq!(c4, 1, "Nested brace inside template expression");

    let c5 = count_braces_with_state("      deep: true", &mut state);
    assert_eq!(c5, 0);

    let c6 = count_braces_with_state("    }", &mut state);
    assert_eq!(c6, -1, "Closing nested brace inside template expression");

    let c7 = count_braces_with_state(r#"  })}`)"#, &mut state);
    assert_eq!(c7, -1, "Closing outer object brace");

    let c8 = count_braces_with_state("}", &mut state);
    assert_eq!(c8, -1);

    assert_eq!(
        c1 + c2 + c3 + c4 + c5 + c6 + c7 + c8,
        0,
        "Total: function opens and closes"
    );
    assert!(!state.in_string);
    assert!(state.template_expr_brace_stack.is_empty());
}

#[test]
fn test_count_braces_regular_strings_with_braces() {
    let mut state = StringTrackState::default();

    let line = r#"if (x) { var s = "}" + '{' }"#;
    let count = count_braces_with_state(line, &mut state);
    assert_eq!(count, 0, "Braces inside regular strings should be ignored");
}

#[test]
fn test_extract_template_parts_deeply_nested_multiline() {
    let template_code = r#"import { toDisplayString as _toDisplayString, createElementBlock as _createElementBlock, openBlock as _openBlock, createCommentVNode as _createCommentVNode, createElementVNode as _createElementVNode } from "vue"

export function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (cond.value)
    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
        _createElementVNode("p", null, _toDisplayString(`${items.value.map(x => ({
          name: x.name,
          label: `${x.prefix}-${
            x.suffix
          }`
        })).length} items`)),
        _createElementVNode("span", null, "after")
      ]))
    : _createCommentVNode("v-if", true)
}"#;

    let (_imports, _hoisted, _preamble, render_body, _render_fn_name) =
        extract_template_parts(template_code);

    insta::assert_snapshot!(render_body.as_str());
}

#[test]
fn test_extract_template_parts_full_deeply_nested_multiline() {
    let template_code = r#"import { toDisplayString as _toDisplayString } from "vue"

export function render(_ctx, _cache) {
  return _toDisplayString(`${items.map(x => ({
    name: x.name,
    value: `nested-${
      x.value
    }`
  })).length} items`)
}"#;

    let (_imports, _hoisted, render_fn, _render_fn_name) =
        extract_template_parts_full(template_code);

    let trimmed = render_fn.trim();
    assert!(
        trimmed.ends_with('}'),
        "Render function should end with closing brace. Got:\n{}",
        render_fn
    );
    insta::assert_snapshot!(render_fn.as_str());
}
