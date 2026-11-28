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

#[test]
fn declaration_snapshots_preserve_parent_types_and_importer_package_identity() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let fixture: serde_json::Value =
        serde_json::from_str(fixture!("reference-snapshots.json")).unwrap();
    let files = fixture["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap(),
                file["source"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let root = create_project_case_without_node_modules("nuxt-shadowing-snapshots", &files);
    let root = vize_carton::path::canonicalize_non_verbatim(&root);
    let parent = root.join("node_modules/server-kit/dist/parent.d.ts");
    let child = root.join("node_modules/server-kit/dist/child.d.ts");
    let parent_original = std::fs::read_to_string(&parent).unwrap();
    let child_original = std::fs::read_to_string(&child).unwrap();
    let options = BatchTypeCheckerOptions {
        virtual_ts_options: VirtualTsOptions {
            auto_import_stubs: vec![vize_carton::cstr!(
                "/// <reference path=\"{}\" />",
                parent.display()
            )],
            ..Default::default()
        },
        ..Default::default()
    };
    let mut before = BatchTypeChecker::with_options(&root, options.clone()).unwrap();
    before.scan_project().unwrap();
    let old = before.check_project().unwrap();
    assert_eq!(
        old.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![Some(2305)],
        "{old:?}"
    );
    drop(before);
    let mut checker = BatchTypeChecker::with_options(&root, options).unwrap();
    checker.scan_project().unwrap();
    let snapshots = vec![
        (parent.clone(), vize_carton::cstr!("/// <reference path=\"{}\" />\nimport type {{ real }} from 'vue';\nexport {{}}; declare global {{ const fromParent: typeof real }}\n", child.display())),
        (child.clone(), "import type { Marker } from './types';\nexport {}; declare global { const fromChild: Marker }\n".into()),
    ];
    checker.register_declaration_snapshots(&snapshots).unwrap();
    let result = checker.check_project().unwrap();
    assert_eq!(result.diagnostics.len(), 1, "{result:?}");
    assert_eq!(result.diagnostics[0].code, Some(2322));
    assert_eq!(result.diagnostics[0].line, 4);
    assert_eq!(std::fs::read_to_string(&parent).unwrap(), parent_original);
    assert_eq!(std::fs::read_to_string(&child).unwrap(), child_original);
    drop(checker);
    std::fs::remove_dir_all(&root).unwrap();
}
