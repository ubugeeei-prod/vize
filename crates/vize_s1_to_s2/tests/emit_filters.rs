//! Vue 2 pipe-filter emission pins for the S2 DOM backend.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use support::with_transformed_caps;
use vize_s0::Allocator;
use vize_s0::config::VueVersion;
use vize_s1_to_s2::{LegacyCaps, emit_dom, emit_dom_source_with_caps};

fn vue2() -> LegacyCaps {
    LegacyCaps::for_version(VueVersion::V2)
}

fn assembled(source: &str) -> String {
    with_transformed_caps(source, vue2(), |lowered, _folio, facts, _budget| {
        emit_dom(lowered, facts)
            .unwrap_or_else(|error| panic!("emit refused {source:?}: {error:?}"))
            .assembled()
            .to_string()
    })
}

#[test]
fn vue2_resolves_filters_once_in_first_seen_order() {
    assert_eq!(
        assembled(
            "<div><span>{{ msg | cap | trim(arg) }}</span><span>{{ other | cap }}</span></div>"
        ),
        concat!(
            "const { resolveFilter: _resolveFilter, toDisplayString: _toDisplayString, ",
            "createElementVNode: _createElementVNode, openBlock: _openBlock, ",
            "createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  const _filter_cap = _resolveFilter(\"cap\")\n",
            "  const _filter_trim = _resolveFilter(\"trim\")\n",
            "  \n",
            "  return (_openBlock(), _createElementBlock(\"div\", null, [\n",
            "    _createElementVNode(\"span\", null, ",
            "_toDisplayString(_filter_trim(_filter_cap(msg),arg)), 1 /* TEXT */),\n",
            "    _createElementVNode(\"span\", null, ",
            "_toDisplayString(_filter_cap(other)), 1 /* TEXT */)\n",
            "  ]))\n",
            "}"
        )
    );
}

#[test]
fn vue2_filter_names_use_the_shipped_asset_identifier_rule() {
    assert_eq!(
        assembled("<div>{{ x | foo-bar | $cash }}</div>"),
        concat!(
            "const { resolveFilter: _resolveFilter, toDisplayString: _toDisplayString, ",
            "openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue\n",
            "\n",
            "function render(_ctx, _cache, $props, $setup, $data, $options) {\n",
            "  const _filter_foo_bar = _resolveFilter(\"foo-bar\")\n",
            "  const _filter_36cash = _resolveFilter(\"$cash\")\n",
            "  \n",
            "  return (_openBlock(), _createElementBlock(\"div\", null, ",
            "_toDisplayString(_filter_36cash(_filter_foo_bar(x))), 1 /* TEXT */))\n",
            "}"
        )
    );
}

#[test]
fn vue3_pipe_expressions_do_not_resolve_filters() {
    let allocator = Allocator::new();
    let out = emit_dom_source_with_caps(&allocator, "<div>{{ msg | cap }}</div>", LegacyCaps::VUE3)
        .expect("emit")
        .assembled();
    assert_eq!(
        out.as_str(),
        "\
const { toDisplayString: _toDisplayString, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(msg | cap), 1 /* TEXT */))
}"
    );
}
