//! Static attributes the template string cannot carry are set at run time, as
//! `@vue/compiler-vapor`'s `isRuntimeOnlyProp`: the server drops a checkbox's
//! `true-value`, and a template attribute never writes content or a
//! `<textarea>` / `<select>` value. The transform binds their literal.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[track_caller]
fn assert_compiles(source: &str, retained: bool, expected: &str) {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane: retained,
            ..Default::default()
        },
    );
    assert_eq!(
        result.error_messages.len(),
        0,
        "{:?}",
        result.error_messages
    );
    assert_eq!(
        result.code.trim_end(),
        expected,
        "{source} (retained: {retained})"
    );
}

#[track_caller]
fn assert_both_lanes(source: &str, expected: &str) {
    for retained in [false, true] {
        assert_compiles(source, retained, expected);
    }
}

/// The model applies after mount, so it reads the props set before it.
#[test]
fn checkbox_value_props_leave_the_template() {
    assert_both_lanes(
        r#"<input type="checkbox" v-model="data" true-value="yes" false-value="no">"#,
        r#"import { setAttr as _setAttr, applyCheckboxModel as _applyCheckboxModel, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<input type=\"checkbox\">", true)

export function render(_ctx) {
  const n0 = t0()
  _applyCheckboxModel(n0, () => (_ctx.data), _value => (_ctx.data = _value))
  _renderEffect(() => {
    _setAttr(n0, "true-value", "yes")
    _setAttr(n0, "false-value", "no")
  })
  return n0
}"#,
    );
}

#[test]
fn a_nested_static_checkbox_gets_a_reference() {
    assert_both_lanes(
        r#"<div><input type="checkbox" true-value="y"><span>{{ a }}</span></div>"#,
        r#"import { child as _child, next as _next, txt as _txt, toDisplayString as _toDisplayString, setText as _setText, setAttr as _setAttr, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<div><input type=\"checkbox\"><span> </span></div>", true)

export function render(_ctx) {
  const n2 = t0()
  const n0 = _child(n2)
  const n1 = _next(n0)
  const x1 = _txt(n1)
  _renderEffect(() => {
    _setAttr(n0, "true-value", "y")
    _setText(x1, _toDisplayString(_ctx.a))
  })
  return n2
}"#,
    );
}

#[test]
fn a_spread_merges_checkbox_value_props() {
    assert_both_lanes(
        r#"<input v-bind="o" true-value="y">"#,
        r#"import { setDynamicProps as _setDynamicProps, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<input>", true)

export function render(_ctx) {
  const n0 = t0()
  _renderEffect(() => _setDynamicProps(n0, [_ctx.o, { "true-value": "y" }]))
  return n0
}"#,
    );
}

#[test]
fn textarea_value_leaves_the_template() {
    assert_both_lanes(
        r#"<textarea value="x"></textarea>"#,
        r#"import { setProp as _setProp, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<textarea></textarea>", true)

export function render(_ctx) {
  const n0 = t0()
  _renderEffect(() => _setProp(n0, "value", "x"))
  return n0
}"#,
    );
}

#[test]
fn select_value_leaves_the_template() {
    assert_both_lanes(
        r#"<select value="b"></select>"#,
        r#"import { setProp as _setProp, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<select></select>", true)

export function render(_ctx) {
  const n0 = t0()
  _renderEffect(() => _setProp(n0, "value", "b"))
  return n0
}"#,
    );
}

#[test]
fn content_attributes_leave_the_template() {
    assert_both_lanes(
        r#"<div innerHTML="<b>x</b>" textContent="t"></div>"#,
        r#"import { setProp as _setProp, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<div></div>", true)

export function render(_ctx) {
  const n0 = t0()
  _renderEffect(() => {
    _setProp(n0, "innerHTML", "<b>x</b>")
    _setProp(n0, "textContent", "t")
  })
  return n0
}"#,
    );
}

#[test]
fn an_input_that_cannot_be_a_checkbox_keeps_its_attribute() {
    assert_both_lanes(
        r#"<input true-value="y">"#,
        r#"import { template as _template } from 'vue';
const t0 = _template("<input true-value=\"y\">", true)

export function render(_ctx) {
  const n0 = t0()
  return n0
}"#,
    );
}

#[test]
fn a_dynamically_typed_input_sets_checkbox_value_props_at_run_time() {
    assert_both_lanes(
        r#"<input :type="kind" true-value="y">"#,
        r#"import { setProp as _setProp, setAttr as _setAttr, renderEffect as _renderEffect, template as _template } from 'vue';
const t0 = _template("<input>", true)

export function render(_ctx) {
  const n0 = t0()
  _renderEffect(() => {
    _setProp(n0, "type", _ctx.kind)
    _setAttr(n0, "true-value", "y")
  })
  return n0
}"#,
    );
}
