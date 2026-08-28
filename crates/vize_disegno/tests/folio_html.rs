//! TS-16 for `vue.html` (P2-11): Vue's `v-html` raw-HTML surface as a
//! dialect binding, parseable and mirrorable like the other S2 ops.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::folio::{DisegnoFolio, FolioBinding, FolioElement, FolioExpr, FolioOp, FolioVueHtml};
use vize_s2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueHtmlOp};

const CANONICAL: &str = "\
[disegno]
ops=2

[disegno.ops]
ui.element div @0:24
  vue.html value=js(\"raw\" @13:16) @5:17

";

const VALUE_LESS: &str = "\
[disegno]
ops=2

[disegno.ops]
ui.element div @0:16
  vue.html @5:11

";

fn hand_built() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![FolioBinding::VueHtml(FolioVueHtml {
                value: Some(FolioExpr::Js {
                    source: String::from("raw"),
                    span: Span::new(13, 16),
                }),
                span: Span::new(5, 17),
            })],
            children: vec![],
            span: Span::new(0, 24),
        })],
    }
}

fn hand_built_value_less() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![FolioBinding::VueHtml(FolioVueHtml {
                value: None,
                span: Span::new(5, 11),
            })],
            children: vec![],
            span: Span::new(0, 16),
        })],
    }
}

#[test]
fn the_html_op_round_trips() {
    let value = hand_built();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn the_value_less_html_op_round_trips() {
    let value = hand_built_value_less();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), VALUE_LESS);
    assert_eq!(
        DisegnoFolio::parse(VALUE_LESS).expect("value-less text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_html_op() {
    let arena = Allocator::default();
    let allocator = &arena;
    let raw = ExprRef::Js(JsExpr::parse_in(allocator, "raw", Span::new(13, 16)).expect("admitted"));
    let ops = ArenaVec::from_iter_in(
        [Op::Element(Box::new_in(
            ElementOp {
                tag: "div",
                namespace: Namespace::Html,
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::from_iter_in(
                    [BindingOp::VueHtml(Box::new_in(
                        VueHtmlOp {
                            value: Some(raw),
                            span: Span::new(5, 17),
                        },
                        &allocator,
                    ))],
                    &allocator,
                ),
                children: Region {
                    ops: ArenaVec::new_in(&allocator),
                },
                span: Span::new(0, 24),
            },
            &allocator,
        ))],
        &allocator,
    );
    assert_eq!(
        DisegnoFolio::of(&ops)
            .print_to_string(FolioMode::Full)
            .as_str(),
        CANONICAL
    );
}
