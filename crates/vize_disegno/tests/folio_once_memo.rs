//! TS-16 for `vue.once` / `vue.memo` (P2-11): Vue one-shot and
//! dependency-memoized render as dialect bindings. Split from
//! `folio_laws.rs` so the Vue 3 family pin stays inside the source
//! budget.

use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use vize_disegno::folio::{
    DisegnoFolio, FolioBinding, FolioElement, FolioExpr, FolioOp, FolioVueMemo, FolioVueOnce,
};
use vize_disegno::op::{BindingOp, ElementOp, Namespace, Op, Region, VueMemoOp, VueOnceOp};
use vize_s0::{Allocator, Box, Span, String, Vec as ArenaVec};

const CANONICAL: &str = "\
[disegno]
ops=4

[disegno.ops]
ui.element div @0:50
  vue.once @0:7
  vue.memo value=js(\"[id]\" @16:20) @8:21
  vue.memo value=opaque(parse-rejected \"%\" @30:31) @22:36

";

fn hand_built() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![
                FolioBinding::VueOnce(FolioVueOnce {
                    span: Span::new(0, 7),
                }),
                FolioBinding::VueMemo(FolioVueMemo {
                    value: FolioExpr::Js {
                        source: String::from("[id]"),
                        span: Span::new(16, 20),
                    },
                    span: Span::new(8, 21),
                }),
                FolioBinding::VueMemo(FolioVueMemo {
                    value: FolioExpr::Opaque {
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
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses"),
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
        DisegnoFolio::of(&ops)
            .print_to_string(FolioMode::Full)
            .as_str(),
        CANONICAL
    );
}
