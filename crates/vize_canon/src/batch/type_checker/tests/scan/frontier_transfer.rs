//! Real explicit batch roots use the scoped walk shared with editor reuse.
//! The actual caller reaches a pinned spawn error; this supplies only graph proof.
#![cfg(unix)]

use std::path::PathBuf;

use crate::batch::type_checker::{BatchTypeChecker, BatchTypeCheckerOptions};
use crate::virtual_ts::ProjectionMapping;
use std::os::unix::fs::PermissionsExt;
use vize_carton::{FxHashSet, String};

const WITHOUT: &str =
    "<script setup lang='ts'>const value = 1;</script><template>{{ value }}</template>\n";
const WITH: &str = "<script setup lang='ts'>import { value } from './T';</script><template>{{ value }}</template>\n";
const TARGET: &str = "// é🦀\r\nexport { value } from './U';\r\n";
const DESCENDANT: &str = "export const value = 'whole target';\n";

struct Case {
    root: PathBuf,
    paths: Vec<PathBuf>,
    target: PathBuf,
    descendant: PathBuf,
    checker: BatchTypeChecker,
}

impl Case {
    fn new(name: &str) -> Self {
        let root = super::super::create_project_case_without_node_modules(
            name,
            &[
                ("src/A.vue", WITHOUT),
                ("src/B.vue", WITH),
                ("src/T.ts", TARGET),
                ("src/U.ts", DESCENDANT),
            ],
        )
        .canonicalize()
        .unwrap();
        let mut paths = vec![root.join("src/A.vue"), root.join("src/B.vue")];
        paths.sort();
        let mut checker = checker(&root);
        checker.scan_paths(&paths).unwrap();
        let target = root.join("src/T.ts");
        let descendant = root.join("src/U.ts");
        assert!(!checker.project.is_declaration_root(&target));
        assert!(!checker.project.is_declaration_root(&descendant));
        assert!(checker.project.find_by_original(&target).is_some());
        assert!(checker.project.find_by_original(&descendant).is_some());
        assert_eq!(owners(&checker, &target), vec![paths[1].clone()]);
        assert_eq!(owners(&checker, &descendant), vec![target.clone()]);
        Self {
            root,
            paths,
            target,
            descendant,
            checker,
        }
    }

    fn patch(&mut self, paths: &[PathBuf]) {
        let before = self.checker.project.registered_original_paths_sorted();
        assert!(before.contains(&self.paths[0]));
        assert!(before.contains(&self.paths[1]));
        let error = self.checker.check_incremental_snapshot(paths).unwrap_err();
        match error {
            crate::batch::error::CorsaError::CorsaExecution { exit_code, message } => {
                assert_eq!(exit_code, -1);
                assert_eq!(
                    message.as_str(),
                    format!(
                        "Failed to start Corsa API session: {}",
                        std::io::Error::from_raw_os_error(13),
                    )
                );
            }
            other => panic!("expected the unchanged terminal executor spawn error, got {other:?}"),
        }
        assert!(self.checker.project.is_declaration_root(&self.paths[0]));
        assert!(self.checker.project.is_declaration_root(&self.paths[1]));
        assert!(!self.checker.project.is_declaration_root(&self.target));
        assert!(!self.checker.project.is_declaration_root(&self.descendant));
        let metrics = self.checker.incremental_metrics();
        assert_eq!(metrics.session_starts, 0);
        assert_eq!(metrics.session_reuses, 0);
    }

    fn assert_cold_equal(&self) {
        let before = facts(&self.checker);
        let mut cold = checker(&self.root);
        cold.scan_paths(&self.paths).unwrap();
        cold.project.materialize().unwrap();
        assert_eq!(before, facts(&cold));
    }

    fn assert_owners(&self, target: Vec<PathBuf>, descendant: Vec<PathBuf>) {
        assert_eq!(owners(&self.checker, &self.target), target);
        assert_eq!(owners(&self.checker, &self.descendant), descendant);
    }
}

