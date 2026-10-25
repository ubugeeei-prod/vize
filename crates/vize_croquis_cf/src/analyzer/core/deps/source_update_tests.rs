use crate::{CrossFileAnalyzer, CrossFileOptions, DependencyEdge, FileId};
use vize_croquis::{
    Analyzer, AnalyzerOptions, EffectGraphScript, build_effect_graph_from_sfc_scripts,
};
use vize_l0::{Allocator, cstr};

use DependencyEdge::{
    ComponentUsage as C, DynamicImport as D, EventEmit as E, Import as I, ProvideInject as P,
    ReExport as R, SlotContent as S, TypeImport as T,
};

#[derive(Debug, PartialEq, Eq)]
struct NodeEdges {
    id: u32,
    outgoing: Vec<(u32, DependencyEdge)>,
    incoming: Vec<(u32, DependencyEdge)>,
}

fn node(
    id: u32,
    outgoing: &[(u32, DependencyEdge)],
    incoming: &[(u32, DependencyEdge)],
) -> NodeEdges {
    NodeEdges {
        id,
        outgoing: outgoing.to_vec(),
        incoming: incoming.to_vec(),
    }
}

fn packet(analyzer: &CrossFileAnalyzer) -> Vec<NodeEdges> {
    assert_eq!(analyzer.graph().len(), analyzer.registry().len());
    (0..analyzer.registry().len())
        .map(|id| {
            let id = u32::try_from(id).unwrap();
            let current = analyzer.graph().get_node(FileId::new(id)).unwrap();
            NodeEdges {
                id,
                outgoing: current
                    .imports
                    .iter()
                    .map(|(id, kind)| (id.as_u32(), *kind))
                    .collect(),
                incoming: current
                    .importers
                    .iter()
                    .map(|(id, kind)| (id.as_u32(), *kind))
                    .collect(),
            }
        })
        .collect()
}

fn register_sfc(
    analyzer: &mut CrossFileAnalyzer,
    path: &str,
    script: &str,
    template: &str,
) -> FileId {
    let allocator = Allocator::default();
    let root = vize_armature::Parser::new(&allocator, template).parse().0;
    let mut single = Analyzer::with_options(AnalyzerOptions::full());
    single.analyze_script_setup_with_generic_jsx(script, None);
    single.analyze_template(&root);
    let source =
        cstr!("<script setup lang=\"tsx\">{script}</script>\r\n<template>{template}</template>");
    let effects = build_effect_graph_from_sfc_scripts(
        None,
        Some(EffectGraphScript::new(script, Some("tsx"))),
    )
    .summary();
    analyzer.add_file_with_analysis_and_effect_summary(path, &source, single.finish(), effects)
}

#[test]
fn deleting_an_import_breaks_the_retained_cycle_in_both_graph_directions() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());
    let first = analyzer.add_file(
        "First.ts",
        "import { last } from './Last'; export const first = last",
    );
    analyzer.add_file(
        "Last.ts",
        "import { first } from './First'; export const last = first",
    );
    analyzer.rebuild_import_edges();
    assert_eq!(
        packet(&analyzer),
        vec![node(0, &[(1, I)], &[(1, I)]), node(1, &[(0, I)], &[(0, I)])]
    );
    assert!(!analyzer.analyze().circular_deps.is_empty());

    assert_eq!(
        analyzer.add_file("First.ts", "export const first = '日本語🙂'"),
        first
    );
    let expected = vec![node(0, &[], &[(1, I)]), node(1, &[(0, I)], &[])];
    assert_eq!(packet(&analyzer), expected);
    for _ in 0..3 {
        analyzer.rebuild_import_edges();
        let result = analyzer.analyze();
        assert!(result.circular_deps.is_empty());
        assert_eq!(result.stats.dependency_edges, 1);
        assert_eq!(packet(&analyzer), expected);
    }
}

#[test]
fn runtime_type_transitions_preserve_unchanged_import_and_unrelated_owner_order() {
    let mut analyzer = CrossFileAnalyzer::default();
    analyzer.add_file("B.ts", "export const b = 1; export interface Shape {}");
    analyzer.add_file("C.ts", "export const c = 2");
    let owner = analyzer.add_file(
        "Owner.ts",
        "import { b } from './B'; import { c } from './C'",
    );
    analyzer.add_file("Other.ts", "import { b } from './B'");
    let unchanged = analyzer
        .get_analysis(FileId::new(3))
        .unwrap()
        .semantic_snapshot();
    analyzer.add_file(
        "Owner.ts",
        "import type { Shape } from './B'; import { c } from './C'",
    );
    assert_eq!(
        packet(&analyzer),
        vec![
            node(0, &[], &[(3, I), (2, T)]),
            node(1, &[], &[(2, I)]),
            node(2, &[(1, I), (0, T)], &[]),
            node(3, &[(0, I)], &[]),
        ]
    );
    analyzer.add_file(
        "Owner.ts",
        "import { b } from './B'; import { c } from './C'",
    );
    let expected = vec![
        node(0, &[], &[(3, I), (2, I)]),
        node(1, &[], &[(2, I)]),
        node(2, &[(1, I), (0, I)], &[]),
        node(3, &[(0, I)], &[]),
    ];
    assert_eq!(packet(&analyzer), expected);
    // Source declaration order does not move retained graph edges.
    assert_eq!(
        analyzer.add_file(
            "Owner.ts",
            "import { c } from './C'; import { b } from './B'"
        ),
        owner
    );
    analyzer.rebuild_import_edges();
    assert_eq!(packet(&analyzer), expected);
    assert_eq!(
        analyzer
            .get_analysis(FileId::new(3))
            .unwrap()
            .semantic_snapshot(),
        unchanged
    );
}

