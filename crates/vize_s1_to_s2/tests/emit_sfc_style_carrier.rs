//! P2-11: SFC style-block carriers are facts for consumers, not DOM output.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

mod support;

use support::{with_lowered, with_transformed};
use vize_davinci::pass::BudgetObserver;
use vize_s0::{Allocator, SourceRoot};
use vize_s1::parse;
use vize_s1_to_s2::{emit_dom, emit_dom_source, lower_source_block};

fn block_between<'a>(source: &'a str, open: &str, close: &str) -> (&'a str, u32) {
    let tag = source.find(open).expect("opening tag");
    let inner = tag + open.len();
    let rel = source[inner..].find(close).expect("closing tag");
    (&source[inner..inner + rel], inner as u32)
}

fn style_carrier_emit(source: &str) -> String {
    let allocator = Allocator::new();
    let root = SourceRoot::new(source).expect("source is small");
    let (template, template_start) = block_between(source, "<template>", "</template>");
    let (css, css_start) = block_between(source, "<style>", "</style>");
    let template_block = root
        .block(template, template_start)
        .expect("template block is a source slice");
    let css_block = root
        .block(css, css_start)
        .expect("style block is a source slice");
    let (tree, errors) = parse(&allocator, template);
    let mut lowered = lower_source_block(&allocator, &tree, &errors, template_block);
    lowered.push_style_block_in(&allocator, css_block);
    let mut budget = BudgetObserver::new();
    let facts = vize_s1_to_s2::pass::run_transform(&mut lowered, &mut budget);
    emit_dom(&lowered, &facts)
        .expect("style carrier is skipped by DOM emit")
        .assembled()
        .to_string()
}

fn template_emit(template: &str) -> String {
    let allocator = Allocator::new();
    emit_dom_source(&allocator, template)
        .expect("template emits")
        .assembled()
        .to_string()
}

#[test]
fn style_v_bind_carrier_does_not_become_a_second_root() {
    let source = "<template><p>{{ color }}</p></template><style>.foo{color:v-bind(color)}</style>";
    assert_eq!(
        style_carrier_emit(source),
        template_emit("<p>{{ color }}</p>")
    );
}

#[test]
fn style_v_bind_carrier_does_not_make_multi_root_fragments_wider() {
    let source = "<template><p>A</p><p>B</p></template><style>.foo{color:v-bind(color)}</style>";
    assert_eq!(
        style_carrier_emit(source),
        template_emit("<p>A</p><p>B</p>")
    );
}

#[test]
fn real_template_style_elements_still_emit() {
    with_transformed("<style></style>", |lowered, _, facts, _| {
        assert_eq!(
            emit_dom(lowered, facts)
                .expect("template style emits")
                .assembled(),
            "\
const { openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock(\"style\"))
}"
        );
    });
}

#[test]
fn the_skip_preserves_later_direct_template_emitters() {
    with_lowered("<style></style>", |lowered, _| {
        assert_eq!(lowered.op_count, 1);
    });
    let source = "<template></template><style>.foo{color:v-bind(color)}</style>";
    assert_eq!(
        style_carrier_emit(source),
        "\
\n\
function render(_ctx, _cache, $props, $setup, $data, $options) {
  return null
}"
    );
}
