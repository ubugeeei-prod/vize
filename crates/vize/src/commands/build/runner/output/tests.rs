use std::path::{Path, PathBuf};

use super::{OutputError, plan_inputs, preflight_outputs};
use crate::commands::build::{OutputFormat, ScriptExtension};
use vize_carton::cstr;

#[test]
fn preserves_directories_for_duplicate_basenames() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("src");
    let files = vec![root.join("a/index.vue"), root.join("b/index.vue")];

    let planned = plan_inputs(files.clone(), std::slice::from_ref(&root)).unwrap();

    assert_eq!(relative_sources(&planned, &root), files);
    preflight_outputs(
        &planned,
        &case.path().join("dist"),
        OutputFormat::Js,
        ScriptExtension::Downcompile,
    )
    .unwrap();
}

#[test]
fn planning_is_independent_of_file_and_root_order() {
    let case = tempfile::tempdir().unwrap();
    let packages = case.path().join("packages");
    let alpha = packages.join("alpha/src");
    let beta = packages.join("beta/src");
    let forward_files = vec![alpha.join("index.vue"), beta.join("index.vue")];
    let reverse_files = forward_files.iter().rev().cloned().collect::<Vec<_>>();

    let forward = plan_inputs(forward_files, &[alpha.clone(), beta.clone()]).unwrap();
    let reverse = plan_inputs(reverse_files, &[beta, alpha]).unwrap();

    assert_eq!(planned_pairs(&forward), planned_pairs(&reverse));
    assert_eq!(
        forward
            .iter()
            .map(|input| input.relative_source.as_path())
            .collect::<Vec<_>>(),
        [
            Path::new("alpha/src/index.vue"),
            Path::new("beta/src/index.vue")
        ]
    );
}

#[test]
fn empty_searched_root_does_not_later_reshape_existing_outputs() {
    let case = tempfile::tempdir().unwrap();
    let packages = case.path().join("packages");
    let alpha = packages.join("alpha/src");
    let beta = packages.join("beta/src");
    let alpha_file = alpha.join("index.vue");
    let beta_file = beta.join("index.vue");

    let initial = plan_inputs(vec![alpha_file.clone()], &[alpha.clone(), beta.clone()]).unwrap();
    let expanded = plan_inputs(vec![alpha_file, beta_file], &[alpha, beta]).unwrap();

    assert_eq!(initial[0].relative_source, expanded[0].relative_source);
    assert_eq!(initial[0].relative_source, Path::new("alpha/src/index.vue"));
}

#[test]
fn rejects_parent_file_and_child_directory_collision() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("src");
    let files = vec![root.join("a.vue"), root.join("a.js/index.vue")];
    let planned = plan_inputs(files, std::slice::from_ref(&root)).unwrap();

    let error = preflight_outputs(
        &planned,
        &case.path().join("dist"),
        OutputFormat::Js,
        ScriptExtension::Downcompile,
    )
    .unwrap_err();

    assert!(matches!(error, OutputError::Collision { .. }));
    let message = cstr!("{error}");
    assert!(message.contains("a.vue"), "{message}");
    assert!(message.contains("a.js/index.vue"), "{message}");
    assert!(message.contains("a.js/index.js"), "{message}");
}

#[test]
fn rejects_case_only_output_collision_portably() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("src");
    let planned = plan_inputs(
        vec![root.join("Card.vue"), root.join("card.vue")],
        std::slice::from_ref(&root),
    )
    .unwrap();

    let error = preflight_outputs(
        &planned,
        &case.path().join("dist"),
        OutputFormat::Js,
        ScriptExtension::Downcompile,
    )
    .unwrap_err();

    assert!(matches!(error, OutputError::Collision { .. }));
}

fn relative_sources(planned: &[super::PlannedInput], root: &Path) -> Vec<PathBuf> {
    planned
        .iter()
        .map(|input| root.join(&input.relative_source))
        .collect()
}

fn planned_pairs(planned: &[super::PlannedInput]) -> Vec<(PathBuf, PathBuf)> {
    planned
        .iter()
        .map(|input| (input.source.clone(), input.relative_source.clone()))
        .collect()
}
