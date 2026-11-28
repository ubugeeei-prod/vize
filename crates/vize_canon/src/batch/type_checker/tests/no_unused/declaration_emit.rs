//! Declarations are emitted only after the check reports nothing, so an
//! option that only adds diagnostics must not fail the emit-only program on
//! Canon's generated scaffolding.
//!
//! The emit-only program reads the same generated modules as the check
//! program, but reported its diagnostics raw, in generated coordinates. Under
//! `noUnusedLocals` a `<slot />` outlet helper the check program never reads
//! (`__vizeSlotOutlet`) failed the emit with `TS6133` after the check passed,
//! and left TypeScript's unrewritten outputs behind. `vue-tsc` emits these
//! sources under the same `tsconfig.json` without a diagnostic.

use super::super::{
    BatchTypeChecker, DeclarationEmitOptions, TypeChecker, create_project_case, relative_path,
    resolve_test_tsgo_binary,
};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/declaration-emit-diagnostic-only-options/",
            $name
        ))
    };
}

#[test]
fn diagnostic_only_options_never_fail_declaration_emit() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "declaration-emit-diagnostic-only-options",
        &[
            ("tsconfig.json", fixture!("tsconfig.json.txt")),
            ("src/Panel.vue", fixture!("Panel.vue.txt")),
            ("src/Popover.vue", fixture!("Popover.vue.txt")),
            ("src/index.ts", fixture!("index.ts.txt")),
        ],
    );
    let mut checker = BatchTypeChecker::new(&root).expect("batch checker construction");
    checker.scan_project().expect("project scan");
    let diagnostics = checker
        .check_project()
        .expect("project check")
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(&root, &diagnostic.file),
                diagnostic.code,
                diagnostic.line + 1,
                diagnostic.message,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(diagnostics, Vec::new(), "the authored sources check clean");

    let out_dir = root.join("dist");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()))
        .expect("diagnostic-only options must not fail declaration emit");
    let mut paths = emitted
        .files
        .iter()
        .map(|file| relative_path(&out_dir, &file.path))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(
        paths,
        [
            "Panel.vue.d.ts",
            "Popover.vue.d.ts",
            "__vize_helpers.d.ts",
            "index.d.ts"
        ]
    );
    // The finished outputs import their helpers from the shipped file.
    let mut imports = emitted
        .files
        .iter()
        .filter_map(|file| {
            let name = relative_path(&out_dir, &file.path);
            name.ends_with(".vue.d.ts")
                .then(|| (name, file.content.lines().next().map(String::from)))
        })
        .collect::<Vec<_>>();
    imports.sort();
    let helpers = "import type { $IsAny, $ComponentInput, $ComponentInstanceFor, $IsUnknown, $UsePublicInstance, $PublicEmit, $PublicSlots, $ComponentPublicBase } from \"./__vize_helpers.js\";";
    assert_eq!(
        imports,
        [
            ("Panel.vue.d.ts".into(), Some(helpers.into())),
            ("Popover.vue.d.ts".into(), Some(helpers.into())),
        ]
    );
    let _ = std::fs::remove_dir_all(root);
}
