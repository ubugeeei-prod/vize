use super::{BatchTypeChecker, TypeChecker, create_project_case, resolve_test_tsgo_binary};

#[test]
fn lowered_jsx_imports_refresh_types_and_native_extension_priority() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let source = "<script setup lang=\"ts\">\nimport { value } from './entry';\nconst number: number = value;\nvoid number;\n</script>";
    let root = create_project_case(
        "incremental-jsx-module-identity",
        &[
            ("src/App.vue", source),
            ("src/entry.tsx", "export const value = 123;\n"),
        ],
    );
    let app = root.join("src/App.vue");
    let jsx = root.join("src/entry.tsx");
    let script = root.join("src/entry.ts");
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.enable_jsx_typecheck();
    checker.scan_project().unwrap();
    let identities = |result: &crate::batch::TypeCheckResult| {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.file.clone(),
                    diagnostic.code,
                    diagnostic.line,
                    diagnostic.column,
                )
            })
            .collect::<Vec<_>>()
    };
    let clean = checker.check_incremental(&[]).unwrap();
    assert_eq!(identities(&clean), Vec::new());

    std::fs::write(&jsx, "export const value = 'wrong';\n").unwrap();
    let broken = checker
        .check_incremental(std::slice::from_ref(&jsx))
        .unwrap();
    assert_eq!(identities(&broken), vec![(app.clone(), Some(2322), 2, 6)]);

    // A newly created .ts wins over .tsx in the same native project. The
    // importer must return to its authored edge rather than retain a rewrite
    // to the previous lower-priority companion.
    std::fs::write(&script, "export const value = 456;\n").unwrap();
    let preferred = checker
        .check_incremental(std::slice::from_ref(&script))
        .unwrap();
    assert_eq!(identities(&preferred), Vec::new());
    std::fs::remove_file(&script).unwrap();
    let restored = checker
        .check_incremental(std::slice::from_ref(&script))
        .unwrap();
    assert_eq!(identities(&restored), vec![(app, Some(2322), 2, 6)]);

    std::fs::write(&jsx, "export const value = 789;\n").unwrap();
    let repaired = checker.check_incremental(&[jsx]).unwrap();
    assert_eq!(identities(&repaired), Vec::new());
    assert_eq!(checker.incremental_metrics().session_starts, 1);
    drop(checker);
    std::fs::remove_dir_all(root).unwrap();
}
