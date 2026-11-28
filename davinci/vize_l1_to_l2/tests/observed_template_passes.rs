#![cfg(feature = "observed-template-passes")]

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use davinci_harness::paired::assert_paired_time;
use std::{alloc::System, hint::black_box};
use vize_l0::line_index::{LineIndex, Position};
use vize_l0::{
    Allocator, SourceBlock, SourceRoot, Span,
    config::{VueDialect, VueVersion},
    pass::{NoObserver, PassObserver, RemarkCollector, run_pipeline_remarked},
    side_table::SideTable,
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::{SurfaceParseOptions, parse},
};
use vize_l1_to_l2::{
    Lowered, lower_source_block,
    pass::{cfg, hoist, template},
};
use vize_l2::{
    analysis::{static_analysis::StaticAnalysis, template_complexity::ComplexityAnalysis},
    walk::{NodeRef, PageWalk, visit_nodes},
};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn independent<O: PassObserver>(
    lowered: &mut Lowered<'_>,
    observer: &mut O,
) -> (SideTable<StaticAnalysis>, ComplexityAnalysis) {
    let mut statics = None;
    let mut complexity = None;
    run_pipeline_remarked(&template::PLAN, observer, |event, remarks| {
        if event.desc() == hoist::DESC {
            statics = Some(hoist::run_remarked(lowered, remarks));
        } else {
            assert_eq!(event.desc(), cfg::DESC);
            complexity = Some(cfg::run(lowered));
        }
        Ok(())
    })
    .unwrap();
    (statics.unwrap(), complexity.unwrap())
}

#[derive(Debug, Default)]
struct Counts {
    owners: usize,
    contributions: usize,
    visits: u32,
    errors: usize,
    remarks: usize,
}

fn check(block: SourceBlock<'_>, inspect: impl FnOnce(&template::TemplateAnalysis)) -> Counts {
    let arena = Allocator::default();
    let (tree, errors) = parse(&arena, block.source());
    let mut before = lower_source_block(&arena, &tree, &errors, block);
    let mut after = lower_source_block(&arena, &tree, &errors, block);
    let mut old_observer = RemarkCollector::default();
    let mut new_observer = RemarkCollector::default();
    let (statics, complexity) = independent(&mut before, &mut old_observer);
    let joint = template::analyze(&mut after, &mut new_observer).unwrap();
    assert_eq!(
        joint.static_facts.sorted_entries(),
        statics.sorted_entries()
    );
    assert_eq!(joint.complexity, complexity);
    assert_eq!(before.artifact.provenance, after.artifact.provenance);
    assert_eq!(before.diagnostics, after.diagnostics);
    inspect(&joint);
    let expected = old_observer.finish();
    assert_eq!(new_observer.finish(), expected);
    let mut actual_visits = 0;
    let mut page = PageWalk::new();
    visit_nodes(&mut page, &before.artifact.root.ops, &mut |_, node| {
        actual_visits += u32::from(matches!(node, NodeRef::Op(_)));
    })
    .unwrap();
    assert_eq!(page.minted(), before.op_count);
    assert_eq!(joint.op_visits, actual_visits);
    for row in &joint.complexity.contributions {
        assert!(
            row.span.start >= block.start() && row.span.end <= block.span().end,
            "{:?} {:?}",
            block.span(),
            row.span
        );
        let relative = Span::new(row.span.start - block.start(), row.span.end - block.start());
        assert_eq!(
            row.span.slice(block.root_source()),
            relative.slice(block.source())
        );
    }
    Counts {
        owners: statics.len(),
        contributions: complexity.contributions.len(),
        visits: actual_visits,
        errors: before.diagnostics.len(),
        remarks: expected.len(),
    }
}

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
fn original(source: &str) -> Option<Counts> {
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(&arena, source, options());
    let selected = observation.admitted().ok()?.template()?;
    Some(check(selected.block(), |_| {}))
}

