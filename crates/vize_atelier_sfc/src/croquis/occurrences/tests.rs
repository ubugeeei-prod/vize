#![expect(
    clippy::string_slice,
    reason = "authored input witnesses are asserted by tests"
)]

use super::*;
use crate::{SfcParseOptions, parse_sfc};
use vize_croquis::binding_occurrences::{BindingIdentity, OccurrenceBlock};
use vize_l0::Allocator;

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/binding-occurrences-original/Field.vue"
));

fn analyze(
    source: &str,
    run: impl FnOnce(&SfcDescriptor<'_>, &SfcCroquisAnalysis, &BindingOccurrences),
) {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let allocator = Allocator::default();
    let root = descriptor.template.as_ref().map(|template| {
        let (root, errors) = vize_armature::parse(&allocator, &template.content);
        assert!(errors.is_empty());
        root
    });
    let mut options = SfcCroquisOptions::lint_demand();
    options.analyzer_options.collect_template_expressions = true;
    let ordinary =
        super::super::analyze_sfc_descriptor_with_context(&descriptor, root.as_ref(), options);
    let (analysis, packet) =
        analyze_sfc_descriptor_with_occurrences(&descriptor, root.as_ref(), options);
    assert_eq!(ordinary.croquis.to_vir(), analysis.croquis.to_vir());
    assert_eq!(
        serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
        serde_json::to_value(analysis.croquis.semantic_snapshot()).unwrap()
    );
    assert_eq!(ordinary.script_content, analysis.script_content);
    assert_eq!(ordinary.script_offset, analysis.script_offset);
    let packet = packet.expect("complete original AST/source witnesses");
    run(&descriptor, &analysis, &packet);
}

fn binding(packet: &BindingOccurrences, name: &str) -> BindingIdentity {
    packet
        .bindings()
        .find(|binding| binding.name == name && binding.identity.block == OccurrenceBlock::Script)
        .unwrap()
        .identity
}

#[test]
fn full_original_project_has_one_label_one_hint_and_three_id_template_reads() {
    analyze(ORIGINAL, |descriptor, analysis, packet| {
        let script = analysis.script_content_ref().unwrap();
        let template = descriptor.template.as_ref().unwrap();
        assert_eq!(packet.occurrences().len(), 5);
        for (name, count) in [("label", 1), ("hint", 1), ("id", 3)] {
            let identity = binding(packet, name);
            assert_eq!(
                &script[identity.start as usize..identity.end as usize],
                name
            );
            let reads: Vec<_> = packet
                .occurrences()
                .iter()
                .filter(|reference| reference.binding == identity)
                .collect();
            assert_eq!(reads.len(), count);
            for reference in reads {
                assert_eq!(reference.block, OccurrenceBlock::Template);
                assert_eq!(
                    &template.content[reference.start as usize..reference.end as usize],
                    name
                );
                let physical = template.loc.start + reference.start as usize;
                assert_eq!(&ORIGINAL[physical..physical + name.len()], name);
            }
        }
    });
}

#[test]
fn script_closure_declarator_and_template_alias_reads_have_distinct_owners() {
    let source = "<script setup lang=\"ts\">const id = 'outer'; function own(id: string) { return id } const copy = id; { const id = 'block'; consume(id) }</script><template><p>{{ id }}</p><p v-for=\"id in [1]\">{{ id }}</p></template>";
    analyze(source, |descriptor, analysis, packet| {
        let script = analysis.script_content_ref().unwrap();
        let global = packet
            .bindings()
            .find(|binding| binding.name == "id" && binding.identity.start == 6)
            .unwrap()
            .identity;
        let global_reads: Vec<_> = packet
            .occurrences()
            .iter()
            .filter(|reference| reference.binding == global)
            .collect();
        assert_eq!(global_reads.len(), 2);
        assert_eq!(
            global_reads
                .iter()
                .filter(|reference| reference.block == OccurrenceBlock::Script)
                .count(),
            1
        );
        let locals: Vec<_> = packet
            .bindings()
            .filter(|binding| binding.name == "id" && binding.identity != global)
            .collect();
        assert_eq!(locals.len(), 3);
        assert!(
            locals
                .iter()
                .all(|binding| binding.identity.scope != global.scope)
        );
        for binding in locals {
            let text = match binding.identity.block {
                OccurrenceBlock::Script => script,
                OccurrenceBlock::Template => descriptor.template.as_ref().unwrap().content.as_ref(),
                _ => unreachable!(),
            };
            assert_eq!(
                &text[binding.identity.start as usize..binding.identity.end as usize],
                "id"
            );
            assert_eq!(
                packet
                    .occurrences()
                    .iter()
                    .filter(|reference| reference.binding == binding.identity)
                    .count(),
                1
            );
        }
    });
}

#[test]
fn existing_style_analysis_retains_both_blocks_exact_quotes_and_unicode_spans() {
    let source = "<script setup>const color = 'red'; const 名称 = 'name'</script><template><p>{{ 名称 }}</p></template><style>p { color: v-bind(color); content: \"v-bind(color)\"; } /* v-bind(color) */</style><style scoped>p { color: v-bind('color'); --name: v-bind(名称); }</style>";
    analyze(source, |descriptor, _, packet| {
        let color = binding(packet, "color");
        let reads: Vec<_> = packet
            .occurrences()
            .iter()
            .filter(|reference| reference.binding == color)
            .collect();
        assert_eq!(reads.len(), 2);
        for reference in reads {
            let OccurrenceBlock::Style(index) = reference.block else {
                panic!("CSS occurrence");
            };
            let style = &descriptor.styles[index as usize];
            assert_eq!(
                &style.content[reference.start as usize..reference.end as usize],
                "color"
            );
        }
        let name = binding(packet, "名称");
        assert_eq!(
            packet
                .occurrences()
                .iter()
                .filter(|reference| reference.binding == name)
                .count(),
            2
        );
    });
}

#[test]
fn split_scripts_use_the_existing_joined_script_space_and_original_blocks() {
    let source = "<script>export const plain = 1</script>\r\n<script setup>const id = plain; const out = id</script>\r\n<template>{{ plain }} {{ id }}</template>";
    analyze(source, |descriptor, analysis, packet| {
        let script = analysis.script_content_ref().unwrap();
        for name in ["plain", "id"] {
            let identity = binding(packet, name);
            assert_eq!(
                &script[identity.start as usize..identity.end as usize],
                name
            );
            let physical = analysis.script_source_offset(descriptor, identity.start) as usize;
            assert_eq!(&source[physical..physical + name.len()], name);
        }
        let plain = binding(packet, "plain");
        assert_eq!(
            packet
                .occurrences()
                .iter()
                .filter(|reference| reference.binding == plain)
                .count(),
            2
        );
        let id = binding(packet, "id");
        assert_eq!(
            packet
                .occurrences()
                .iter()
                .filter(|reference| reference.binding == id)
                .count(),
            2
        );
    });
}

#[test]
fn unprovided_blocks_or_patterns_refuse_the_whole_packet_without_new_diagnostics() {
    for source in [
        "<script setup>const id = 1</script><template>{{ id }}</template><style>p { color: v-bind(id +); }</style>",
        "<script setup>const id = 1</script><template>{{ id }}</template><style src=\"external.css\" />",
        "<script src=\"external.js\" /><script setup>const id = 1</script><template>{{ id }}</template>",
        "<script lang=\"coffee\">const id = 1</script><template>{{ id }}</template>",
        "<script setup>const id = 1; const fn = (value = id) => value</script><template>{{ id }}</template>",
    ] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let allocator = Allocator::default();
        let (root, errors) =
            vize_armature::parse(&allocator, &descriptor.template.as_ref().unwrap().content);
        assert!(errors.is_empty());
        let options = SfcCroquisOptions::lint_demand();
        let ordinary =
            super::super::analyze_sfc_descriptor_with_context(&descriptor, Some(&root), options);
        let (analysis, packet) =
            analyze_sfc_descriptor_with_occurrences(&descriptor, Some(&root), options);
        assert!(packet.is_none());
        assert_eq!(ordinary.croquis.to_vir(), analysis.croquis.to_vir());
        assert_eq!(
            serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
            serde_json::to_value(analysis.croquis.semantic_snapshot()).unwrap()
        );
    }
}
