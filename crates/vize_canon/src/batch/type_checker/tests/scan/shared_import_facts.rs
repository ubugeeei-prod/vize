//! The combined API retains the complete legacy projection and native errors.

use std::path::{Path, PathBuf};

use super::super::{BatchTypeChecker, create_project_case, resolve_test_tsgo_binary};
use crate::batch::{Diagnostic, SfcBlockType, TypeCheckResult, TypeChecker};
use vize_carton::{String, cstr};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typecheck/cli-preparation-import-facts/"
);

fn source(name: &str) -> String {
    std::fs::read_to_string(Path::new(FIXTURE).join(name))
        .unwrap()
        .into()
}

fn fixture(name: &str) -> PathBuf {
    create_project_case(
        name,
        &[
            ("src/App.vue", &source("App.vue.txt")),
            ("src/left/Late.vue", &source("Late.vue.txt")),
            ("src/right/Late.vue", &source("Late.vue.txt")),
            ("src/left/value.ts", &source("Value.valid.ts.txt")),
            ("src/right/value.ts", &source("Value.invalid.ts.txt")),
        ],
    )
}

fn prepare(root: &Path, combined: bool) -> BatchTypeChecker {
    let mut checker = BatchTypeChecker::new(root).unwrap();
    let app = root.join("src/App.vue");
    if combined {
        checker
            .scan_paths_with_included_sources(std::slice::from_ref(&app))
            .unwrap();
    } else {
        checker.scan_paths(std::slice::from_ref(&app)).unwrap();
        checker.ensure_included_sources().unwrap();
    }
    checker
}

#[test]
fn combined_included_scan_preserves_complete_legacy_project_and_subset_scope() {
    let root = fixture("shared-import-facts-projection");
    {
        let mut subset = BatchTypeChecker::new(&root).unwrap();
        subset.scan_paths(&[root.join("src/App.vue")]).unwrap();
        assert!(
            subset
                .project
                .find_by_original(&root.join("src/right/Late.vue"))
                .is_none(),
            "the original explicit subset API must not admit an include-only file"
        );

        let legacy = prepare(&root, false);
        let combined = prepare(&root, true);
        assert_eq!(
            combined.project.preparation_import_facts_snapshot(),
            legacy.project.preparation_import_facts_snapshot()
        );
        assert_eq!(combined.file_count(), 5);
        for file in [
            "src/left/Late.vue",
            "src/right/Late.vue",
            "src/right/value.ts",
        ] {
            assert!(
                combined
                    .project
                    .find_by_original(&root.join(file))
                    .is_some()
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn combined_included_scan_keeps_identical_source_importers_and_native_repair_exact() {
    // PR source-only lanes explicitly disable native execution. The merge
    // group sets REQUIRE_TSGO=1 and supplies the physical Corsa executable.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = fixture("shared-import-facts-native");
    let diagnostics = |result: &TypeCheckResult| {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let Diagnostic {
                    file,
                    line,
                    column,
                    message,
                    code,
                    severity,
                    block_type,
                } = diagnostic;
                (
                    file.clone(),
                    *line,
                    *column,
                    message.clone(),
                    *code,
                    *severity,
                    *block_type,
                )
            })
            .collect::<Vec<_>>()
    };
    for repaired in [false, true] {
        if repaired {
            std::fs::write(
                root.join("src/right/value.ts"),
                source("Value.valid.ts.txt"),
            )
            .unwrap();
        }
        let legacy = prepare(&root, false);
        let combined = prepare(&root, true);
        assert_eq!(
            combined.project.preparation_import_facts_snapshot(),
            legacy.project.preparation_import_facts_snapshot()
        );
        let expected = legacy.check_project().unwrap();
        let actual = combined.check_project().unwrap();
        assert_eq!(actual.success, expected.success);
        assert_eq!(actual.exit_code, expected.exit_code);
        assert_eq!(diagnostics(&actual), diagnostics(&expected));
        if repaired {
            assert!(actual.success, "{:#?}", actual.diagnostics);
            assert_eq!(actual.exit_code, 0);
            assert_eq!(diagnostics(&actual), Vec::new());
        } else {
            assert!(!actual.success);
            assert_ne!(actual.exit_code, 0);
            assert_eq!(
                diagnostics(&actual),
                vec![(
                    root.join("src/right/Late.vue"),
                    2,
                    6,
                    cstr!("Type 'number' is not assignable to type 'string'."),
                    Some(2322),
                    1,
                    Some(SfcBlockType::ScriptSetup),
                )]
            );
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
