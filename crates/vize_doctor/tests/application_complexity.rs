#![cfg(feature = "application-analysis")]
//! The template-complexity hotspot finding (Davinci P4-9b): a component
//! whose rendered template complexity exceeds the corpus p95 gets one
//! notice, with its own and rendered scores, its largest contributors and
//! every file of its render tree as invalidation inputs.

use vize_armature::parse;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileOptions};
use vize_doctor::application_analysis::{
    RENDERED_COGNITIVE_HOTSPOT_ABOVE, TEMPLATE_COMPLEXITY_HOTSPOT, findings_from_application_graph,
};
use vize_doctor::{DoctorCategory, FindingSeverity};
use vize_s0::{Allocator, cstr};

/// Cognitive 15, cyclomatic 6: three nested `v-if`s (+1, +2, +3), a
/// nested `v-for` (+4) and a `?:` inside it (+5).
const CHILD_TEMPLATE: &str = r#"<div v-if="a"><div v-if="b"><div v-if="c"><p v-for="x in xs">{{ x ? 1 : 2 }}</p></div></div></div>"#;

fn analyze(script: &str, template: &str) -> Croquis {
    let allocator = Allocator::new();
    let (root, _) = parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    analyzer.finish()
}

fn add(analyzer: &mut CrossFileAnalyzer, path: &str, script: &str, template: &str) {
    let source = cstr!("<script setup>\n{script}\n</script>\n<template>{template}</template>\n");
    analyzer.add_file_with_analysis(path, source.as_str(), analyze(script, template));
}

#[test]
fn a_large_render_tree_is_one_hotspot_notice() {
    let mut analyzer =
        CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), "/workspace");
    let names: Vec<_> = (1..=12).map(|index| cstr!("Card{index}")).collect();
    let script = names
        .iter()
        .map(|name| cstr!("import {name} from './{name}.vue'"))
        .collect::<Vec<_>>()
        .join("\n");
    let template = cstr!(
        "<main>{}</main>",
        names
            .iter()
            .map(|name| cstr!("<{name} />"))
            .collect::<Vec<_>>()
            .join("")
    );
    add(
        &mut analyzer,
        "/workspace/src/Board.vue",
        &script,
        &template,
    );
    for name in &names {
        add(
            &mut analyzer,
            &cstr!("/workspace/src/{name}.vue"),
            "defineProps<{ a: boolean }>()",
            CHILD_TEMPLATE,
        );
    }
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();

    let findings = findings_from_application_graph(&analyzer, &result).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "only the board's render tree is a hotspot"
    );
    let finding = &findings[0];
    assert_eq!(finding.code, TEMPLATE_COMPLEXITY_HOTSPOT);
    assert_eq!(finding.category, DoctorCategory::Maintainability);
    assert_eq!(finding.assessment.severity, FindingSeverity::Notice);
    assert_eq!(finding.primary.path, "src/Board.vue");
    const { assert!(12 * 15 > RENDERED_COGNITIVE_HOTSPOT_ABOVE) };
    assert_eq!(
        finding.message,
        "Rendering Board.vue pulls in 12 other components: rendered template complexity is \
         cyclomatic 73 and cognitive 180 (own 1 and 0); the hotspot thresholds are 106 and 139."
    );
    let mut inputs = finding.provenance.invalidation_inputs.clone();
    let board = inputs.remove(0);
    inputs.sort();
    assert_eq!(board, "src/Board.vue");
    let mut expected: Vec<_> = names.iter().map(|name| cstr!("src/{name}.vue")).collect();
    expected.sort();
    assert_eq!(inputs, expected);
}

#[test]
fn a_render_tree_below_the_thresholds_has_no_finding() {
    let mut analyzer =
        CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), "/workspace");
    add(
        &mut analyzer,
        "/workspace/src/Page.vue",
        "import Card from './Card.vue'",
        "<main><Card /><Card /></main>",
    );
    add(
        &mut analyzer,
        "/workspace/src/Card.vue",
        "defineProps<{ a: boolean }>()",
        CHILD_TEMPLATE,
    );
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();

    assert_eq!(result.template_complexity.len(), 2);
    assert_eq!(
        findings_from_application_graph(&analyzer, &result).unwrap(),
        vec![]
    );
}
