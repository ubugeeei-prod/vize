//! TS-16 for `vue.cloak` (P2-11): Vue's `v-cloak` DOM cloak marker as a
//! dialect binding, parseable and mirrorable like the other S2 ops.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_s2::folio::{DisegnoFolio, FolioBinding, FolioElement, FolioOp, FolioVueCloak};
use vize_s2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueCloakOp};

const CANONICAL: &str = "\
[disegno]
ops=2

[disegno.ops]
ui.element div @0:20
  vue.cloak @5:12

";

fn hand_built() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![FolioBinding::VueCloak(FolioVueCloak {
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
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses"),
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
        DisegnoFolio::of(&ops)
            .print_to_string(FolioMode::Full)
            .as_str(),
        CANONICAL
    );
}
