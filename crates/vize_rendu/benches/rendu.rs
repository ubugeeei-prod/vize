//! Frontend-neutral Rendu HIR traversal benchmark.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use vize_rendu::{
    RenduBuilder, RenduNamespace, RenduNode, RenduProvenance, RenduSource, summarize_rendu,
};

fn render_tree(width: usize, depth: usize) -> vize_rendu::RenduRoot {
    let mut builder = RenduBuilder::new();
    builder.add_source(RenduSource::named("benchmark.vue", "<generated />").with_language("vue"));
    let mut level = (0..width)
        .map(|_| {
            builder.add_node(RenduNode::Text {
                value: "leaf".into(),
                provenance: RenduProvenance::generated(),
            })
        })
        .collect::<Vec<_>>();
    for _ in 0..depth {
        level = level
            .chunks(4)
            .map(|children| {
                builder.add_node(RenduNode::Element {
                    tag: "div".into(),
                    namespace: RenduNamespace::Html,
                    properties: Vec::new(),
                    children: children.to_vec(),
                    provenance: RenduProvenance::generated(),
                })
            })
            .collect();
    }
    builder.set_entry(level);
    builder.finish().expect("benchmark HIR must validate")
}

fn benchmark_rendu_walk(criterion: &mut Criterion) {
    for (name, width, depth) in [
        ("rendu_hir_flat", 64, 1),
        ("rendu_hir_nested", 64, 4),
        ("rendu_hir_wide", 1_024, 3),
    ] {
        let root = render_tree(width, depth);
        criterion.bench_function(name, |bench| {
            bench.iter(|| black_box(summarize_rendu(black_box(&root))));
        });
    }
}

criterion_group!(benches, benchmark_rendu_walk);
criterion_main!(benches);
