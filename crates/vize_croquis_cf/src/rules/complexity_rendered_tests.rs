//! Rendered template complexity through real SFCs (P4-9b fixtures): a
//! recursive component, a child shared by two parents, and an aliased
//! import. Each SFC is analyzed from its source (script setup + template),
//! so component edges come from the analyzer's own import resolution.
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::path::Path;

use crate::{CrossFileAnalyzer, CrossFileOptions, CrossFileResult};
use vize_armature::parse;
use vize_carton::Allocator;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};

fn analyze(script: &str, template: &str) -> Croquis {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "fixture templates parse cleanly");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    analyzer.finish()
}

fn add(analyzer: &mut CrossFileAnalyzer, path: &str, script: &str, template: &str) {
    let source = vize_carton::cstr!(
        "<script setup lang=\"ts\">\n{script}\n</script>\n\n<template>{template}</template>\n"
    );
    analyzer.add_file_with_analysis(Path::new(path), source.as_str(), analyze(script, template));
}

fn run(files: &[(&str, &str, &str)]) -> CrossFileResult {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    for (path, script, template) in files {
        add(&mut analyzer, path, script, template);
    }
    analyzer.rebuild_component_edges();
    analyzer.analyze()
}

/// `(file, own cyclomatic, own cognitive, rendered cyclomatic, rendered
/// cognitive, rendered components, recursive)` in report order.
type Row = (String, u32, u32, u32, u32, u32, bool);

fn rows(result: &CrossFileResult) -> Vec<Row> {
    result
        .template_complexity
        .iter()
        .map(|component| {
            (
                component.file_name.to_string(),
                component.template.own.cyclomatic,
                component.template.own.cognitive,
                component.rendered.cyclomatic,
                component.rendered.cognitive,
                component.rendered_components,
                component.recursive,
            )
        })
        .collect()
}

fn row(file: &str, own: (u32, u32), rendered: (u32, u32), components: u32, recursive: bool) -> Row {
    (
        file.into(),
        own.0,
        own.1,
        rendered.0,
        rendered.1,
        components,
        recursive,
    )
}

#[test]
fn a_child_shared_by_two_parents_counts_once_per_render_tree() {
    let card = r#"<article v-if="item.visible"><h2>{{ item.title ?? 'Untitled' }}</h2></article>"#;
    let result = run(&[
        (
            "Page.vue",
            "import Card from './Card.vue'\nimport Grid from './Grid.vue'",
            r#"<main><Card v-if="hero" :item="hero" /><Grid :items="items" /><Card :item="footer" /></main>"#,
        ),
        (
            "Grid.vue",
            "import Card from './Card.vue'",
            r#"<ul><li v-for="item in items" :key="item.id"><Card :item="item" /></li></ul>"#,
        ),
        ("Card.vue", "defineProps<{ item: Item }>()", card),
        (
            "Sidebar.vue",
            "import Card from './Card.vue'",
            r#"<aside><Card v-for="item in pinned" :key="item.id" :item="item" /></aside>"#,
        ),
    ]);
    // Card: v-if (2/1) + `??` (+1/+1) = (3, 2). Grid: v-for (2/1).
    // Page: v-if (2/1); renders Grid and Card once each: 2+2+3 = 7, 1+1+2.
    // Sidebar: v-for (2/1); renders Card once: (5, 3).
    assert_eq!(
        rows(&result),
        vec![
            row("Page.vue", (2, 1), (7, 4), 2, false),
            row("Grid.vue", (2, 1), (5, 3), 1, false),
            row("Sidebar.vue", (2, 1), (5, 3), 1, false),
            row("Card.vue", (3, 2), (3, 2), 0, false),
        ]
    );
}

#[test]
fn recursion_is_counted_once() {
    let result = run(&[
        (
            "Tree.vue",
            "import TreeNode from './TreeNode.vue'",
            r#"<TreeNode v-if="root" :node="root" />"#,
        ),
        (
            "TreeNode.vue",
            "import TreeNode from './TreeNode.vue'",
            r#"<li>{{ node.label }}<ul v-if="node.open && node.children"><TreeNode v-for="child in node.children" :key="child.id" :node="child" /></ul></li>"#,
        ),
        (
            "Ping.vue",
            "import Pong from './Pong.vue'",
            r#"<Pong v-if="depth > 0" :depth="depth - 1" />"#,
        ),
        (
            "Pong.vue",
            "import Ping from './Ping.vue'",
            r#"<Ping v-if="depth > 0" :depth="depth - 1" />"#,
        ),
    ]);
    // TreeNode: v-if (+1/+1), `&&` (+1/+1), nested v-for (+1/+2) = (4, 4);
    // its self-edge makes it recursive and adds nothing. Tree: v-if (2, 1)
    // plus TreeNode once. Ping and Pong reach each other: each counts the
    // pair once.
    assert_eq!(
        rows(&result),
        vec![
            row("Tree.vue", (2, 1), (6, 5), 1, false),
            row("TreeNode.vue", (4, 4), (4, 4), 0, true),
            row("Ping.vue", (2, 1), (4, 2), 1, true),
            row("Pong.vue", (2, 1), (4, 2), 1, true),
        ]
    );
}

#[test]
fn an_aliased_import_resolves_to_the_imported_file() {
    let result = run(&[
        (
            "Toolbar.vue",
            "import FancyButton from './Button.vue'",
            r#"<nav><FancyButton v-for="action in actions" :key="action.id" :label="action.label" /></nav>"#,
        ),
        (
            "Button.vue",
            "defineProps<{ label: string; busy?: boolean }>()",
            r#"<button :disabled="busy">{{ busy ? 'Saving' : label }}</button>"#,
        ),
    ]);
    // Button: `?:` (2, 1). Toolbar: v-for (2, 1) + Button through the alias.
    assert_eq!(
        rows(&result),
        vec![
            row("Toolbar.vue", (2, 1), (4, 2), 1, false),
            row("Button.vue", (2, 1), (2, 1), 0, false),
        ]
    );
}

#[test]
fn contributors_carry_file_positions() {
    let result = run(&[(
        "Status.vue",
        "defineProps<{ ok: boolean }>()",
        "\n  <p v-if=\"ok\">fine</p>\n  <p v-else>broken</p>\n",
    )]);
    let contributors: Vec<(&str, u32, u32, u32, u32)> = result.template_complexity[0]
        .template
        .contributors
        .iter()
        .map(|row| {
            (
                row.kind,
                row.line,
                row.column,
                row.cyclomatic,
                row.cognitive,
            )
        })
        .collect();
    assert_eq!(
        contributors,
        vec![("v-if", 6, 12, 1, 1), ("v-else", 7, 3, 0, 1)]
    );
}
