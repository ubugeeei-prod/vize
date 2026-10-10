//! Error owners are parked on the heap; node-size gates do not make them free.

use core::mem::{align_of, needs_drop, size_of, size_of_val};
use davinci_harness::alloc::{measure_returning, stats};
use oxc_span::SourceType;
use vize_l0::Allocator;
use vize_relief::{RetainedJsAst, RetainedJsAstKind, SimpleExpressionNode};

pub(super) fn check() {
    assert!(!needs_drop::<RetainedJsAstKind<'static>>());
    assert!(!needs_drop::<RetainedJsAst<'static>>());
    assert!(!needs_drop::<SimpleExpressionNode<'static>>());
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(size_of::<Option<RetainedJsAst<'_>>>(), 24);
        assert_eq!(size_of::<SimpleExpressionNode<'_>>(), 88);
    }
    let parser_arena = Allocator::new();
    let errors = oxc_parser::Parser::new(parser_arena.as_oxc(), "{ value: }", SourceType::ts())
        .parse_slot_parameters()
        .expect_err("malformed control");
    let diagnostics_bytes = size_of_val(&errors);
    let mut parking = Allocator::with_capacity(4096);
    let arena_start = parking.allocated_bytes();
    let (parked, metrics) = measure_returning(|| parking.alloc_owned(errors));
    let metrics = metrics.expect("counting allocator installed");
    assert!(!parked.is_empty());
    assert_eq!(
        parking.allocated_bytes(),
        arena_start,
        "parking is outside the arena"
    );
    assert!(
        metrics.calls >= 2,
        "the diagnostic owner box and parking table both allocate"
    );
    assert!(metrics.peak_bytes_over_start >= diagnostics_bytes as u64);
    let live_before_reset = stats().live_bytes;
    parking.reset();
    let released = live_before_reset.saturating_sub(stats().live_bytes);
    assert!(
        released >= diagnostics_bytes as u64,
        "reset drops the diagnostic owner and contents"
    );

    let mut refusal_arena = Allocator::with_capacity(4096);
    let arena_start = refusal_arena.allocated_bytes();
    let (refusal, empty_metrics) =
        measure_returning(|| RetainedJsAst::slot_refusal_in(&refusal_arena, "deep", None));
    let empty_metrics = empty_metrics.expect("counting allocator installed");
    assert!(
        refusal
            .as_slot_parameters()
            .expect("refused role")
            .expect_err("silent refusal")
            .is_empty()
    );
    let kind_bytes = size_of::<RetainedJsAstKind<'_>>();
    let arena_bytes = refusal_arena.allocated_bytes() - arena_start;
    assert!(arena_bytes >= kind_bytes);
    assert!(arena_bytes < kind_bytes + align_of::<RetainedJsAstKind<'_>>());
    assert!(
        empty_metrics.calls >= 2,
        "empty diagnostics still park an owned box"
    );
    refusal_arena.reset();
    println!(
        "retained slot layout: kind={kind_bytes}, option={}, node={}, no Drop",
        size_of::<Option<RetainedJsAst<'_>>>(),
        size_of::<SimpleExpressionNode<'_>>()
    );
    println!(
        "original diagnostics parking: heap_calls={}, peak_heap_bytes={}, reset_released_bytes={released}; empty guard refusal: heap_calls={}, peak_heap_bytes={}, arena_kind_bytes={arena_bytes}",
        metrics.calls,
        metrics.peak_bytes_over_start,
        empty_metrics.calls,
        empty_metrics.peak_bytes_over_start
    );
}
