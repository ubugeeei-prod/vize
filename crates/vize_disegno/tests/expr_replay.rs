//! Arena-reset replay (P2-5b): P1-11's resident-cache reset scenario
//! applied to folios.
//!
//! The law under test: a printed folio carries everything an S2 tree
//! needs to survive the arena it was built in. The mechanism that makes
//! surviving *in place* impossible is the pool guard plus the debug
//! arena-generation stamp (`crates/vize_carton/src/allocator/generation.rs`:
//! a stamped value read after `Allocator::reset` panics) - so the folio
//! stores owned text + span, and a `js(...)` payload re-parses into a
//! fresh arena on load. This test drives the full cycle: build in a
//! pooled arena, mirror, print, **drop the `pool::acquire()` guard**
//! (arena reset), parse the surviving text, assert structural equality,
//! then re-parse the retained payload into a second pooled arena.

use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::expr::{ExprRef, JsExpr, OpaqueExpr, OpaqueReason};
use vize_disegno::folio::{DisegnoFolio, FolioExpr, FolioInterpolation, FolioOp};
use vize_disegno::op::{InterpolationOp, Op};
use vize_s0::{Allocator, Box, Span, Vec as ArenaVec, pool};

/// Build the replay tree in `allocator`: one admitted `js` payload and
/// one escape payload, each under an interpolation.
fn arena_built<'a>(allocator: &'a Allocator) -> ArenaVec<'a, Op<'a>> {
    let retained = JsExpr::parse_in(allocator, "items.filter(Boolean).length", Span::new(2, 30))
        .expect("the replay payload is admitted");
    let escape = allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::MultiStatement,
        source: "a++; b++",
        span: Span::new(36, 44),
    });
    ArenaVec::from_iter_in(
        [
            Op::Interpolation(Box::new_in(
                InterpolationOp {
                    expression: ExprRef::Js(retained),
                    span: Span::new(0, 32),
                },
                &allocator,
            )),
            Op::Interpolation(Box::new_in(
                InterpolationOp {
                    expression: ExprRef::Opaque(escape),
                    span: Span::new(34, 46),
                },
                &allocator,
            )),
        ],
        &allocator,
    )
}

/// The canonical page the replay tree prints - pinned so the test cannot
/// silently degenerate into comparing two empty artifacts.
const CANONICAL: &str = "\
[disegno]
ops=2

[disegno.ops]
ui.interpolation js(\"items.filter(Boolean).length\" @2:30) @0:32
ui.interpolation opaque(multi-statement \"a++; b++\" @36:44) @34:46

";

#[test]
fn a_printed_folio_replays_across_an_arena_reset() {
    // -- print inside the pooled arena's lifetime -------------------------
    let (printed, mirrored) = {
        let guard = pool::acquire();
        let ops = arena_built(&guard);
        // (`&guard` deref-coerces to `&Allocator`; the guard owns the
        // arena for this block.)
        let mirrored = DisegnoFolio::of(&ops);
        (mirrored.print_to_string(FolioMode::Full), mirrored)
        // `guard` drops here: the arena resets and every `ExprRef` above -
        // including the retained oxc AST - is gone. Only the owned folio
        // and its printed text survive, which is the P1-11 contract this
        // test exists to hold the folio to.
    };
    assert_eq!(printed.as_str(), CANONICAL);

    // -- parse after the reset --------------------------------------------
    let replayed = DisegnoFolio::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(replayed, mirrored);

    // -- re-parse the retained payload into a fresh arena on load ---------
    let guard = pool::acquire();
    let allocator: &Allocator = &guard;
    let FolioOp::Interpolation(FolioInterpolation {
        expression: FolioExpr::Js { source, span },
        ..
    }) = &replayed.ops[0]
    else {
        panic!("the first replayed op must be the retained-js interpolation");
    };
    let source = allocator.alloc_str(source.as_str());
    let loaded = ExprRef::parse_js_in(allocator, source, *span);
    let ExprRef::Js(loaded) = loaded else {
        panic!("an admitted payload must reload as ExprRef::Js");
    };
    assert_eq!(loaded.source, "items.filter(Boolean).length");
    assert_eq!(loaded.span, Span::new(2, 30));
    // The reloaded tree prints the same page: the load lost nothing.
    let ops = ArenaVec::from_iter_in(
        [
            Op::Interpolation(Box::new_in(
                InterpolationOp {
                    expression: ExprRef::Js(loaded),
                    span: Span::new(0, 32),
                },
                &allocator,
            )),
            Op::Interpolation(Box::new_in(
                InterpolationOp {
                    expression: ExprRef::Opaque(allocator.alloc(OpaqueExpr {
                        reason: OpaqueReason::MultiStatement,
                        source: "a++; b++",
                        span: Span::new(36, 44),
                    })),
                    span: Span::new(34, 46),
                },
                &allocator,
            )),
        ],
        &allocator,
    );
    assert_eq!(
        DisegnoFolio::of(&ops)
            .print_to_string(FolioMode::Full)
            .as_str(),
        CANONICAL
    );
}

/// The load path is total: text a folio should never contain still loads,
/// as the escape variant with the text-classified reason - never a panic,
/// never a partial state.
#[test]
fn the_load_path_falls_back_to_the_escape_variant() {
    let arena = Allocator::default();
    let allocator = &arena;

    let rejected = ExprRef::parse_js_in(allocator, "a++; b++", Span::new(0, 8));
    let ExprRef::Opaque(rejected) = rejected else {
        panic!("non-covering text must load as ExprRef::Opaque");
    };
    assert_eq!(rejected.reason, OpaqueReason::ParseRejected);
    assert_eq!(rejected.source, "a++; b++");
    assert_eq!(rejected.span, Span::new(0, 8));

    let invalid = ExprRef::parse_js_in(allocator, "%", Span::new(0, 1));
    let ExprRef::Opaque(invalid) = invalid else {
        panic!("invalid text must load as ExprRef::Opaque");
    };
    assert_eq!(invalid.reason, OpaqueReason::ParseRejected);

    // 40 levels of nesting: refused by the shared guard before oxc ever
    // sees the text (`vize_s0::expression_guard`, depth cap 31).
    let deep = "((((((((((((((((((((((((((((((((((((((((x))))))))))))))))))))))))))))))))))))))))";
    let refused = ExprRef::parse_js_in(allocator, deep, Span::new(0, 81));
    let ExprRef::Opaque(refused) = refused else {
        panic!("guard-refused text must load as ExprRef::Opaque");
    };
    assert_eq!(refused.reason, OpaqueReason::NestingRefused);

    let admitted = ExprRef::parse_js_in(allocator, "a + b", Span::new(0, 5));
    let ExprRef::Js(admitted) = admitted else {
        panic!("admitted text must load as ExprRef::Js");
    };
    assert_eq!(admitted.source, "a + b");
}
