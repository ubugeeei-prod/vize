//! Corpus measurement of rendered template complexity (P4-9b): every
//! project under `VIZE_DAVINCI_RENDERED_CORPUS=<dir>[,<dir>…]` (each
//! immediate subdirectory is one project) runs the real pipeline, the
//! rendered distribution is printed for `complexity-metrics.md`, and two
//! laws are checked on every component:
//!
//! - rendered ≥ own, and rendered == own when a component renders nothing;
//! - rendered is monotone along the render tree: a parent never renders
//!   less than any child it renders.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::path::{Path, PathBuf};

use vize_croquis_cf::{CrossFileAnalyzer, CrossFileOptions, FileId, TemplateScores};
use vize_s0::FxHashMap;

use super::template_section::add;

fn vue_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            if child.file_name().is_some_and(|name| name == "node_modules") {
                continue;
            }
            vue_files(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

fn percentile(sample: &[u32], q: usize) -> u32 {
    let mut sorted = sample.to_vec();
    sorted.sort_unstable();
    if sorted.is_empty() {
        return 0;
    }
    sorted[(q * sorted.len()).div_ceil(100).max(1) - 1]
}

#[derive(Default)]
struct Totals {
    projects: usize,
    files: usize,
    components: usize,
    edges: usize,
    recursive: usize,
    cyclomatic: Vec<u32>,
    cognitive: Vec<u32>,
    renders: Vec<u32>,
}

fn run_project(root: &Path, totals: &mut Totals) {
    let mut files = Vec::new();
    vue_files(root, &mut files);
    if files.is_empty() {
        return;
    }
    let mut analyzer = CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), root);
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        add(&mut analyzer, &file.to_string_lossy(), &source);
    }
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();
    let rendered: FxHashMap<FileId, TemplateScores> = result
        .template_complexity
        .iter()
        .map(|component| (component.file_id, component.rendered))
        .collect();
    for component in &result.template_complexity {
        let own = component.template.own;
        assert!(
            component.rendered.cyclomatic >= own.cyclomatic,
            "{}",
            component.file_name
        );
        assert!(
            component.rendered.cognitive >= own.cognitive,
            "{}",
            component.file_name
        );
        if component.rendered_components == 0 {
            assert_eq!(component.rendered, own, "{}", component.file_name);
        }
        totals.cyclomatic.push(component.rendered.cyclomatic);
        totals.cognitive.push(component.rendered.cognitive);
        totals.renders.push(component.rendered_components);
        totals.recursive += usize::from(component.recursive);
    }
    for (parent, child) in analyzer.graph().component_usage() {
        let (Some(parent_scores), Some(child_scores)) =
            (rendered.get(&parent), rendered.get(&child))
        else {
            continue;
        };
        totals.edges += 1;
        assert!(
            parent_scores.cyclomatic >= child_scores.cyclomatic
                && parent_scores.cognitive >= child_scores.cognitive,
            "rendered complexity must not shrink toward the root ({parent:?} -> {child:?})"
        );
    }
    totals.projects += 1;
    totals.files += files.len();
    totals.components += result.template_complexity.len();
}

#[test]
fn the_rendered_distribution_over_a_corpus_obeys_its_laws() {
    let Some(roots) = std::env::var_os("VIZE_DAVINCI_RENDERED_CORPUS") else {
        eprintln!("VIZE_DAVINCI_RENDERED_CORPUS unset: nothing to measure");
        return;
    };
    let mut totals = Totals::default();
    for root in roots.to_string_lossy().split(',') {
        let mut projects: Vec<PathBuf> = std::fs::read_dir(root)
            .expect("VIZE_DAVINCI_RENDERED_CORPUS must name directories")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|path| path.is_dir())
            .collect();
        projects.sort();
        for project in projects {
            run_project(&project, &mut totals);
        }
    }
    assert!(totals.components > 0, "the corpus scored no component");
    eprintln!(
        "rendered corpus: projects={} files={} components={} render-edges={} recursive={}",
        totals.projects, totals.files, totals.components, totals.edges, totals.recursive
    );
    eprintln!("| metric | n | p50 | p90 | p95 | p99 | max |");
    eprintln!("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for (name, sample) in [
        ("rendered cyclomatic", &totals.cyclomatic),
        ("rendered cognitive", &totals.cognitive),
        ("rendered components", &totals.renders),
    ] {
        eprintln!(
            "| {name} | {} | {} | {} | {} | {} | {} |",
            sample.len(),
            percentile(sample, 50),
            percentile(sample, 90),
            percentile(sample, 95),
            percentile(sample, 99),
            sample.iter().max().copied().unwrap_or(0)
        );
    }
}
