//! TS-16 for `vue.cloak` (P2-11): Vue's `v-cloak` DOM cloak marker as a
//! dialect binding, parseable and mirrorable like the other L2 ops.

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_l2::dump::{
    Binding as DumpBinding, Element as DumpElement, Op as DumpOp, Page as L2Page,
    VueCloak as DumpVueCloak,
};
use vize_l2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueCloakOp};

const CANONICAL: &str = "\
[l2-dump-v2]
ops=2

[l2-dump-v2.ops]
ui.element div @0:20
  vue.cloak @5:12

";

fn hand_built() -> L2Page {
    L2Page {
        ops: vec![DumpOp::Element(DumpElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![DumpBinding::VueCloak(DumpVueCloak {
                span: Span::new(5, 12),
            })],
            children: vec![],
            span: Span::new(0, 20),
        })],
    }
}

#[test]
fn the_cloak_op_round_trips() {
    let value = hand_built();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(
        L2Page::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_cloak_op() {
    let arena = Allocator::default();
    let allocator = &arena;
    let ops = ArenaVec::from_iter_in(
        [Op::Element(Box::new_in(
            ElementOp {
                tag: "div",
                namespace: Namespace::Html,
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::from_iter_in(
                    [BindingOp::VueCloak(Box::new_in(
                        VueCloakOp {
                            span: Span::new(5, 12),
                        },
                        &allocator,
                    ))],
                    &allocator,
                ),
                children: Region {
                    ops: ArenaVec::new_in(&allocator),
                },
                span: Span::new(0, 20),
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
