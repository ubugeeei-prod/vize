//! TS-16 for the Vue 2 dialect ops (P2-9 installment 7): `vue.sync`,
//! `vue.slot-scope`, and `vue.filter` round-trip through the folio.
//! Split from `dump_laws.rs` so the original reference page stays the
//! Vue 3 family pin and this file stays inside the source budget.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, Box, Span, String, Vec as ArenaVec};
use vize_l2::dump::{
    Binding as DumpBinding, Component as DumpComponent, Expr as DumpExpr,
    Interpolation as DumpInterpolation, Op as DumpOp, Page as L2Page,
    VueSlotScope as DumpVueSlotScope, VueSync as DumpVueSync,
};
use vize_l2::expr::{ExprRef, JsExpr, VueFilterExpr};
use vize_l2::op::{BindingOp, ComponentOp, InterpolationOp, Op, Region, VueSlotScopeOp, VueSyncOp};

const CANONICAL: &str = "\
[l2-dump-v2]
ops=4

[l2-dump-v2.ops]
ui.component Card @0:40
  vue.sync name=\"title\" mods=\"camel\" value=js(\"heading\" @10:17) @0:18
  vue.slot-scope name=\"header\" params=js(\"props\" @20:25) @19:26
  ui.interpolation vue.filter(\"msg | cap\" @28:37) @27:38

";

fn js(source: &str, start: u32, end: u32) -> DumpExpr {
    DumpExpr::Js {
        source: String::from(source),
        span: Span::new(start, end),
    }
}

fn hand_built() -> L2Page {
    L2Page {
        ops: vec![DumpOp::Component(DumpComponent {
            name: String::from("Card"),
            attributes: vec![],
            bindings: vec![
                DumpBinding::VueSync(DumpVueSync {
                    name: String::from("title"),
                    modifiers: vec![String::from("camel")],
                    value: js("heading", 10, 17),
                    span: Span::new(0, 18),
                }),
                DumpBinding::VueSlotScope(DumpVueSlotScope {
                    name: Some(String::from("header")),
                    params: Some(js("props", 20, 25)),
                    span: Span::new(19, 26),
                }),
            ],
            children: vec![DumpOp::Interpolation(DumpInterpolation {
                expression: DumpExpr::Filter {
                    source: String::from("msg | cap"),
                    span: Span::new(28, 37),
                },
                span: Span::new(27, 38),
            })],
            span: Span::new(0, 40),
        })],
    }
}

fn arena_js<'a>(allocator: &'a Allocator, source: &'a str, start: u32, end: u32) -> ExprRef<'a> {
    ExprRef::Js(JsExpr::parse_in(allocator, source, Span::new(start, end)).expect("admitted"))
}

#[test]
fn the_legacy_dialect_ops_round_trip() {
    let value = hand_built();
    assert_eq!(value.op_count(), 4);
    assert_eq!(value.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(
        L2Page::parse(CANONICAL).expect("canonical text parses"),
        value
    );
}

#[test]
fn an_arena_tree_mirrors_the_legacy_dialect_ops() {
    let arena = Allocator::default();
    let allocator = &arena;
    let heading = arena_js(allocator, "heading", 10, 17);
    let props = arena_js(allocator, "props", 20, 25);
    let filter = VueFilterExpr::parse_in(allocator, "msg | cap", Span::new(28, 37))
        .expect("a filter chain is admitted");
    let ops = ArenaVec::from_iter_in(
        [Op::Component(Box::new_in(
            ComponentOp {
                name: "Card",
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::from_iter_in(
                    [
                        BindingOp::VueSync(Box::new_in(
                            VueSyncOp {
                                name: "title",
                                modifiers: ArenaVec::from_iter_in(["camel"], &allocator),
                                value: heading,
                                span: Span::new(0, 18),
                            },
                            &allocator,
                        )),
                        BindingOp::VueSlotScope(Box::new_in(
                            VueSlotScopeOp {
                                name: Some("header"),
                                params: Some(props),
                                span: Span::new(19, 26),
                            },
                            &allocator,
                        )),
                    ],
                    &allocator,
                ),
                children: Region {
                    ops: ArenaVec::from_iter_in(
                        [Op::Interpolation(Box::new_in(
                            InterpolationOp {
                                expression: ExprRef::Filter(filter),
                                span: Span::new(27, 38),
                            },
                            &allocator,
                        ))],
                        &allocator,
                    ),
                },
                span: Span::new(0, 40),
            },
            &allocator,
        ))],
        &allocator,
    );
    let mirrored = L2Page::of(&ops).expect("compatibility expression aliases");
    assert_eq!(mirrored.print_to_string(DumpMode::Full).as_str(), CANONICAL);
    assert_eq!(mirrored, hand_built());
}
