//! Raw JS comment formatting parity for emitted prop expressions.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s1_to_s2::emit_dom;

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

#[track_caller]
fn assert_comment_tokens(source: &str, expected: &[&str]) {
    let output = assembled(source);
    assert_eq!(
        comment_tokens(output.as_str()),
        expected,
        "unexpected emitted comment tokens in output:\n{output}"
    );
}

fn comment_tokens(output: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    for line in output.lines() {
        let mut offset = 0;
        while offset < line.len() {
            let block = line[offset..].find("/*").map(|start| offset + start);
            let line_comment = line[offset..].find("//").map(|start| offset + start);
            let Some(start) = earliest_comment(block, line_comment) else {
                break;
            };
            if block == Some(start) {
                let end = line[start..]
                    .find("*/")
                    .map(|comment_end| start + comment_end + 2)
                    .unwrap_or(line.len());
                tokens.push(&line[start..end]);
                offset = end;
            } else {
                tokens.push(line[start..].trim_end());
                break;
            }
        }
    }
    tokens
}

fn earliest_comment(block: Option<usize>, line_comment: Option<usize>) -> Option<usize> {
    match (block, line_comment) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(offset), None) | (None, Some(offset)) => Some(offset),
        (None, None) => None,
    }
}

#[test]
fn object_prop_value_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<div :data-payload="{
  value,
  next: count // payload lane note
}"></div>"#,
        ),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", { \"data-payload\": {
  value,
  next: count /*  payload lane note */
} }, null, 8 /* PROPS */, [\"data-payload\"]))
}"
    );
}

#[test]
fn normalized_style_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<div :style="{
  transitionDelay: `${index * 50}ms`, // delay between each item
}"></div>"#,
        ),
        "\
const { normalizeStyle: _normalizeStyle, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    style: _normalizeStyle({
  transitionDelay: `${index * 50}ms`, /*  delay between each item */
})
  }, null, 4 /* STYLE */))
}"
    );
}

#[test]
fn normalized_class_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<div class="base" :class="[
  active,
  pending // pending class note
]"></div>"#,
        ),
        "\
const { normalizeClass: _normalizeClass, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    class: _normalizeClass([\"base\", [
  active,
  pending /*  pending class note */
]])
  }, null, 2 /* CLASS */))
}"
    );
}

#[test]
fn object_on_spread_line_comment_is_emitted_as_block_comment() {
    assert_eq!(
        assembled(
            r#"<button v-on="{
  click: onClick // listener map note
}"></button>"#,
        ),
        "\
const { toHandlers: _toHandlers, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"button\", _toHandlers({
  click: onClick /*  listener map note */
}, true), null, 16 /* FULL_PROPS */))
}"
    );
}

#[test]
fn child_and_directive_expression_line_comments_are_emitted_as_block_comments() {
    assert_comment_tokens(
        r#"<div v-if="ready // gate note
"></div>"#,
        &["/*  gate note */"],
    );
    assert_comment_tokens(
        r#"<div>{{ total // interpolation note
}}</div>"#,
        &["/*  interpolation note */", "/* TEXT */"],
    );
    assert_comment_tokens(
        r#"<div v-html="html // html note
"></div>"#,
        &["/*  html note */", "/* PROPS */"],
    );
    assert_comment_tokens(
        r#"<div v-text="label // text note
"></div>"#,
        &["/*  text note */", "/* PROPS */"],
    );
    assert_comment_tokens(
        r#"<div v-show="visible // show note
"></div>"#,
        &["/* NEED_PATCH */", "/*  show note */"],
    );
    assert_comment_tokens(
        r#"<div v-demo="payload // directive note
"></div>"#,
        &["/* NEED_PATCH */", "/*  directive note */"],
    );
}

#[test]
fn spread_model_and_memo_line_comments_are_emitted_as_block_comments() {
    assert_comment_tokens(
        r#"<div v-bind="props // bind object note
"></div>"#,
        &["/*  bind object note */", "/* FULL_PROPS */"],
    );
    assert_comment_tokens(
        r#"<div v-bind="{
  id: currentId // bind spread note
}"></div>"#,
        &["/*  bind spread note */", "/* FULL_PROPS */"],
    );
    assert_comment_tokens(
        r#"<button v-on="listeners // on spread note
"></button>"#,
        &["/*  on spread note */", "/* FULL_PROPS */"],
    );
    assert_comment_tokens(
        r#"<slot v-on="{
  click: onClick // outlet listener note
}"></slot>"#,
        &["/*  outlet listener note */"],
    );
    assert_comment_tokens(
        r#"<input v-model="form.name // model note
">"#,
        &["/*  model note */", "/* PROPS */", "/*  model note */"],
    );
    assert_comment_tokens(
        r#"<div v-memo="[item.id // memo note
]"></div>"#,
        &["/*  memo note */"],
    );
}

#[test]
fn loop_and_slot_line_comments_are_emitted_as_block_comments() {
    assert_comment_tokens(
        r#"<div v-for="item in items // list note
" :key="item.id"></div>"#,
        &["/*  list note */", "/* KEYED_FRAGMENT */"],
    );
    assert_comment_tokens(
        r#"<slot :name="slotName // outlet name note
"></slot>"#,
        &["/*  outlet name note */"],
    );
    assert_comment_tokens(
        r#"<Comp v-slots="slots // slots note
"></Comp>"#,
        &["/*  slots note */", "/* DYNAMIC_SLOTS */"],
    );
}
