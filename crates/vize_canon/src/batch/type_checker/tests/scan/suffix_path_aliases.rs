//! Authored #3984 suffix-alias corpus through the existing explicit scan API.

use super::super::{BatchTypeChecker, create_project_case, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typecheck/suffix-path-aliases-3984/"
);

fn source(name: &str) -> vize_carton::String {
    std::fs::read_to_string(std::path::Path::new(FIXTURE).join(name))
        .unwrap()
        .into()
}

#[test]
fn app_only_suffix_alias_reports_and_repairs_the_authored_child() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "suffix-path-aliases-3984",
        &[
            ("src/App.vue", &source("App.vue.txt")),
            (
                "src/components/api/DirectiveTable.vue",
                &source("DirectiveTable.vue.txt"),
            ),
        ],
    );
    std::fs::write(root.join("tsconfig.json"), source("tsconfig.json.txt")).unwrap();
    let app = root.join("src/App.vue");
    let child = root.join("src/components/api/DirectiveTable.vue");
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_paths(std::slice::from_ref(&app)).unwrap();
    assert_eq!(
        checker.file_count(),
        2,
        "the reachable child is a real root"
    );

    let broken = checker.check_project().unwrap();
    assert!(!broken.success);
    assert_eq!(broken.diagnostics.len(), 1, "{:#?}", broken.diagnostics);
    let diagnostic = &broken.diagnostics[0];
    assert_eq!(diagnostic.file, child);
    assert_eq!(diagnostic.code, Some(2322));
    assert_eq!((diagnostic.line, diagnostic.column), (1, 6));
    assert_eq!(diagnostic.severity, 1);
    assert_eq!(
        diagnostic.message,
        "Type 'number' is not assignable to type 'string'."
    );

    std::fs::write(&child, source("DirectiveTable.repaired.vue.txt")).unwrap();
    // Repeat the App-only cold scan; persistent disk invalidation is a separate law.
    drop(checker);
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_paths(std::slice::from_ref(&app)).unwrap();
    assert_eq!(checker.file_count(), 2);
    let repaired = checker.check_project().unwrap();
    assert!(repaired.success, "{:#?}", repaired.diagnostics);
    assert!(
        repaired.diagnostics.is_empty(),
        "{:#?}",
        repaired.diagnostics
    );
    std::fs::remove_dir_all(root).unwrap();
}