#[test]
fn runtime_and_type_imports_of_one_target_are_reconciled_independently() {
    let mut analyzer = CrossFileAnalyzer::default();
    analyzer.add_file("B.ts", "export const b = 1; export interface Shape {}");
    analyzer.add_file(
        "Owner.ts",
        "import { b } from './B'; import type { Shape } from './B'",
    );
    assert_eq!(
        packet(&analyzer),
        vec![
            node(0, &[], &[(1, I), (1, T)]),
            node(1, &[(0, I), (0, T)], &[])
        ]
    );
    analyzer.add_file("Owner.ts", "import type { Shape } from './B'");
    assert_eq!(
        packet(&analyzer),
        vec![node(0, &[], &[(1, T)]), node(1, &[(0, T)], &[])]
    );
    analyzer.add_file(
        "Owner.ts",
        "import { b } from './B'; import type { Shape } from './B'",
    );
    assert_eq!(
        packet(&analyzer),
        vec![
            node(0, &[], &[(1, T), (1, I)]),
            node(1, &[(0, T), (0, I)], &[])
        ]
    );
    analyzer.add_file("Owner.ts", "export const noImports = true");
    assert_eq!(
        packet(&analyzer),
        vec![node(0, &[], &[]), node(1, &[], &[])]
    );
}

#[test]
fn missing_renamed_and_repaired_targets_replace_only_the_edited_owner() {
    let mut analyzer = CrossFileAnalyzer::default();
    analyzer.add_file("Before.ts", "export const value = 1");
    analyzer.add_file("Owner.ts", "import { value } from './Before'");
    analyzer.add_file("Other.ts", "import { value } from './Before'");
    analyzer.add_file("Owner.ts", "import { value } from './Missing'");
    assert_eq!(
        packet(&analyzer),
        vec![
            node(0, &[], &[(2, I)]),
            node(1, &[], &[]),
            node(2, &[(0, I)], &[])
        ]
    );
    analyzer.add_file("Renamed.ts", "export const value = 2");
    analyzer.add_file("Owner.ts", "import { value } from './Renamed'");
    assert_eq!(
        packet(&analyzer),
        vec![
            node(0, &[], &[(2, I)]),
            node(1, &[(3, I)], &[]),
            node(2, &[(0, I)], &[]),
            node(3, &[], &[(1, I)])
        ]
    );
    analyzer.add_file("Owner.ts", "import { value } from './Before'");
    let expected = vec![
        node(0, &[], &[(2, I), (1, I)]),
        node(1, &[(0, I)], &[]),
        node(2, &[(0, I)], &[]),
        node(3, &[], &[]),
    ];
    for _ in 0..3 {
        analyzer.rebuild_import_edges();
        assert_eq!(packet(&analyzer), expected);
    }
}

#[test]
fn sfc_unicode_crlf_jsx_updates_preserve_all_unrelated_edge_kinds() {
    let mut analyzer = CrossFileAnalyzer::default();
    register_sfc(&mut analyzer, "B.vue", "", "<p>旧🙂</p>");
    register_sfc(&mut analyzer, "C.vue", "", "<p>新🙂</p>");
    let owner = register_sfc(
        &mut analyzer,
        "Owner.vue",
        "import B from './B.vue'\r\nconst view = <i />",
        "<B label=\"日本語🙂\" />",
    );
    analyzer.add_file("Other.ts", "import B from './B.vue'");
    for (target, kind) in [(1, D), (0, P), (1, E), (1, S), (0, R)] {
        analyzer.graph.add_edge(owner, FileId::new(target), kind);
    }
    register_sfc(
        &mut analyzer,
        "Owner.vue",
        "import C from './C.vue'\r\nconst view = <b />",
        "<C label=\"復元🙂\" />",
    );
    let expected = vec![
        node(0, &[], &[(3, I), (2, P), (2, R)]),
        node(1, &[], &[(2, D), (2, E), (2, S), (2, I), (2, C)]),
        node(
            2,
            &[(1, D), (0, P), (1, E), (1, S), (0, R), (1, I), (1, C)],
            &[],
        ),
        node(3, &[(0, I)], &[]),
    ];
    assert_eq!(packet(&analyzer), expected);
    for _ in 0..3 {
        analyzer.rebuild_import_edges();
        assert_eq!(packet(&analyzer), expected);
    }
}
