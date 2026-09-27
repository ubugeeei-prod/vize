//! TS-16 for `vue.text` (P2-11): Vue's `v-text` text-content surface as a
//! dialect binding, parseable and mirrorable like the other L2 ops.

use vize_davinci::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_l2::dump::{
    Binding as DumpBinding, Element as DumpElement, Expr as DumpExpr, Op as DumpOp, Page as L2Page,
    VueText as DumpVueText,
};
use vize_l2::expr::{ExprRef, JsExpr};
use vize_l2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueTextOp};

const CANONICAL: &str = "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.element div @0:24
  vue.text value=js(\"raw\" @13:16) @5:17

";

const VALUE_LESS: &str = "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.element div @0:16
  vue.text @5:11

";

fn hand_built() -> L2Page {
    L2Page {
        ops: vec![DumpOp::Element(DumpElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![DumpBinding::VueText(DumpVueText {
                value: Some(DumpExpr::Js {
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

fn hand_built_value_less() -> L2Page {
    L2Page {
        ops: vec![DumpOp::Element(DumpElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![DumpBinding::VueText(DumpVueText {
                value: None,
                span: Span::new(5, 11),
            })],
            children: vec![],
            span: Span::new(0, 16),
        })],
    }
}

#[test]
fn the_text_op_round_trips() {
    let value = hand_built();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(
        L2Page::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn the_value_less_text_op_round_trips() {
    let value = hand_built_value_less();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(DumpMode::Full).as_str(), VALUE_LESS);
    assert_eq!(
        L2Page::parse(VALUE_LESS).expect("value-less text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_text_op() {
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
                    [BindingOp::VueText(Box::new_in(
                        VueTextOp {
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
        L2Page::of(&ops).print_to_string(DumpMode::Full).as_str(),
        CANONICAL
    );
}
