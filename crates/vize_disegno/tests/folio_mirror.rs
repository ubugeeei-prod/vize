//! The arena side of TS-16 (P2-5a; expression payloads P2-5b): the
//! reference tree built with the op family in a live arena - retained
//! `js` payloads parsed through the real load-path parse site
//! ([`JsExpr::parse_in`]), `opaque` and `foreign` payloads allocated by
//! hand - mirrored through [`DisegnoFolio::of`] into the committed
//! canonical page. The owned twin of this tree lives in
//! `tests/folio_laws.rs`.

use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::expr::{ExprRef, ForeignExpr, JsExpr, OpaqueExpr, OpaqueReason};
use vize_disegno::folio::DisegnoFolio;
use vize_disegno::op::{
    Attribute, BindOp, BindingContract, BindingOp, ComponentOp, DynamicName, ElementOp, ForBinding,
    ForOp, IfBranch, IfOp, InterpolationOp, ModelOp, Namespace, OnOp, Op, Region, SlotOp, TextOp,
    VueDirectiveOp,
};
use vize_s0::{Allocator, Box, Span, Vec as ArenaVec};

/// Canonical text of the reference tree.
const CANONICAL: &str = include_str!("fixtures/reference.folio");

/// Parse a retained `js` payload through the load-path parse site.
fn js<'a>(allocator: &'a Allocator, source: &'a str, start: u32, end: u32) -> ExprRef<'a> {
    let expr = JsExpr::parse_in(allocator, source, Span::new(start, end))
        .expect("reference-tree js payloads are admitted");
    ExprRef::Js(expr)
}

/// Allocate an `opaque` payload.
fn opaque<'a>(
    allocator: &'a Allocator,
    reason: OpaqueReason,
    source: &'a str,
    start: u32,
    end: u32,
) -> ExprRef<'a> {
    ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason,
        source,
        span: Span::new(start, end),
    }))
}

