//! Measure complete and incomplete reads from the same resident graph.
use davinci_harness::{
    alloc::{self, CountingAllocator},
    paired::assert_paired_time,
};
use std::hint::black_box;
use vize_incremental::{BlockKind, ResidentDatabase, block_query::BlockView};
use vize_l0::{Lang, Span, cstr};
use vize_l3::op::OpId;
use vize_resident_graph::{TemplateGraphError, TemplateGraphObservation, TemplateQueries};

mod recovery_sites {
    pub(super) mod positions;
}
use recovery_sites::positions;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

fn source_sites(
    block: BlockView<'_>,
    text: &str,
    observed: bool,
    mut visit: impl FnMut(Span, &str),
) {
    if observed {
        let TemplateGraphObservation::Incomplete(issues) =
            TemplateGraphObservation::read(block, Lang::Js).unwrap()
        else {
            panic!("expected the broken template");
        };
        for span in issues
            .holes()
            .iter()
            .map(|r| r.span)
            .chain(issues.diagnostics().iter().map(|r| r.span))
        {
            let (authored, spelling) = issues.source_site(span).unwrap();
            visit(authored, spelling);
        }
    } else {
        let TemplateGraphError::Incomplete { holes, diagnostics } =
            block.reach_batch(Lang::Js).err().unwrap()
        else {
            panic!("expected the broken template");
        };
        for relative in holes
            .iter()
            .map(|r| r.span)
            .chain(diagnostics.iter().map(|r| r.span))
        {
            let span = block.authored_span(relative).unwrap();
            visit(
                span,
                text.get(span.start as usize..span.end as usize).unwrap(),
            );
        }
    }
}

fn complete() {
    let db = ResidentDatabase::default();
    let file = db.open("Complete.vue", "<template><p/></template>");
    let block = db.document_blocks(file).unwrap().iter().next().unwrap();
    let run = |observed| {
        let block = black_box(block);
        let span = if observed {
            let TemplateGraphObservation::Complete(sites) =
                TemplateGraphObservation::read(block, Lang::Js).unwrap()
            else {
                panic!("expected a graph");
            };
            sites.get(black_box(OpId::new(0))).unwrap().0
        } else {
            block
                .reach_batch(Lang::Js)
                .unwrap()
                .authored_span(black_box(OpId::new(0)))
                .unwrap()
        };
        black_box(span);
    };
    run(false);
    run(true);
    for observed in [false, true] {
        let ((), cost) = alloc::measure_returning(|| run(observed));
        let cost = cost.unwrap();
        assert_eq!((cost.calls, cost.peak_bytes_over_start), (0, 0));
    }
    assert_paired_time("complete graph and first authored site", 256, run);
}

fn main() {
    alloc::mark_installed();
    complete();
    for width in [1, 4, 32, 128] {
        let db = ResidentDatabase::default();
        let text = cstr!(
            "<script>/*🌸*/</script>\r\n<template>{}</template>",
            "<p>{{(}}</p>".repeat(width)
        );
        let file = db.open("Recovery.vue", &text);
        let block = db
            .document_blocks(file)
            .unwrap()
            .iter()
            .find(|block| block.kind() == &BlockKind::Template)
            .unwrap();
        let TemplateGraphObservation::Incomplete(recovery) =
            TemplateGraphObservation::read(block, Lang::Js).unwrap()
        else {
            panic!("expected retained recovery records");
        };
        positions::measure::<1>(&recovery, block.start(), width);
        positions::measure::<4>(&recovery, block.start(), width);
        positions::measure::<16>(&recovery, block.start(), width);
        positions::measure::<64>(&recovery, block.start(), width);
        let collect = |observed| {
            let mut sites = Vec::new();
            source_sites(block, &text, observed, |span, spelling| {
                sites.push((span, spelling.as_bytes().to_vec()))
            });
            sites
        };
        let expected = collect(false);
        let actual = collect(true);
        assert!(!actual.is_empty());
        assert_eq!(expected, actual);
        let run = |observed| {
            source_sites(black_box(block), &text, observed, |span, spelling| {
                black_box((span, spelling));
            })
        };
        for observed in [false, true] {
            let ((), cost) = alloc::measure_returning(|| run(observed));
            let cost = cost.unwrap();
            assert_eq!((cost.calls, cost.peak_bytes_over_start), (0, 0));
        }
        assert_paired_time(
            &cstr!("recovery sites width={width} sites={}", actual.len()),
            64,
            run,
        );
    }
}
