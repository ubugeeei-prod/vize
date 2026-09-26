#![no_main]

// L2 -> L3 lowering fuzz target (Davinci P3-3, TS-20).
//
// This extends the L1 -> L2 totality target through the Impeto bridge. Arbitrary
// UTF-8 must produce either a verified L3 program and partition facts or typed
// diagnostics from earlier stages; it must not panic. When L3 is emitted, the
// canonical Folio print must re-parse exactly. P3-10 placement annotation must
// verify, leave the graph page byte-identical, and round-trip its own page;
// O3 extraction must do the same and keep the partition export current.
use libfuzzer_sys::fuzz_target;
use vize_davinci::folio::{Folio, FolioMode};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::parse;
use vize_l1_to_l2::lower as lower_l1_to_l2;
use vize_davinci::folio::remarks::RemarkLog;
use vize_davinci::pass::RemarkCollector;
use vize_l2_to_l3::{Lowered, PartitionKind, lower as lower_l2_to_l3};
use vize_l3::extract::{OptTier, L3ExtractionFolio};
use vize_l3::folio::L3Folio;
use vize_l3::optimize::optimize;
use vize_l3::placement::{L3PlacementFolio, annotate};
use vize_l3::verify::verify;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if u32::try_from(source.len()).is_err() {
        return;
    }

    let allocator = Allocator::new();
    let root = SourceRoot::new(source).expect("the fuzz target rejects u32-overflowing sources");
    let (tree, errors) = parse(&allocator, source);
    let s2 = lower_l1_to_l2(&allocator, &tree, &errors);
    let mut lowered = lower_l2_to_l3(&allocator, &s2.root);

    assert_eq!(verify(&lowered.program), vec![]);
    assert_spans_resolve(root, &lowered);
    assert_partition_matches_program(&lowered);

    let folio = L3Folio::of(&lowered.program);
    let printed = folio.print_to_string(FolioMode::Full);
    let reparsed = L3Folio::parse(printed.as_str()).expect("canonical print must re-parse");
    assert_eq!(reparsed, folio);

    annotate(&mut lowered.program);
    assert_eq!(verify(&lowered.program), vec![]);
    assert_eq!(L3Folio::of(&lowered.program), folio);
    assert_partition_matches_program(&lowered);
    let placements = L3PlacementFolio::of(&lowered.program);
    let printed = placements.print_to_string(FolioMode::Full);
    assert_eq!(L3PlacementFolio::parse(printed.as_str()), Ok(placements));

    // P3-10 extraction at the largest tier commits a verified plan, keeps the
    // graph and the partition export exact, and reports a round-tripping page.
    let mut collector = RemarkCollector::new();
    let extraction = optimize(&mut lowered.program, OptTier::O3, &mut collector)
        .expect("the optimization pipeline is closed");
    assert_eq!(verify(&lowered.program), vec![]);
    assert_eq!(L3Folio::of(&lowered.program), folio);
    assert_eq!(lowered.partition.stale(&lowered.program), None);
    let report = L3ExtractionFolio::of(&extraction);
    let printed = report.print_to_string(FolioMode::Full);
    assert_eq!(L3ExtractionFolio::parse(printed.as_str()), Ok(report));
    let remarks = RemarkLog::new(collector.finish());
    assert_eq!(remarks.remarks.len(), extraction.decisions.len());
    let printed = remarks.print_to_string(FolioMode::Full);
    assert_eq!(RemarkLog::parse(printed.as_str()), Ok(remarks));
});

fn assert_spans_resolve(root: SourceRoot<'_>, lowered: &Lowered<'_>) {
    for region in &lowered.program.regions {
        assert_span_resolves(root, region.span, "region");
    }
    for op in &lowered.program.ops {
        assert_span_resolves(root, op.span, "op");
    }
    for effect in &lowered.program.effects {
        assert_span_resolves(root, effect.span, "effect");
    }
    for fact in &lowered.partition.ops {
        assert_span_resolves(root, fact.span, "partition");
    }
}

fn assert_span_resolves(root: SourceRoot<'_>, span: Span, label: &str) {
    assert!(
        root.contains_span(span),
        "{label} span @{}:{} is not a valid authored UTF-8 range",
        span.start,
        span.end
    );
}

fn assert_partition_matches_program(lowered: &Lowered<'_>) {
    assert_eq!(lowered.partition.ops.len(), lowered.program.ops.len());
    for (op, fact) in lowered.program.ops.iter().zip(lowered.partition.ops.iter()) {
        assert_eq!(fact.op, op.id);
        assert_eq!(fact.span, op.span);
        assert_eq!(fact.kind == PartitionKind::Dynamic, op.effect.is_some());
    }
}
