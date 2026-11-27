//! Auto-imported types reach the program as re-exports inside a global
//! augmentation, the form Nuxt generates.

use super::super::{
    create_project_case_without_node_modules, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{BatchTypeChecker, BatchTypeCheckerOptions, TypeChecker};
use crate::virtual_ts::VirtualTsOptions;

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/auto-imported-type-reexports/",
            $name
        ))
    };
}

/// A re-exported type is the real type: its type parameters and their
/// defaults survive, a renamed export answers to its new name, a package type
/// resolves through the package, and a project type is the mirrored module's
/// own declaration, so a value of the directly imported class is assignable
/// to the auto-imported one. An alias of `import('m').T` could keep none of
/// that without restating every parameter list.
#[test]
fn auto_imported_type_reexports_are_the_real_types() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case_without_node_modules(
        "auto-imported-type-reexports",
        &[
            ("tsconfig.json", fixture!("tsconfig.json.txt")),
            ("src/states.ts", fixture!("states.ts.txt")),
            ("src/use.ts", fixture!("use.ts.txt")),
            (
                "node_modules/ui-kit/package.json",
                fixture!("ui-kit-package.json.txt"),
            ),
            (
                "node_modules/ui-kit/index.d.ts",
                fixture!("ui-kit-index.d.ts.txt"),
            ),
        ],
    );
    let project_root = vize_carton::path::canonicalize_non_verbatim(&root);
    let stub = fixture!("stub.d.ts.txt").replace(
        "__PROJECT_ROOT__",
        project_root.to_str().expect("utf-8 test path"),
    );
    let options = BatchTypeCheckerOptions {
        virtual_ts_options: VirtualTsOptions {
            auto_import_stubs: vec![stub.as_str().into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let mut checker = BatchTypeChecker::with_options(&root, options).unwrap();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&root, &diagnostic.file),
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.code,
                diagnostic.message.clone(),
            )
        })
        .collect();
    diagnostics.sort();
    std::fs::remove_dir_all(&root).unwrap();

    let expected = [
        (
            11,
            14,
            2322,
            "Type '\"green\"' is not assignable to type 'ColorName'.",
        ),
        (
            12,
            40,
            2322,
            "Type 'string' is not assignable to type 'number'.",
        ),
        (
            13,
            61,
            2322,
            "Type 'string' is not assignable to type 'boolean | undefined'.",
        ),
        (
            14,
            44,
            2322,
            "Type 'number' is not assignable to type 'string'.",
        ),
        (
            15,
            42,
            2322,
            "Type 'string' is not assignable to type 'number'.",
        ),
        (
            16,
            23,
            2707,
            "Generic type 'Box<T, Label>' requires between 1 and 2 type arguments.",
        ),
        (
            17,
            14,
            2741,
            "Property 'brand' is missing in type '{ describe: () => string; }' but required in type 'Token'.",
        ),
    ]
    .map(|(line, column, code, message)| {
        (
            "src/use.ts".into(),
            line,
            column,
            Some(code),
            message.into(),
        )
    });
    assert_eq!(diagnostics, expected);
}
