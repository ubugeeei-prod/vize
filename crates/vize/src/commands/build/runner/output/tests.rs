use std::path::{Path, PathBuf};

use super::{OutputError, plan_inputs, preflight_outputs};
use crate::commands::build::{OutputFormat, ScriptExtension};
use vize_s0::cstr;

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

#[test]
fn deduplicates_aliases_of_the_same_source_and_output() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("src");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let source = root.join("App.vue");
    std::fs::write(&source, "<template><div /></template>").unwrap();

    let planned = plan_inputs(
        vec![source.clone(), nested.join("../App.vue")],
        std::slice::from_ref(&root),
    )
    .unwrap();

    assert_eq!(planned.len(), 1);
    assert_eq!(planned[0].source, source);
    assert_eq!(planned[0].relative_source, Path::new("App.vue"));
}

#[cfg(any(unix, windows))]
#[test]
fn deduplicates_portable_layout_aliases_of_the_same_source() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().join("src");
    std::fs::create_dir_all(&root).unwrap();
    let source = root.join("Target.vue");
    let alias = root.join("target.vue");
    std::fs::write(&source, "<template><div /></template>").unwrap();
    if !alias.exists() && !symlink_file(&source, &alias) {
        return;
    }

    let planned = plan_inputs(vec![source.clone(), alias], std::slice::from_ref(&root)).unwrap();

    assert_eq!(planned.len(), 1);
    assert_eq!(planned[0].source, source);
    preflight_outputs(
        &planned,
        &case.path().join("dist"),
        OutputFormat::Js,
        ScriptExtension::Downcompile,
    )
    .unwrap();
}

#[cfg(unix)]
#[test]
fn preserves_distinct_sources_crossing_a_symlinked_parent() {
    let case = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    let root = case.path().join("src");
    let direct = root.join("Target.vue");
    let linked = root.join("link/../Target.vue");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(external.path().join("nested")).unwrap();
    std::fs::write(&direct, "<template><div>direct</div></template>").unwrap();
    std::fs::write(
        external.path().join("Target.vue"),
        "<template><div>linked</div></template>",
    )
    .unwrap();
    if !symlink_directory(&external.path().join("nested"), &root.join("link")) {
        return;
    }

    let planned = plan_inputs(
        vec![direct.clone(), linked.clone()],
        &[root.clone(), root.join("link/..")],
    )
    .unwrap();

    assert_eq!(planned.len(), 2);
    let error = preflight_outputs(
        &planned,
        &case.path().join("dist"),
        OutputFormat::Js,
        ScriptExtension::Downcompile,
    )
    .unwrap_err();
    let OutputError::Collision {
        first_source,
        first_output,
        second_source,
        second_output,
    } = error
    else {
        panic!("expected output collision");
    };
    let mut sources = [first_source, second_source];
    let mut expected_sources = [direct, linked];
    sources.sort();
    expected_sources.sort();
    assert_eq!(sources, expected_sources);
    assert_eq!(first_output, case.path().join("dist/Target.js"));
    assert_eq!(second_output, first_output);
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

#[cfg(unix)]
fn symlink_directory(source: &Path, target: &Path) -> bool {
    match std::os::unix::fs::symlink(source, target) {
        Ok(()) => true,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::Unsupported | std::io::ErrorKind::PermissionDenied
            ) =>
        {
            false
        }
        Err(error) => panic!("failed to create directory symlink: {error}"),
    }
}

#[cfg(any(unix, windows))]
fn symlink_file(source: &Path, target: &Path) -> bool {
    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(source, target);
    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_file(source, target);

    match result {
        Ok(()) => true,
        #[cfg(windows)]
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => false,
        #[cfg(unix)]
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::Unsupported | std::io::ErrorKind::PermissionDenied
            ) =>
        {
            false
        }
        Err(error) => panic!("failed to create file symlink: {error}"),
    }
}
