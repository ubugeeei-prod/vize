//! Event handler-body admission and emission boundaries.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking"
)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use crate::support;

use support::with_transformed;
use vize_l0::Allocator;
use vize_l4::targets::dom::artifact::{
    DomEmitOptions, UnsupportedReason as Reason, emit_dom_source, emit_dom_with_options,
};

fn assembled_with_options(source: &str, is_ts: bool) -> String {
    assembled_prefixed(source, false, is_ts)
}

fn assembled_prefixed(source: &str, prefix_identifiers: bool, is_ts: bool) -> String {
    with_transformed(source, |lowered, _folio, facts, _budget| {
        emit_dom_with_options(
            lowered,
            facts,
            &DomEmitOptions {
                prefix_identifiers,
                is_ts,
                ..DomEmitOptions::DEFAULT
            },
        )
        .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
        .assembled()
        .to_string()
    })
}

fn assembled(source: &str) -> String {
    assembled_with_options(source, false)
}

fn shipped_with_options(source: &str, is_ts: bool) -> String {
    shipped_prefixed(source, false, is_ts)
}

fn shipped_prefixed(source: &str, prefix_identifiers: bool, is_ts: bool) -> String {
    let allocator = Allocator::new();
    let (_, errors, old) = vize_atelier_dom::compile_template_legacy_with_options(
        &allocator,
        source,
        vize_atelier_dom::DomCompilerOptions {
            prefix_identifiers,
            is_ts,
            ..vize_atelier_dom::DomCompilerOptions::default()
        },
    );
    let blocking: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_compatibility_notice())
        .collect();
    assert!(blocking.is_empty(), "{source:?}: {blocking:?}");
    format!("{}\n{}", old.preamble, old.code)
}

fn shipped(source: &str) -> String {
    shipped_with_options(source, false)
}

fn assert_shipped_parity_with_options(source: &str, is_ts: bool) {
    assert_eq!(
        assembled_with_options(source, is_ts),
        shipped_with_options(source, is_ts),
        "{source}"
    );
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

/// Vue forwards an erased identifier or member as the handler itself.
/// `onPick!` and `onPick as any` used to compile to `$event => (_ctx.onPick)`,
/// which evaluates the function and never calls it.
#[test]
fn ts_erased_handler_references_are_called_like_identifiers() {
    let (_, template) = include_str!(
        "../../../tests/_fixtures/differential/compiler/typed-handler-runtime-spelling-11815/Layout.vue.txt"
    )
    .split_once("<template>")
    .expect("pinned handler SFC template start");
    let (pinned_handler, _) = template
        .split_once("</template>")
        .expect("pinned handler SFC template end");
    assert_eq!(pinned_handler, r#"<div @click="onPick as any"></div>"#);
    for prefix in [false, true] {
        let plain = assembled_prefixed(r#"<div @click="onPick"></div>"#, prefix, true);
        let reference = if prefix { "_ctx.onPick" } else { "onPick" };
        let expected = format!(
            "const {{ openBlock: _openBlock, createElementBlock: _createElementBlock }} = Vue\n\nfunction render(_ctx, _cache, $props, $setup, $data, $options) {{\n  return (_openBlock(), _createElementBlock(\"div\", {{ onClick: {reference} }}, null, 8 /* PROPS */, [\"onClick\"]))\n}}"
        );
        assert_eq!(plain, expected, "untyped control prefix={prefix}");
        assert_eq!(
            assembled_prefixed(r#"<div @click="onPick"></div>"#, prefix, false),
            expected,
            "JavaScript control prefix={prefix}",
        );
        for source in [r#"<div @click="onPick!"></div>"#, pinned_handler] {
            let emitted = assembled_prefixed(source, prefix, true);
            assert_eq!(emitted, plain, "prefix={prefix} {source}");
            assert_eq!(
                emitted,
                shipped_prefixed(source, prefix, true),
                "prefix={prefix} {source}"
            );
        }
    }
}

#[test]
fn unchanged_js_handler_padding_keeps_legacy_layout_in_typescript_mode() {
    let source = r#"<div @click=" onPick "></div>"#;
    for prefix in [false, true] {
        let emitted = assembled_prefixed(source, prefix, true);
        assert_eq!(emitted, shipped_prefixed(source, prefix, true));
        assert_eq!(emitted, assembled_prefixed(source, prefix, false));
    }
}

#[test]
fn ts_non_null_call_handlers_keep_legacy_wrapped_shape() {
    for source in [
        r#"<button @click="payload!.click()"></button>"#,
        r#"<div @keyup.d="documentation!.$el.click()"></div>"#,
        r#"<div @contextmenu.prevent="options!.tippy?.show()"></div>"#,
    ] {
        assert_shipped_parity(source);
        assert_shipped_parity_with_options(source, true);
    }
}

#[test]
fn ts_non_null_assignment_handlers_keep_legacy_wrapped_shape() {
    let source =
        r#"<button @click="draft.params.poll!.expiresIn = expiresInOption.seconds"></button>"#;
    assert_shipped_parity(source);
    assert_shipped_parity_with_options(source, true);
}

#[test]
fn branch_root_uppercase_native_events_keep_legacy_key_spelling() {
    assert_shipped_parity(r#"<div v-if="ok" @mouseMoved="mouseMoved($event)"></div>"#);
    assert_shipped_parity(r#"<div v-if="ok" @customEvent="h"></div>"#);
}
