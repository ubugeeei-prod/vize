//! P2-10: style-block `v-bind()` lowers to `vue.css-bind` on a carrier
//! `ui.element style`. Spans are file-absolute.

use vize_davinci::folio::{Folio, FolioMode};
use vize_ricalco::{lower_style_block, lower_style_block_in};
use vize_s0::{Allocator, SourceFrameError, SourceRoot};
use vize_s2::folio::{DisegnoFolio, FolioBinding, FolioElement, FolioExpr, FolioOp};
use vize_s2::op::Op;

fn folio(css: &str, block_start: u32) -> DisegnoFolio {
    let arena = Allocator::default();
    let op = lower_style_block(&arena, css, block_start);
    DisegnoFolio::of(core::slice::from_ref(&op))
}

#[test]
fn two_calls_round_trip_on_the_carrier() {
    let css = ".foo { color: v-bind(color); background: v-bind('bgColor'); }";
    let value = folio(css, 0);
    assert_eq!(value.op_count(), 3);
    assert_eq!(
        value.print_to_string(FolioMode::Full).as_str(),
        "\
[disegno]
ops=3

[disegno.ops]
ui.element style @0:61
  vue.css-bind value=js(\"color\" @21:26) @14:27
  vue.css-bind value=js(\"bgColor\" @49:56) @41:58

"
    );
}

#[test]
fn block_start_produces_file_absolute_spans() {
    let css = ".foo { color: v-bind(color); }";
    let shifted = folio(css, 90);
    assert_eq!(
        shifted.print_to_string(FolioMode::Full).as_str(),
        "\
[disegno]
ops=2

[disegno.ops]
ui.element style @90:120
  vue.css-bind value=js(\"color\" @111:116) @104:117

"
    );
    let FolioOp::Element(FolioElement { bindings, span, .. }) = &folio(css, 90).ops[0] else {
        panic!("carrier is ui.element");
    };
    assert_eq!(span.start, 90);
    let FolioBinding::VueCssBind(bind) = &bindings[0] else {
        panic!("first binding is vue.css-bind");
    };
    assert_eq!(bind.span.start, 104);
    assert_eq!(bind.span.end, 117);
    let FolioExpr::Js { span, source } = &bind.value else {
        panic!("color is admitted js");
    };
    assert_eq!(source.as_str(), "color");
    assert_eq!((span.start, span.end), (111, 116));
}

#[test]
fn validated_blocks_keep_equal_style_blocks_distinct() {
    let source = "<style>.foo{color:v-bind(color)}</style><style>.foo{color:v-bind(color)}</style>";
    let first_start = source.find(".foo").expect("first style content");
    let second_start = source.rfind(".foo").expect("second style content");
    let first_end = first_start + source[first_start..].find("</style>").expect("first close");
    let second_end = second_start
        + source[second_start..]
            .find("</style>")
            .expect("second close");
    let first_css = &source[first_start..first_end];
    let second_css = &source[second_start..second_end];
    let root = SourceRoot::new(source).expect("small source");
    let second_block = root
        .block(second_css, second_start as u32)
        .expect("second style block");
    assert_eq!(
        root.block(first_css, second_start as u32),
        Err(SourceFrameError::BlockNotRootSlice)
    );

    let arena = Allocator::default();
    let op = lower_style_block_in(&arena, second_block);
    let FolioOp::Element(FolioElement { bindings, span, .. }) =
        &DisegnoFolio::of(core::slice::from_ref(&op)).ops[0]
    else {
        panic!("carrier is ui.element");
    };
    assert_eq!(span.start, second_start as u32);
    let FolioBinding::VueCssBind(bind) = &bindings[0] else {
        panic!("first binding is vue.css-bind");
    };
    assert_eq!(bind.span.start, second_start as u32 + 11);
    let FolioExpr::Js { span, source } = &bind.value else {
        panic!("color is admitted js");
    };
    assert_eq!(source.as_str(), "color");
    assert_eq!(
        (span.start, span.end),
        (second_start as u32 + 18, second_start as u32 + 23)
    );
}

#[test]
fn strings_comments_and_prefixed_names_are_not_calls() {
    let css = r#"
.icon::before {
  content: "v-bind(icon)";
  color: v-bind(color /* keep ) inside comments */);
}
/* background: v-bind(bg); */
// width: v-bind(width);
.label { background: 'v-bind(bg)'; }
.foo { transition: my-v-bind(x); animation: -webkit-v-bind(y); }
"#;
    let FolioOp::Element(FolioElement { bindings, .. }) = &folio(css, 0).ops[0] else {
        panic!("carrier is ui.element");
    };
    assert_eq!(bindings.len(), 1);
    let FolioBinding::VueCssBind(bind) = &bindings[0] else {
        panic!("only real v-bind(color)");
    };
    // Shipped extractor keeps the comment in the var text.
    let FolioExpr::Opaque { reason, source, .. } = &bind.value else {
        panic!("comment-bearing argument is not one JS expression");
    };
    assert_eq!(source.as_str(), "color /* keep ) inside comments */");
    assert_eq!(*reason, vize_s2::expr::OpaqueReason::ParseRejected);
}

#[test]
fn quoted_expressions_keep_inner_parentheses() {
    let css = r#".header { background: v-bind("parentBg ?? 'var(--bg)'"); }"#;
    assert_eq!(
        folio(css, 0).print_to_string(FolioMode::Full).as_str(),
        "\
[disegno]
ops=2

[disegno.ops]
ui.element style @0:58
  vue.css-bind value=js(\"parentBg ?? 'var(--bg)'\" @30:53) @22:55

"
    );
}

#[test]
fn an_empty_block_is_a_carrier_with_no_binds() {
    let arena = Allocator::default();
    let op = lower_style_block(&arena, "/* no binds */", 0);
    let Op::Element(element) = &op else {
        panic!("carrier");
    };
    assert!(element.bindings.is_empty());
    assert_eq!(DisegnoFolio::of(core::slice::from_ref(&op)).op_count(), 1);
}
