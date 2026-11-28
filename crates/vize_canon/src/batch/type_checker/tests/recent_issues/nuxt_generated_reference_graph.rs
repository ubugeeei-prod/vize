//! The stub forms that carry Nuxt's generated reference graph into the
//! program: a reference to an installed declaration, and file-local
//! component types declared in a namespace of their own.

use super::super::{
    create_project_case_without_node_modules, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::{BatchTypeChecker, BatchTypeCheckerOptions, TypeChecker};
use crate::virtual_ts::VirtualTsOptions;

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/nuxt-generated-reference-graph/",
            $name
        ))
    };
}

/// An installed augmentation the stub file references reaches every program
/// file, as Nitro's `augments.d.mts` does through `.nuxt/nuxt.d.ts` under
/// `tsc`: the augmented event has its `path`, typed. A lazy component typed
/// through a carried alias keeps both its own members and the alias's, and
/// the namespace that carries the alias leaves no global for authored code
/// to name.
#[test]
fn nuxt_generated_reference_graph_reaches_the_program() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case_without_node_modules(
        "nuxt-generated-reference-graph",
        &[
            ("tsconfig.json", fixture!("tsconfig.json.txt")),
            ("src/box.ts", fixture!("box.ts.txt")),
            ("src/use.ts", fixture!("use.ts.txt")),
            (
                "node_modules/server-schema/package.json",
                fixture!("schema-package.json.txt"),
            ),
            (
                "node_modules/server-schema/index.d.ts",
                fixture!("schema-index.d.ts.txt"),
            ),
            (
                "node_modules/.pnpm/server-kit/dist/augments.d.mts",
                fixture!("augments.d.mts.txt"),
            ),
        ],
    );
    let project_root = vize_carton::path::canonicalize_non_verbatim(&root);
    let stub = fixture!("stub.d.ts.txt").replace(
        "__PROJECT_ROOT__",
        &project_root
            .to_str()
            .expect("utf-8 test path")
            .replace('\\', "/"),
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
            7,
            43,
            2322,
            "Type 'number' is not assignable to type 'string'.",
        ),
        (
            8,
            14,
            2322,
            "Type 'string' is not assignable to type 'number'.",
        ),
        (
            9,
            20,
            2322,
            "Type 'false' is not assignable to type 'true'.",
        ),
        (10, 24, 2304, "Cannot find name 'LazyComponent'."),
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
