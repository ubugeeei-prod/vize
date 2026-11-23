use davinci_harness::{alloc, paired::assert_paired_time};
use std::hint::black_box;
use vize_incremental::{BlockKind, ResidentDatabase};
use vize_l0::{Lang, cstr};
use vize_l3::op::OpId;
use vize_resident_graph::TemplateQueries;

pub(crate) fn profile() {
    let db = ResidentDatabase::default();
    let text = cstr!(
        "<script>/*😀*/</script>\r\n<template>{}</template>",
        "<p title='雪'/>".repeat(256)
    );
    let file = db.open("SourceSites.vue", &text);
    let blocks = db.document_blocks(file).unwrap();
    let block = blocks
        .iter()
        .find(|block| block.kind() == &BlockKind::Template)
        .unwrap();
    let graph = block.reach_batch(Lang::Js).unwrap();
    let independent = |id| {
        let span = graph.authored_span(id).unwrap();
        let spelling = text.get(span.start as usize..span.end as usize).unwrap();
        (span, spelling)
    };
    for width in [1, 2, 8, 64, 256] {
        let ids: Vec<_> = (0..width).map(OpId::new).collect();
        let sites = graph.source_sites();
        for &id in &ids {
            assert_eq!(sites.get(id), Ok(independent(id)));
        }
        let run = |bound| {
            if bound {
                let sites = graph.source_sites();
                for &id in &ids {
                    black_box(sites.get(black_box(id)).unwrap());
                }
            } else {
                for &id in &ids {
                    black_box(independent(black_box(id)));
                }
            }
        };
        for bound in [false, true] {
            let ((), cost) = alloc::measure_returning(|| run(bound));
            let cost = cost.unwrap();
            assert_eq!((cost.calls, cost.peak_bytes_over_start), (0, 0));
        }
        assert_paired_time(&cstr!("authored source sites width={width}"), 64, run);
    }
}
