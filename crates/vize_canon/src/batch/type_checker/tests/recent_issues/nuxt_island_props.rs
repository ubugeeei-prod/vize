//! A serialized island constructor must retain its generated SFC's own props.

use super::super::{create_project_case, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeChecker, TypeChecker};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/nuxt-island-props-13820/",
            $name
        ))
    };
}

#[test]
fn serialized_islands_check_wrong_and_missing_props_like_plain_components() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "nuxt-island-props-13820",
        &[
            ("tsconfig.json", fixture!("tsconfig.json.txt")),
            ("src/App.vue", fixture!("App.vue.txt")),
            (
                "src/ServerProps.server.vue",
                fixture!("ServerProps.server.vue.txt"),
            ),
            ("src/components.d.ts", fixture!("components.d.ts.txt")),
        ],
    );
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.line + 1, diagnostic.column + 1, diagnostic.code))
        .collect();
    diagnostics.sort();
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(
        diagnostics,
        [
            (8, 11, Some(2322)),
            (9, 4, Some(2345)),
            (11, 17, Some(2322)),
            (12, 4, Some(2345)),
        ],
        "{:#?}",
        result.diagnostics
    );
}
