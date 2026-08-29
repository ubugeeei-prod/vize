//! TS-16 for `vue.show` (P2-11): Vue's `v-show` display toggle as a
//! dialect binding, parseable and mirrorable like the other S2 ops.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_s2::expr::{ExprRef, JsExpr};
use vize_s2::folio::{DisegnoFolio, FolioBinding, FolioElement, FolioExpr, FolioOp, FolioVueShow};
use vize_s2::op::{BindingOp, ElementOp, Namespace, Op, Region, VueShowOp};

const CANONICAL: &str = "\
[disegno]
ops=2

[disegno.ops]
ui.element div @0:25
  vue.show value=js(\"open\" @13:17) @5:18

";

fn hand_built() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("div"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![FolioBinding::VueShow(FolioVueShow {
                value: FolioExpr::Js {
                    source: String::from("open"),
                    span: Span::new(13, 17),
                },
                span: Span::new(5, 18),
            })],
            children: vec![],
            span: Span::new(0, 25),
        })],
    }
}

#[test]
fn the_show_op_round_trips() {
    let value = hand_built();
    assert_eq!(value.op_count(), 2);
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_show_op() {
    let arena = Allocator::default();
    let allocator = &arena;
    let open =
        ExprRef::Js(JsExpr::parse_in(allocator, "open", Span::new(13, 17)).expect("admitted"));
    let ops = ArenaVec::from_iter_in(
        [Op::Element(Box::new_in(
            ElementOp {
                tag: "div",
                namespace: Namespace::Html,
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::from_iter_in(
                    [BindingOp::VueShow(Box::new_in(
                        VueShowOp {
                            value: open,
                            span: Span::new(5, 18),
                        },
                        &allocator,
                    ))],
                    &allocator,
                ),
                children: Region {
                    ops: ArenaVec::new_in(&allocator),
                },
                span: Span::new(0, 25),
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
