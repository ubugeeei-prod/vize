//! TS-16 for `vue.once` / `vue.memo` (P2-11): Vue one-shot and
//! dependency-memoized render as dialect bindings. Split from
//! `dump_laws.rs` so the Vue 3 family pin stays inside the source
//! budget.

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_l2::dump::{
    Binding as DumpBinding, Element as DumpElement, Expr as DumpExpr, Op as DumpOp, Page as L2Page,
    VueMemo as DumpVueMemo, VueOnce as DumpVueOnce,
};
use vize_l2::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use vize_l2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueMemoOp, VueOnceOp};

const CANONICAL: &str = "\
[l2-dump-v2]
ops=4

[l2-dump-v2.ops]
ui.element div @0:50
  vue.once @0:7
  vue.memo value=js(\"[id]\" @16:20) @8:21
  vue.memo value=opaque(parse-rejected \"%\" @30:31) @22:36

";

fn hand_built() -> L2Page {
    L2Page {
        ops: vec![DumpOp::Element(DumpElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![
                DumpBinding::VueOnce(DumpVueOnce {
                    span: Span::new(0, 7),
                }),
                DumpBinding::VueMemo(DumpVueMemo {
                    value: DumpExpr::Js {
                        source: String::from("[id]"),
                        span: Span::new(16, 20),
                    },
                    span: Span::new(8, 21),
                }),
                DumpBinding::VueMemo(DumpVueMemo {
                    value: DumpExpr::Opaque {
                        reason: OpaqueReason::ParseRejected,
                        source: String::from("%"),
                        span: Span::new(30, 31),
                    },
                    span: Span::new(22, 36),
                }),
            ],
            children: vec![],
            span: Span::new(0, 50),
        })],
    }
}

#[test]
fn the_once_and_memo_ops_round_trip() {
    let value = hand_built();
    assert_eq!(value.op_count(), 4);
    assert_eq!(value.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(
        L2Page::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_once_and_memo_ops() {
    let arena = Allocator::default();
    let allocator = &arena;
    let admitted =
        ExprRef::Js(JsExpr::parse_in(allocator, "[id]", Span::new(16, 20)).expect("admitted"));
    let rejected = ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source: "%",
        span: Span::new(30, 31),
    }));
    let ops = ArenaVec::from_iter_in(
        [Op::Element(Box::new_in(
            ElementOp {
                tag: "div",
                namespace: Namespace::Html,
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::from_iter_in(
                    [
                        BindingOp::VueOnce(Box::new_in(
                            VueOnceOp {
                                span: Span::new(0, 7),
                            },
                            &allocator,
                        )),
                        BindingOp::VueMemo(Box::new_in(
                            VueMemoOp {
                                value: admitted,
                                span: Span::new(8, 21),
                            },
                            &allocator,
                        )),
                        BindingOp::VueMemo(Box::new_in(
                            VueMemoOp {
                                value: rejected,
                                span: Span::new(22, 36),
                            },
                            &allocator,
                        )),
                    ],
                    &allocator,
                ),
                children: Region {
                    ops: ArenaVec::new_in(&allocator),
                },
                span: Span::new(0, 50),
            },
            &allocator,
        ))],
        &allocator,
    );
    assert_eq!(
        L2Page::of(&ops)
            .expect("compatibility expression aliases")
            .print_to_string(DumpMode::Full)
            .as_str(),
        CANONICAL
    );
}
