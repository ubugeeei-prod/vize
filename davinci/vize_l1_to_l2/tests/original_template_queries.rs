//! A structural template request retains the exact L1 source and all L2 results.
mod original_template_queries {
    mod corpus;
    mod profile;
}

#[global_allocator]
static ALLOCATOR: davinci_harness::alloc::CountingAllocator<std::alloc::System> =
    davinci_harness::alloc::CountingAllocator::system();

use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    pass::{NoObserver, RemarkCollector},
};
use vize_l1::{
    container::{
        Vue,
        vue::{CheckedTemplateBlock, DescriptorOptions},
    },
    markup::{SurfaceError, SurfaceParseOptions, parse_with_options, render},
};
use vize_l1_to_l2::{
    lower_source_block,
    pass::{cfg, hoist, template::OriginalTemplate},
};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn render_source(tree: &vize_l1::markup::SurfaceTree<'_>) -> String {
    let mut text = String::new();
    render(tree, &mut |part| text.push_str(part));
    text
}

fn errors(rows: &[SurfaceError], text: &str) -> Vec<(String, Span, String)> {
    rows.iter()
        .map(|e| {
            (
                format!("{:?}", e.code),
                e.span(text.len() as u32),
                e.message(text).to_string(),
            )
        })
        .collect()
}

fn compare(arena: &Allocator, selected: CheckedTemplateBlock<'_, '_>) -> (usize, usize) {
    let block = selected.block();
    let mut observer = RemarkCollector::default();
    let session = OriginalTemplate::analyze(arena, selected, &mut observer).unwrap();
    let (original, parse_errors) =
        parse_with_options(arena, block.source(), selected.surface_options());
    let mut reference = lower_source_block(arena, &original, &parse_errors, block);
    let mut reference_observer = RemarkCollector::default();
    let (mut statics, mut complexity) = (None, None);
    vize_l0::pass::run_pipeline_remarked(
        &vize_l1_to_l2::pass::template::PLAN,
        &mut reference_observer,
        |event, remarks| {
            match event.desc().name {
                hoist::NAME => statics = Some(hoist::run_remarked(&mut reference, remarks)),
                cfg::NAME => complexity = Some(cfg::run(&reference)),
                _ => panic!("unexpected analysis"),
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(render_source(session.l1ir()), block.source());
    assert_eq!(render_source(&original), block.source());
    assert!(std::ptr::eq(
        session.block().root_source(),
        block.root_source()
    ));
    assert_eq!(session.block().span(), block.span());
    assert_eq!(
        errors(session.syntax_errors(), block.source()),
        errors(&parse_errors, block.source())
    );
    assert_eq!(session.l2ir().diagnostics, reference.diagnostics);
    assert_eq!(
        session.l2ir().artifact.provenance,
        reference.artifact.provenance
    );
    assert_eq!(session.analysis().complexity, complexity.unwrap());
    assert_eq!(
        session.analysis().static_facts.sorted_entries(),
        statics.unwrap().sorted_entries()
    );
    assert_eq!(observer.finish(), reference_observer.finish());
    (
        parse_errors.len(),
        session.analysis().complexity.contributions.len(),
    )
}

#[test]
fn sibling_language_failures_keep_the_original_tree_and_absolute_condition() {
    let text = "<!--🦀-->\r\n<div v-if='ready && open'>雪</div>";
    for sibling in [
        "<script src='./missing.ts'></script>",
        "<style src='./missing.css'></style>",
        "<script lang='coffee'>x = -> 1</script>",
        "<script>x</script><script>y</script>",
        "<script>let x</script><script setup lang='ts'>const x: =</script>",
    ] {
        let source = format!("<!--🧭-->\r\n{sibling}\r\n<template>{text}</template>");
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(&arena, &source, options());
        assert!(descriptor.admitted().is_err());
        let view = descriptor.admitted_template().unwrap().unwrap();
        assert_eq!(view.block().source(), text);
        assert_eq!(view.tag_names().unwrap().0.slice(&source), "template");
        assert_eq!(compare(&arena, view), (0, 2));
        let session = OriginalTemplate::analyze(&arena, view, &mut NoObserver).unwrap();
        let [condition, logical] = session.analysis().complexity.contributions.as_slice() else {
            panic!("both operators")
        };
        assert_eq!(
            (condition.kind, logical.kind),
            (cfg::DecisionKind::If, cfg::DecisionKind::Logical)
        );
        assert_eq!(condition.span.slice(&source), "ready && open");
    }
}

#[test]
fn absence_refusal_and_recovered_content_have_separate_results() {
    let arena = Allocator::default();
    let absent = Vue.observe_descriptor(&arena, "<style src='x'></style>", options());
    assert!(absent.admitted_template().unwrap().is_none());
    for source in [
        "<template>a</template><template>b</template>",
        "<template src='external.html'></template>",
        "<template lang='pug'>p Hello</template>",
        "<template lang='html'lang='html'>x</template>",
        "<script>x</script><template>x",
        "<template>x</template><style",
        "<custom>x</custom><template>y</template>",
    ] {
        let descriptor = Vue.observe_descriptor(&arena, source, options());
        let refusal = descriptor.admitted_template().unwrap_err();
        assert_eq!(refusal.issues(), descriptor.issues());
        assert_eq!(refusal.errors(), descriptor.container().errors.as_slice());
    }
    let source = "<script src='outside'></script><template><p :x=>雪</p></template>";
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let selected = descriptor.admitted_template().unwrap().unwrap();
    let (errors, _) = compare(&arena, selected);
    assert!(
        errors > 0,
        "missing value must retain the original diagnostic"
    );
    let session = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
    assert_eq!(session.block().source(), "<p :x=>雪</p>");
    assert!(!session.l2ir().diagnostics.is_empty());
}

#[test]
fn a_rejected_sibling_cannot_override_the_original_parser_profile() {
    use vize_l0::TemplateSyntaxMode;
    let source = "<script src='external'></script><template><p>a<div>雪</div></p></template>";
    let arena = Allocator::default();
    for html_tree_construction in [false, true] {
        let policy = DescriptorOptions {
            template: SurfaceParseOptions {
                html_tree_construction,
                ..Default::default()
            },
            ..options()
        };
        let descriptor = Vue.observe_descriptor(&arena, source, policy);
        let selected = descriptor.admitted_template().unwrap().unwrap();
        assert_eq!(selected.surface_options(), policy.template);
        compare(&arena, selected);
    }
    for policy in [
        DescriptorOptions {
            version: VueVersion::V2,
            ..options()
        },
        DescriptorOptions {
            template: SurfaceParseOptions {
                syntax: TemplateSyntaxMode::Strict,
                ..Default::default()
            },
            ..options()
        },
        DescriptorOptions {
            template: SurfaceParseOptions {
                syntax: TemplateSyntaxMode::Quirks,
                ..Default::default()
            },
            ..options()
        },
        DescriptorOptions {
            template: SurfaceParseOptions {
                experimental_in_tag_comments: true,
                ..Default::default()
            },
            ..options()
        },
    ] {
        let descriptor = Vue.observe_descriptor(&arena, source, policy);
        let refusal = descriptor.admitted_template().unwrap_err();
        assert_eq!(refusal.issues(), descriptor.issues());
        assert!(
            refusal
                .issues()
                .iter()
                .any(|issue| issue.container_index.is_none())
        );
    }
}
