//! P2-10 follow-up: keep the ricalco CSS `v-bind()` scanner byte-for-byte
//! aligned with the shipped SFC extractor while the scanner remains local.

#![allow(clippy::disallowed_macros, clippy::disallowed_methods)]

use vize_atelier_sfc::style::extract_css_vars;
use vize_croquis::drawer::extract_identifier_refs_oxc;
use vize_s0::{Allocator, String, ToCompactString};
use vize_s1_to_s2::lower_style_block;
use vize_s2::op::{BindingOp, Op};

#[derive(Debug, Eq, PartialEq)]
struct LoweredCssBind {
    source: String,
    call_span: (u32, u32),
    value_span: (u32, u32),
    refs: Vec<(String, u32)>,
}

fn lowered_css_binds(css: &str, block_start: u32) -> Vec<LoweredCssBind> {
    let allocator = Allocator::default();
    let op = lower_style_block(&allocator, css, block_start);
    let Op::Element(element) = &op else {
        panic!("style block lowering must emit the style carrier");
    };
    element
        .bindings
        .iter()
        .map(|binding| {
            let BindingOp::VueCssBind(bind) = binding else {
                panic!("style carrier must contain only vue.css-bind bindings");
            };
            let source = bind.value.source();
            let value_span = bind.value.span();
            LoweredCssBind {
                source: source.to_compact_string(),
                call_span: (bind.span.start, bind.span.end),
                value_span: (value_span.start, value_span.end),
                refs: identifier_refs(source),
            }
        })
        .collect()
}

fn identifier_refs(source: &str) -> Vec<(String, u32)> {
    extract_identifier_refs_oxc(source)
        .into_iter()
        .map(|reference| (reference.name, reference.offset))
        .collect()
}

fn assert_css_bind_parity(
    css: &str,
    block_start: u32,
    expected_sources: &[&str],
) -> Vec<LoweredCssBind> {
    let shipped = extract_css_vars(css);
    let shipped_sources = shipped
        .iter()
        .map(|source| source.as_str())
        .collect::<Vec<_>>();
    assert_eq!(shipped_sources, expected_sources);
    let shipped_refs = shipped
        .iter()
        .map(|source| identifier_refs(source))
        .collect::<Vec<_>>();

    let lowered = lowered_css_binds(css, block_start);
    assert_eq!(
        lowered
            .iter()
            .map(|binding| binding.source.as_str())
            .collect::<Vec<_>>(),
        shipped_sources,
    );
    assert_eq!(
        lowered
            .iter()
            .map(|binding| &binding.refs)
            .collect::<Vec<_>>(),
        shipped_refs.iter().collect::<Vec<_>>(),
    );
    lowered
}

#[test]
fn matches_shipped_order_quotes_and_duplicates() {
    assert_css_bind_parity(
        ".foo { color: v-bind(color); background: v-bind('bgColor'); border-color: v-bind(color); height: v-bind(\"height + 'px'\"); }",
        0,
        &["color", "bgColor", "color", "height + 'px'"],
    );
}

#[test]
fn matches_shipped_strings_comments_prefixes_and_nested_parentheses() {
    assert_css_bind_parity(
        r#"
.icon::before {
  content: "v-bind(icon)";
  color: v-bind(color /* keep ) inside comments */);
}
/* background: v-bind(bg); */
// width: v-bind(width);
.label { background: 'v-bind(bg)'; }
.header { background-color: color(from v-bind("parentBg ?? 'var(--bg)'") srgb r g b / 0.85); }
.textCountGraph {
  background-image: conic-gradient(
    var(--countColor) 0% v-bind("Math.min(100, textCountPercentage) + '%'"),
    rgba(0, 0, 0, .2) v-bind("Math.min(100, textCountPercentage) + '%'") 100%
  );
}
.foo { transition: my-v-bind(x); animation: -webkit-v-bind(y); }
"#,
        37,
        &[
            "color /* keep ) inside comments */",
            "parentBg ?? 'var(--bg)'",
            "Math.min(100, textCountPercentage) + '%'",
            "Math.min(100, textCountPercentage) + '%'",
        ],
    );
}

#[test]
fn unmatched_call_stops_after_the_last_complete_shipped_hit() {
    assert_css_bind_parity(
        ".ok { color: v-bind(color); } .broken { width: v-bind(width",
        0,
        &["color"],
    );
}

#[test]
fn spans_remain_file_absolute_while_sources_match_the_shipped_extractor() {
    let bindings = assert_css_bind_parity(
        ".foo { color: v-bind(color); background: v-bind('bgColor'); }",
        128,
        &["color", "bgColor"],
    );
    assert_eq!(bindings[0].call_span, (142, 155));
    assert_eq!(bindings[0].value_span, (149, 154));
    assert_eq!(bindings[1].call_span, (169, 186));
    assert_eq!(bindings[1].value_span, (177, 184));
}