#[test]
fn real_template_passes_keep_authored_utf16_prefixes_and_recovered_input() {
    let text = "<main><!-- 🧭 --><i v-if='ok && next'>雪</i><b class='x'>z</b></main>";
    let prefix = "<!-- 🦀 -->\r\n<script setup lang=ts>const open: =</script>\r\n<template>";
    let source = format!("{prefix}{text}</template>");
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(&arena, &source, options());
    let selected = observation.admitted().unwrap().template().unwrap();
    let result = check(selected.block(), |analysis| {
        let rows = &analysis.complexity.contributions;
        assert_eq!(
            rows.iter().map(|r| r.kind).collect::<Vec<_>>(),
            [cfg::DecisionKind::If, cfg::DecisionKind::Logical]
        );
        let start = source.find("ok && next").unwrap() as u32;
        let condition = rows[0].span;
        assert_eq!(condition, Span::new(start, start + 10));
        assert_eq!(condition.slice(&source), "ok && next");
        let positions = LineIndex::new(&source);
        assert_eq!(
            positions.position(condition.start as usize),
            Position::new(2, 36)
        );
        assert_eq!(
            positions.position(condition.end as usize),
            Position::new(2, 46)
        );
    });
    assert_eq!(result.errors, 0);

    for text in [
        "",
        "text<!-- preserved -->",
        "<div><span",
        "<p>{{ broken + }}</p>",
        "<section><li v-for='x in xs' v-if='x.ok'>{{x}}</li></section>",
    ] {
        let source = format!("{prefix}{text}</template>");
        let block = SourceRoot::new(&source)
            .unwrap()
            .block(
                &source[prefix.len()..prefix.len() + text.len()],
                prefix.len() as u32,
            )
            .unwrap();
        let recovered = check(block, |_| {});
        if text == "<div><span" {
            assert!(
                recovered.errors > 0,
                "incomplete start tag must report recovery"
            );
        }
    }
}

#[test]
#[ignore = "checks original tracked or external Vue documents"]
fn every_selected_original_template_matches_separate_runtime_passes() {
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_TEMPLATE_PASS_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut selected, mut invalid_utf8, mut recovered) =
        (0usize, 0usize, 0usize, 0usize);
    let mut total = Counts::default();
    for name in manifest.split_terminator('\0') {
        if !(name.ends_with(".vue") || name.ends_with(".vue.txt")) {
            continue;
        }
        files += 1;
        let bytes = std::fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid_utf8 += 1;
            continue;
        };
        if let Some(c) = original(source) {
            selected += 1;
            recovered += usize::from(c.errors > 0);
            total.owners += c.owners;
            total.contributions += c.contributions;
            total.visits += c.visits;
            total.remarks += c.remarks;
        }
    }
    assert!(
        selected > 0
            && total.owners > 0
            && total.contributions > 0
            && total.visits > 0
            && total.remarks > 0
    );
    println!(
        "original template passes files={files} non_utf8={invalid_utf8} selected={selected} recovered={recovered} owners={} contributions={} operations={} remarks={}",
        total.owners, total.contributions, total.visits, total.remarks
    );
}

#[test]
#[ignore = "isolated complete pass results, allocation counts and alternating runtime"]
fn original_pass_results_have_one_walk_and_measured_resource_cost() {
    mark_installed();
    for elements in [1, 16, 96] {
        let source = format!(
            "<section>{}</section>",
            "<i v-if='ok && active' class='item'>hello</i>".repeat(elements)
        );
        let arena = Allocator::default();
        let root = SourceRoot::new(&source).unwrap();
        let (tree, errors) = parse(&arena, &source);
        let mut before = lower_source_block(&arena, &tree, &errors, root.whole_block());
        let mut after = lower_source_block(&arena, &tree, &errors, root.whole_block());
        let initial = before.artifact.provenance.len();
        let mut run = |stream: bool| {
            if stream {
                black_box(template::analyze(black_box(&mut after), &mut NoObserver).unwrap());
                after.artifact.provenance.truncate(initial);
            } else {
                black_box(independent(black_box(&mut before), &mut NoObserver));
                before.artifact.provenance.truncate(initial);
            }
        };
        run(false);
        run(true);
        let old = measure(|| run(false)).unwrap();
        let new = measure(|| run(true)).unwrap();
        assert!(new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start);
        let label = format!(
            "template passes elements={elements} allocations={}/{} peak_bytes={}/{}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        assert_paired_time(&label, 64, run);
    }
}
