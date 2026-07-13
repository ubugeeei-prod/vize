use std::fs;

use vize_atlas::Shared;
use vize_patina::{HelpLevel, Linter};

use super::{apply_sfc_cross_file_lint, build_cross_file_lint_output};

#[test]
fn production_cross_file_path_queries_the_full_atlas_product() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("App.vue");
    let child = dir.path().join("Child.vue");
    fs::write(
        &app,
        "<script setup>import Child from './Child.vue'</script><template><main id=\"shared\"><Child /></main></template>",
    )
    .unwrap();
    fs::write(&child, "<template><p id=\"shared\">child</p></template>").unwrap();
    let paths = vec![app, child];
    let raw_inputs: Vec<_> = paths
        .iter()
        .map(|path| (path.clone(), fs::read_to_string(path).unwrap()))
        .collect();
    let expected = build_cross_file_lint_output(&raw_inputs, HelpLevel::Short, false);
    let run = super::super::pipeline::lint_inputs(
        super::super::pipeline::read_lint_inputs(&paths, false),
        Shared::new(Linter::new()),
        vize_carton::config::VueVersion::V3,
        false,
        false,
        true,
    );
    let (graph, mut files, _) = run.into_parts();
    apply_sfc_cross_file_lint(&graph, &mut files, HelpLevel::Short, false, false);

    assert!(files.iter().all(|file| file.semantics.is_some()));
    let actual: Vec<_> = files
        .iter()
        .map(|file| {
            file.result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "cross-file")
                .map(|diagnostic| vize_carton::cstr!("{diagnostic:?}"))
                .collect::<Vec<_>>()
        })
        .collect();
    let expected: Vec<_> = expected
        .results
        .iter()
        .map(|result| {
            result
                .diagnostics
                .iter()
                .map(|diagnostic| vize_carton::cstr!("{diagnostic:?}"))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn malformed_atlas_sfc_keeps_cross_file_analysis_recoverable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Malformed.vue");
    fs::write(
        &path,
        "<template><div /></template><template><span /></template>",
    )
    .unwrap();
    let run = super::super::pipeline::lint_inputs(
        super::super::pipeline::read_lint_inputs(std::slice::from_ref(&path), false),
        Shared::new(Linter::new()),
        vize_carton::config::VueVersion::V3,
        false,
        false,
        true,
    );
    let (graph, mut files, _) = run.into_parts();
    assert!(files[0].artifact_backed);
    assert!(files[0].semantics.is_none());
    apply_sfc_cross_file_lint(&graph, &mut files, HelpLevel::Short, false, false);

    assert!(files[0].artifact_backed);
}
