#![no_main]

// S2 -> S3 lowering fuzz target (Davinci P3-3, TS-20).
//
// This extends the S1 -> S2 totality target through the Impeto bridge. Arbitrary
// UTF-8 must produce either a verified S3 program and partition facts or typed
// diagnostics from earlier stages; it must not panic. When S3 is emitted, the
// canonical Folio print must re-parse exactly.
use libfuzzer_sys::fuzz_target;
use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, SourceRoot, Span};
use vize_s1::parse;
use vize_s1_to_s2::lower as lower_s1_to_s2;
use vize_s2_to_s3::{Lowered, PartitionKind, lower as lower_s2_to_s3};
use vize_s3::folio::S3Folio;
use vize_s3::verify::verify;

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
    let s2 = lower_s1_to_s2(&allocator, &tree, &errors);
    let lowered = lower_s2_to_s3(&allocator, &s2.root);

    assert_eq!(verify(&lowered.program), vec![]);
    assert_spans_resolve(root, &lowered);
    assert_partition_matches_program(&lowered);

    let folio = S3Folio::of(&lowered.program);
    let printed = folio.print_to_string(FolioMode::Full);
    let reparsed = S3Folio::parse(printed.as_str()).expect("canonical print must re-parse");
    assert_eq!(reparsed, folio);
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