// Existing installed_sources constructor precedent; no executable permission.
fn checker(root: &std::path::Path) -> BatchTypeChecker {
    let unused = root.join("unused-tsgo");
    std::fs::write(&unused, "unused fixture executable").unwrap();
    std::fs::set_permissions(&unused, std::fs::Permissions::from_mode(0o600)).unwrap();
    BatchTypeChecker::with_options_and_corsa_path(
        root,
        BatchTypeCheckerOptions::default(),
        Some(&unused),
    )
    .unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct Facts {
    documents: Vec<Document>,
    originals: Vec<PathBuf>,
    dependency_owners: Vec<(PathBuf, Vec<PathBuf>)>,
    files: Vec<(PathBuf, Vec<u8>)>,
    links: Vec<(PathBuf, PathBuf)>,
    queries: Vec<PathBuf>,
    producer_diagnostics: std::string::String,
}

fn facts(checker: &BatchTypeChecker) -> Facts {
    let mut paths = checker.project.registered_original_paths_sorted();
    // Retain released targets in the compared vector, even while unregistered.
    paths.extend(
        ["A.vue", "B.vue", "T.ts", "U.ts"]
            .map(|name| checker.project.project_root().join("src").join(name)),
    );
    paths.sort();
    paths.dedup();
    let mut links = checker
        .project
        .desired_package_links()
        .into_iter()
        .collect::<Vec<_>>();
    links.sort();
    let mut files = checker
        .project
        .expected_materialized_files()
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect::<Vec<_>>();
    files.sort();
    Facts {
        documents: documents(checker),
        originals: checker.project.registered_original_paths_sorted(),
        dependency_owners: paths
            .into_iter()
            .map(|path| {
                let owned_by = owners(checker, &path);
                (path, owned_by)
            })
            .collect(),
        files,
        links,
        queries: checker
            .project
            .editor_query_paths(&checker.project.project_root().join("src/A.vue")),
        producer_diagnostics: format!("{:?}", checker.project.diagnostics()),
    }
}

fn owners(checker: &BatchTypeChecker, path: &std::path::Path) -> Vec<PathBuf> {
    checker
        .project
        .dependency_owners_for_sources(&FxHashSet::from_iter([path.to_path_buf()]))
}

impl Drop for Case {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Document {
    path: PathBuf,
    source_path: PathBuf,
    source: String,
    code: String,
    projection: Option<ProjectionMapping>,
    imports: crate::batch::ImportSourceMap,
}

fn documents(checker: &BatchTypeChecker) -> Vec<Document> {
    checker
        .project
        .virtual_files_sorted()
        .into_iter()
        .map(|file| Document {
            path: file.virtual_path.clone(),
            source_path: file.original_path.clone(),
            source: checker
                .project
                .original_content_for_virtual(&file.virtual_path)
                .unwrap_or_default()
                .into(),
            code: file.content.clone(),
            projection: file
                .source_map
                .sfc_map
                .as_ref()
                .map(|map| map.projection().clone()),
            imports: file.source_map.import_map.clone(),
        })
        .collect()
}

#[test]
fn batch_sorted_owner_transfer_reregisters_the_pruned_target_and_its_descendants() {
    let mut case = Case::new("scoped-batch-frontier-transfer");
    std::fs::write(&case.paths[0], WITH).unwrap();
    std::fs::write(&case.paths[1], WITHOUT).unwrap();
    // The sorted queue [A,B] pops B first. It releases B→T and prunes inferred
    // T→U before A resolves T, which is still a readable disk file.
    let changed = case.paths.clone();
    case.patch(&changed);
    assert!(case.target.is_file());
    assert!(case.descendant.is_file());
    assert!(
        case.checker
            .project
            .find_by_original(&case.target)
            .is_some()
    );
    assert!(
        case.checker
            .project
            .find_by_original(&case.descendant)
            .is_some()
    );
    case.assert_owners(vec![case.paths[0].clone()], vec![case.target.clone()]);
    case.assert_cold_equal();
}

#[test]
fn batch_retained_current_registration_keeps_the_target_closure_whole() {
    let mut case = Case::new("scoped-batch-frontier-retained");
    std::fs::write(&case.paths[0], WITH).unwrap();
    case.patch(&[case.paths[0].clone()]);
    assert!(
        case.checker
            .project
            .find_by_original(&case.target)
            .is_some()
    );
    assert!(
        case.checker
            .project
            .find_by_original(&case.descendant)
            .is_some()
    );
    case.assert_owners(case.paths.clone(), vec![case.target.clone()]);
    case.assert_cold_equal();
}

#[test]
fn batch_visited_identity_alone_cannot_retain_a_released_registration() {
    let mut case = Case::new("scoped-batch-frontier-release");
    std::fs::write(&case.paths[1], WITHOUT).unwrap();
    case.patch(&[case.paths[1].clone()]);
    assert!(case.target.is_file());
    assert!(
        case.checker
            .project
            .find_by_original(&case.target)
            .is_none()
    );
    assert!(
        case.checker
            .project
            .find_by_original(&case.descendant)
            .is_none()
    );
    case.assert_owners(Vec::new(), Vec::new());
    case.assert_cold_equal();
    // A later acquisition must recover both currently absent registrations.
    std::fs::write(&case.paths[0], WITH).unwrap();
    case.patch(&[case.paths[0].clone()]);
    assert!(
        case.checker
            .project
            .find_by_original(&case.target)
            .is_some()
    );
    assert!(
        case.checker
            .project
            .find_by_original(&case.descendant)
            .is_some()
    );
    case.assert_owners(vec![case.paths[0].clone()], vec![case.target.clone()]);
    case.assert_cold_equal();
}
