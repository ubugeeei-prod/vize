use super::{TypeSourceSnapshot, source_path};
use crate::script::ScriptCompileContext;
use crate::script::context::external_types::resolution::resolve_import_path;
use std::fs;
use vize_croquis::types::{ResolvedTypeWorld, TypeLookup};

fn exported_signature(world: &ResolvedTypeWorld) -> String {
    let mut modules: Vec<_> = world
        .modules
        .iter()
        .filter(|(path, _)| *path != &world.root_module)
        .collect();
    modules.sort_by(|(left, _), (right, _)| left.cmp(right));
    format!("{modules:?}")
}

#[test]
fn symlink_workspace_package_retains_compatibility_paths_and_complete_exported_worlds() {
    let dir = tempfile::tempdir().unwrap();
    let workspace = dir.path().join("workspace-package");
    let project = dir.path().join("project");
    let scope = project.join("node_modules/@scope");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&scope).unwrap();
    fs::write(workspace.join("package.json"), r#"{"types":"entry.d.ts"}"#).unwrap();
    fs::write(
        workspace.join("entry.d.ts"),
        "import type { Nested } from './nested'; export type Public = { nested: Nested }; export type { Nested } from './nested';",
    )
    .unwrap();
    fs::write(
        workspace.join("nested.d.ts"),
        "export type Nested = { value: string }",
    )
    .unwrap();
    std::os::unix::fs::symlink(&workspace, scope.join("workspace")).unwrap();
    let source = "import type { Public, Nested } from '@scope/workspace'; type Props = Public & { local: Nested };";
    let sources = TypeSourceSnapshot::default();
    let importer = source_path(&project.join("First.vue"));
    let legacy_entry = resolve_import_path(&importer, "@scope/workspace").unwrap();
    let legacy_nested = resolve_import_path(&legacy_entry, "./nested").unwrap();
    assert_eq!(
        sources.resolve_import(&importer, "@scope/workspace"),
        Some(legacy_entry.clone())
    );
    let mut expected_modules = vec![
        importer.to_string_lossy().into_owned(),
        legacy_entry.to_string_lossy().into_owned(),
        legacy_nested.to_string_lossy().into_owned(),
    ];
    expected_modules.sort();
    let first = ScriptCompileContext::new(source).resolve_type_world_with_sources(
        importer.to_str().unwrap(),
        None,
        false,
        &sources,
    );
    let mut admitted_modules: Vec<_> = first.modules.keys().map(|path| path.as_str()).collect();
    admitted_modules.sort();
    assert_eq!(admitted_modules, expected_modules);
    assert!(first.modules.values().all(|module| module.complete));
    for (name, path) in [("Public", &legacy_entry), ("Nested", &legacy_nested)] {
        let TypeLookup::Found(id) = first.resolve(&first.root_module, name) else {
            panic!("workspace exported type must resolve: {name}")
        };
        assert_eq!(id.module.as_str(), path.to_string_lossy());
    }
    // Compare every dependency declaration, import target, export and span,
    // including forwarded module identities, for a sibling cache consumer and
    // a fresh snapshot. Prop-only parity would miss path/fact changes here.
    for snapshot in [&sources, &TypeSourceSnapshot::default()] {
        let sibling = ScriptCompileContext::new(source).resolve_type_world_with_sources(
            project.join("Second.vue").to_str().unwrap(),
            None,
            false,
            snapshot,
        );
        assert_eq!(exported_signature(&first), exported_signature(&sibling));
    }
}
