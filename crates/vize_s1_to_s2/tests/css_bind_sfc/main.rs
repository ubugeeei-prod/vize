//! P2-10 acceptance: a committed `v-bind()`-bearing SFC whose S2 folio
//! pins the style ops beside the template tree.

mod authored;

use authored::assert_authored_artifact;
use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, SourceBlock, SourceRoot};
use vize_s1::parse;
use vize_s1_to_s2::lower_source_block;
use vize_s2::folio::DisegnoFolio;
use vize_s2::op::Op;
use vize_s2::verify::{Rigor, Violation, verify};

const SOURCE: &str = include_str!("../fixtures/css_bind.vue");

const CANONICAL: &str = "\
[disegno]
ops=4

[disegno.ops]
ui.element p @11:41
  attr class=\"foo\" @14:25
  ui.interpolation js(\"color\" @29:34) @26:37
ui.element style @61:93
  vue.css-bind value=js(\"color\" @83:88) @76:89

";

fn block_between<'a>(source: &'a str, open: &str, close: &str) -> (&'a str, u32) {
    let tag = source.find(open).expect("opening tag");
    let inner = tag + open.len();
    let rel = source[inner..].find(close).expect("closing tag");
    (&source[inner..inner + rel], inner as u32)
}

#[test]
fn the_sfc_fixture_folio_pins_template_and_css_bind() {
    let allocator = Allocator::default();
    let root = SourceRoot::new(SOURCE).expect("fixture source");
    let (template, template_start) = block_between(SOURCE, "<template>", "</template>");
    let (css, css_start) = block_between(SOURCE, "<style>", "</style>");
    let template_block = root
        .block(template, template_start)
        .expect("template block is a source slice");
    let css_block = root
        .block(css, css_start)
        .expect("style block is a source slice");
    let (tree, errors) = parse(&allocator, template);
    let mut lowered = lower_source_block(&allocator, &tree, &errors, template_block);
    lowered.push_style_block_in(&allocator, css_block);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    assert_eq!(u64::from(lowered.op_count), folio.op_count());
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(verify(&folio, Rigor::Raw), Vec::<Violation>::new());
    let provenance: Vec<(&str, u32, u32, &str)> = lowered
        .provenance
        .iter()
        .map(|record| {
            (
                record.rule.as_str(),
                record.span.start,
                record.span.end,
                record.before.as_str(),
            )
        })
        .collect();
    assert_eq!(
        provenance,
        [
            ("condense.drop-whitespace", 10, 11, "\n"),
            ("lower.element", 11, 41, "<p class=\"foo\">"),
            ("lower.interpolation", 26, 37, " color "),
            ("condense.drop-whitespace", 41, 42, "\n"),
        ],
        "provenance rules, authored spans, and consumed text must pin exactly"
    );
    assert_authored_artifact(SOURCE, &lowered);
}

#[test]
fn unicode_prefix_does_not_shift_authored_spans() {
    let source = "\u{e9}// prefix\n<template>\n<p>{{ color }}</p>\n</template>\n<style>\n.foo{color:v-bind(color)}\n</style>\n";
    let allocator = Allocator::default();
    let root = SourceRoot::new(source).expect("fixture source");
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

    let folio = DisegnoFolio::of(&lowered.root.ops);
    assert_eq!(verify(&folio, Rigor::Raw), Vec::<Violation>::new());
    assert_authored_artifact(source, &lowered);
    assert!(lowered.root.ops.iter().any(|op| match op {
        Op::Element(element) if element.tag == "style" => {
            element.span.start == css_start && element.span.end == css_start + css.len() as u32
        }
        _ => false,
    }));
}

#[test]
fn duplicate_style_blocks_keep_their_source_identity() {
    let source = "<template><p>{{ color }}</p></template><style>.foo{color:v-bind(color)}</style><style>.foo{color:v-bind(color)}</style>";
    let allocator = Allocator::default();
    let root = SourceRoot::new(source).expect("fixture source");
    let (template, template_start) = block_between(source, "<template>", "</template>");
    let first_css_start = source.find(".foo").expect("first style") as u32;
    let second_css_start = source.rfind(".foo").expect("second style") as u32;
    let (_, second_block) = style_block_at(root, source, second_css_start);
    let (tree, errors) = parse(&allocator, template);
    let mut lowered = lower_source_block(
        &allocator,
        &tree,
        &errors,
        root.block(template, template_start)
            .expect("template block is a source slice"),
    );
    lowered.push_style_block_in(&allocator, second_block);

    let style = lowered
        .root
        .ops
        .iter()
        .find_map(|op| match op {
            Op::Element(element) if element.tag == "style" => Some(element),
            _ => None,
        })
        .expect("style carrier");
    assert_eq!(style.span.start, second_css_start);
    assert_ne!(style.span.start, first_css_start);
    assert_authored_artifact(source, &lowered);
}

fn style_block_at<'a>(
    root: SourceRoot<'a>,
    source: &'a str,
    start: u32,
) -> (&'a str, SourceBlock<'a>) {
    let start = start as usize;
    let end = start + source[start..].find("</style>").expect("style close");
    let css = &source[start..end];
    let block = root.block(css, start as u32).expect("style block");
    (css, block)
}
