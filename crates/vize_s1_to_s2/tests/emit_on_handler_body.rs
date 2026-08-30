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
