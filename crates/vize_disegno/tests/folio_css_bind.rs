//! TS-16 for `vue.css-bind` (P2-10): SFC style `v-bind()` as a dialect
//! binding. Split from `folio_laws.rs` so the Vue 3 family pin stays
//! inside the source budget.

use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use vize_disegno::folio::{
    DisegnoFolio, FolioBinding, FolioElement, FolioExpr, FolioOp, FolioVueCssBind,
};
use vize_disegno::op::{BindingOp, ElementOp, Namespace, Op, Region, VueCssBindOp};
use vize_s0::{Allocator, Box, Span, String, Vec as ArenaVec};

const CANONICAL: &str = "\
[disegno]
ops=3

[disegno.ops]
ui.element style @0:50
  vue.css-bind value=js(\"color\" @8:13) @0:15
  vue.css-bind value=opaque(parse-rejected \"%\" @20:21) @16:30

";

fn hand_built() -> DisegnoFolio {
    DisegnoFolio {
        ops: vec![FolioOp::Element(FolioElement {
            tag: String::from("style"),
            namespace: Namespace::Html,
            attributes: vec![],
            bindings: vec![
                FolioBinding::VueCssBind(FolioVueCssBind {
                    value: FolioExpr::Js {
                        source: String::from("color"),
                        span: Span::new(8, 13),
                    },
                    span: Span::new(0, 15),
                }),
                FolioBinding::VueCssBind(FolioVueCssBind {
                    value: FolioExpr::Opaque {
                        reason: OpaqueReason::ParseRejected,
                        source: String::from("%"),
                        span: Span::new(20, 21),
                    },
                    span: Span::new(16, 30),
                }),
            ],
            children: vec![],
            span: Span::new(0, 50),
        })],
    }
}

#[test]
fn the_css_bind_op_round_trips() {
    let value = hand_built();
    assert_eq!(value.op_count(), 3);
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_css_bind_op() {
    let arena = Allocator::default();
    let allocator = &arena;
    let color =
        ExprRef::Js(JsExpr::parse_in(allocator, "color", Span::new(8, 13)).expect("admitted"));
    let rejected = ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source: "%",
        span: Span::new(20, 21),
    }));
    let ops = ArenaVec::from_iter_in(
        [Op::Element(Box::new_in(
            ElementOp {
                tag: "style",
                namespace: Namespace::Html,
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::from_iter_in(
                    [
                        BindingOp::VueCssBind(Box::new_in(
                            VueCssBindOp {
                                value: color,
                                span: Span::new(0, 15),
                            },
                            &allocator,
                        )),
                        BindingOp::VueCssBind(Box::new_in(
                            VueCssBindOp {
                                value: rejected,
                                span: Span::new(16, 30),
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