/// The same reference tree, built in a live arena with the op family.
fn arena_built<'a>(allocator: &'a Allocator) -> ArenaVec<'a, Op<'a>> {
    // `read` and `write` share one payload on purpose: `ExprRef` is a
    // shared reference, and the mirror must not care.
    let model_expr = js(allocator, "draft.note", 29, 39);
    let if_op = Op::If(Box::new_in(
        IfOp {
            branches: ArenaVec::from_iter_in(
                [
                    IfBranch {
                        condition: Some(js(allocator, "open", 65, 69)),
                        region: Region {
                            ops: ArenaVec::from_iter_in(
                                [Op::Text(Box::new_in(
                                    TextOp {
                                        content: "a\"b\\c",
                                        span: Span::new(66, 70),
                                    },
                                    &allocator,
                                ))],
                                &allocator,
                            ),
                        },
                        span: Span::new(61, 75),
                    },
                    IfBranch {
                        condition: None,
                        region: Region {
                            ops: ArenaVec::from_iter_in(
                                [Op::Interpolation(Box::new_in(
                                    InterpolationOp {
                                        expression: opaque(
                                            allocator,
                                            OpaqueReason::NestingRefused,
                                            "((((x))))",
                                            82,
                                            88,
                                        ),
                                        span: Span::new(80, 88),
                                    },
                                    &allocator,
                                ))],
                                &allocator,
                            ),
                        },
                        span: Span::new(75, 90),
                    },
                ],
                &allocator,
            ),
            span: Span::new(61, 90),
        },
        &allocator,
    ));
    let element = Op::Element(Box::new_in(
        ElementOp {
            tag: "form",
            namespace: Namespace::Html,
            attributes: ArenaVec::from_iter_in(
                [Attribute {
                    name: "method",
                    value: Some("post"),
                    span: Span::new(5, 20),
                }],
                &allocator,
            ),
            bindings: ArenaVec::from_iter_in(
                [
                    BindingOp::Model(Box::new_in(
                        ModelOp {
                            contract: BindingContract {
                                read: model_expr,
                                write: model_expr,
                            },
                            attributes: ArenaVec::from_iter_in(
                                [Attribute {
                                    name: "element-kind",
                                    value: Some("textarea"),
                                    span: Span::new(21, 40),
                                }],
                                &allocator,
                            ),
                            span: Span::new(21, 40),
                        },
                        &allocator,
                    )),
                    BindingOp::VueDirective(Box::new_in(
                        VueDirectiveOp {
                            name: "pin",
                            argument: Some(DynamicName::Static("top")),
                            modifiers: ArenaVec::from_iter_in(["lazy", "trim"], &allocator),
                            value: Some(opaque(
                                allocator,
                                OpaqueReason::MultiStatement,
                                "top++; sync()",
                                46,
                                59,
                            )),
                            span: Span::new(41, 60),
                        },
                        &allocator,
                    )),
                ],
                &allocator,
            ),
            children: Region {
                ops: ArenaVec::from_iter_in([if_op], &allocator),
            },
            span: Span::new(0, 99),
        },
        &allocator,
    ));
    let foreign = Op::Interpolation(Box::new_in(
        InterpolationOp {
            expression: ExprRef::Foreign(allocator.alloc(ForeignExpr {
                dialect: "moonbit",
                source: "count + 1",
                span: Span::new(150, 159),
                facts: ArenaVec::new_in(&allocator),
            })),
            span: Span::new(148, 161),
        },
        &allocator,
    ));
    let for_op = Op::For(Box::new_in(
        ForOp {
            binding: ForBinding {
                source: opaque(allocator, OpaqueReason::ForValue, "a in b in c", 110, 121),
                value: js(allocator, "a", 105, 106),
                key: Some(js(allocator, "i", 108, 109)),
                index: None,
            },
            region: Region {
                ops: ArenaVec::from_iter_in(
                    [Op::Slot(Box::new_in(
                        SlotOp {
                            name: DynamicName::Dynamic(js(allocator, "kind", 136, 140)),
                            // The outlet's props surface (P2-9 series 5):
                            // one static slot prop, one `ui.bind` with a
                            // modifier, one bare `ui.on` listener.
                            attributes: ArenaVec::from_iter_in(
                                [Attribute {
                                    name: "tone",
                                    value: Some("brisk"),
                                    span: Span::new(141, 147),
                                }],
                                &allocator,
                            ),
                            bindings: ArenaVec::from_iter_in(
                                [
                                    BindingOp::Bind(Box::new_in(
                                        BindOp {
                                            name: Some(DynamicName::Static("chip")),
                                            modifiers: ArenaVec::from_iter_in(
                                                ["camel"],
                                                &allocator,
                                            ),
                                            value: Some(js(allocator, "chipVal", 149, 156)),
                                            span: Span::new(148, 157),
                                        },
                                        &allocator,
                                    )),
                                    BindingOp::On(Box::new_in(
                                        OnOp {
                                            name: Some(DynamicName::Static("press")),
                                            modifiers: ArenaVec::from_iter_in(["stop"], &allocator),
                                            handler: None,
                                            span: Span::new(158, 160),
                                        },
                                        &allocator,
                                    )),
                                ],
                                &allocator,
                            ),
                            fallback: Region {
                                ops: ArenaVec::from_iter_in([foreign], &allocator),
                            },
                            span: Span::new(131, 161),
                        },
                        &allocator,
                    ))],
                    &allocator,
                ),
            },
            span: Span::new(100, 161),
        },
        &allocator,
    ));
    let component = Op::Component(Box::new_in(
        ComponentOp {
            name: "Chrome",
            attributes: ArenaVec::new_in(&allocator),
            bindings: ArenaVec::new_in(&allocator),
            children: Region {
                ops: ArenaVec::new_in(&allocator),
            },
            span: Span::new(162, 171),
        },
        &allocator,
    ));
    ArenaVec::from_iter_in([element, for_op, component], &allocator)
}

#[test]
fn an_arena_tree_mirrors_into_the_same_folio() {
    let allocator = Allocator::default();
    let ops = arena_built(&allocator);
    let mirrored = DisegnoFolio::of(&ops);
    assert_eq!(mirrored.op_count(), 12);
    assert_eq!(
        mirrored.print_to_string(FolioMode::Full).as_str(),
        CANONICAL
    );
    assert_eq!(
        mirrored,
        DisegnoFolio::parse(CANONICAL).expect("canonical text parses")
    );
}
