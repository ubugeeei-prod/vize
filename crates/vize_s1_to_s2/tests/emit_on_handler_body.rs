//! Event handler-body admission and emission boundaries.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed;
use vize_s0::Allocator;
use vize_s1_to_s2::{UnsupportedReason as Reason, emit_dom, emit_dom_source};

fn assembled(source: &str) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

fn shipped(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template(&allocator, source);
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn assert_shipped_parity(source: &str) {
    assert_eq!(assembled(source), shipped(source), "{source}");
}

fn refused_reason(source: &str) -> Option<Reason> {
    let allocator = Allocator::new();
    emit_dom_source(&allocator, source)
        .expect_err(source)
        .reason()
}

#[test]
fn return_statement_handler_is_admitted_in_body_context() {
    assert_eq!(
        assembled(r#"<div @click="return false"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: $event => {return false}
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn line_comment_handler_keeps_the_closing_brace_outside_the_comment() {
    assert_eq!(
        assembled(r#"<div @click="foo(); // note"></div>"#),
        "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", {
    onClick: $event => {foo(); // note
    }
  }, null, 8 /* PROPS */, [\"onClick\"]))
}"
    );
}

#[test]
fn multiline_statement_handler_keeps_authored_padding() {
    assert_shipped_parity(
        r#"<button @click="
          state.showModal = false;
          updateDescription();
        "></button>"#,
    );
    assert_shipped_parity(
        r#"<button @click.stop.prevent="
          state.hide = !state.hide;
          clearConnectedPort();
        "></button>"#,
    );
}

#[test]
fn module_only_handler_forms_stay_unsupported() {
    for source in [
        r#"<div @click="import thing from 'pkg'"></div>"#,
        r#"<div @click="export const value = 1"></div>"#,
    ] {
        assert_eq!(refused_reason(source), Some(Reason::OnHandlerNotJs));
    }
}

#[test]
fn invalid_body_control_flow_stays_unsupported() {
    assert_eq!(
        refused_reason(r#"<div @click="break"></div>"#),
        Some(Reason::OnHandlerNotJs)
    );
}

#[test]
fn duplicate_lexical_handler_declarations_stay_unsupported() {
    assert_eq!(
        refused_reason(r#"<div @click="let value; let value"></div>"#),
        Some(Reason::OnHandlerNotJs)
    );
}

#[test]
fn ts_non_null_call_handlers_keep_legacy_raw_shape() {
    assert_shipped_parity(r#"<button @click="payload!.click()"></button>"#);
    assert_shipped_parity(r#"<div @keyup.d="documentation!.$el.click()"></div>"#);
    assert_shipped_parity(r#"<div @contextmenu.prevent="options!.tippy?.show()"></div>"#);
}

#[test]
fn ts_non_null_assignment_handlers_keep_legacy_raw_shape() {
    assert_shipped_parity(
        r#"<button @click="draft.params.poll!.expiresIn = expiresInOption.seconds"></button>"#,
    );
}

#[test]
fn branch_root_uppercase_native_events_keep_legacy_key_spelling() {
    assert_shipped_parity(r#"<div v-if="ok" @mouseMoved="mouseMoved($event)"></div>"#);
    assert_shipped_parity(r#"<div v-if="ok" @customEvent="h"></div>"#);
}
