//! `ui.and` (#13131): `left && <region>` whose falsy `left` renders as a child
//! value, parseable and mirrorable like the other L2 ops.

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_l2::dump::{
    And as DumpAnd, Element as DumpElement, Expr as DumpExpr, Op as DumpOp, Page as L2Page,
};
use vize_l2::expr::{ExprRef, OpaqueExpr, OpaqueReason};
use vize_l2::op::{AndOp, ElementOp, Namespace, Op, Region};

const CANONICAL: &str = "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.and opaque(parse-rejected \"n\" @1:2) @0:14
  ui.element b @6:13

";

fn hand_built() -> L2Page {
    L2Page {
        ops: vec![DumpOp::And(DumpAnd {
            left: DumpExpr::Opaque {
                reason: OpaqueReason::ParseRejected,
                source: String::from("n"),
                span: Span::new(1, 2),
            },
            ops: vec![DumpOp::Element(DumpElement {
                tag: String::from("b"),
                namespace: Namespace::Html,
                attributes: vec![],
                bindings: vec![],
                children: vec![],
                span: Span::new(6, 13),
            })],
            span: Span::new(0, 14),
        })],
    }
}

#[test]
fn the_and_op_round_trips() {
    let value = hand_built();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(
        L2Page::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_and_op() {
    let arena = Allocator::default();
    let allocator = &arena;
    let left = ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source: "n",
        span: Span::new(1, 2),
    }));
    let element = Op::Element(Box::new_in(
        ElementOp {
            tag: "b",
            namespace: Namespace::Html,
            attributes: ArenaVec::new_in(&allocator),
            bindings: ArenaVec::new_in(&allocator),
            children: Region {
                ops: ArenaVec::new_in(&allocator),
            },
            span: Span::new(6, 13),
        },
        &allocator,
    ));
    let ops = ArenaVec::from_iter_in(
        [Op::And(Box::new_in(
            AndOp {
                left,
                region: Region {
                    ops: ArenaVec::from_iter_in([element], &allocator),
                },
                span: Span::new(0, 14),
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

#[test]
fn a_branch_under_ui_and_is_rejected() {
    let text = "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.and opaque(parse-rejected \"n\" @1:2) @0:14
  branch @6:13

";
    assert!(L2Page::parse(text).is_err());
}
