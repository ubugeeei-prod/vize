//! Reachability and the final import rewrite replay the registration's
//! import-rewrite parse, and produce exactly what parsing again produces.

use std::fs;
use std::path::{Path, PathBuf};

use super::super::VirtualProject;
use super::super::preparation_import_facts_test_snapshot::PreparationImportFactsSnapshot;

fn case_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("module-projection")
        .join(vize_carton::cstr!("{name}-{}", std::process::id()).as_str())
}

fn write(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, content).unwrap();
    path
}

fn fixture(root: &Path) -> Vec<PathBuf> {
    write(
        root,
        "tsconfig.json",
        "{ \"compilerOptions\": { \"strict\": true } }\n",
    );
    write(
        root,
        "src/Child.vue",
        "<script lang=\"ts\">\nexport const named = 1;\n</script>\n<script setup lang=\"ts\">\ndefineProps<{ label: string }>();\n</script>\n<template><span>{{ label }}</span></template>\n",
    );
    write(
        root,
        "src/Lazy.vue",
        "<script setup lang=\"ts\">\nconst count = 1;\n</script>\n<template><b>{{ count }}</b></template>\n",
    );
    write(root, "src/util.ts", "export const util = 1;\n");
    write(root, "shared/Outside.vue", "<template><i /></template>\n");
    vec![
        write(
            root,
            "src/App.vue",
            "<script setup lang=\"ts\">\n// import Ghost from './Ghost.vue'\nimport Child from './Child.vue';\nimport type { named } from './Child.vue';\nimport Outside from '../shared/Outside.vue';\nimport { util } from './util';\nconst lazy = () => import('./Lazy.vue');\nconst text = \"export * from './Text.vue'\";\n</script>\n<template><Child :label=\"String(util)\" /><Outside /></template>\n",
        ),
        write(
            root,
            "src/index.ts",
            "export * from './Child.vue';\nexport { default as App } from './App.vue';\nexport type { util } from './util';\nexport const load = () => import('./Lazy.vue');\n",
        ),
    ]
}

fn prepare(root: &Path, jsx_typecheck: bool, replay: bool) -> PreparationImportFactsSnapshot {
    let paths = fixture(root);
    let mut project = VirtualProject::new(root).unwrap();
    project.set_jsx_typecheck(jsx_typecheck);
    project.register_paths(&paths).unwrap();
    for path in &paths {
        let syntax = project.source_artifacts[path]
            .rewrite_syntax
            .as_ref()
            .unwrap_or_else(|| panic!("{} keeps its registration parse", path.display()));
        // The parse answers for the very text reachability and the final
        // rewrite read back, so neither parses it again.
        let file = project.find_by_original(path).unwrap();
        let walked = project.walked_source(file);
        assert!(
            syntax.reusable_for(
                &walked,
                super::super::editor_source_facts::walked_source_type(&file.virtual_path)
            ),
            "{}",
            path.display()
        );
        if let Some(source) = project.module_source(file) {
            let source_type =
                super::super::build::source_type_for_path(&file.virtual_path).unwrap();
            assert!(
                syntax.reusable_for(&source, source_type),
                "{}",
                path.display()
            );
        }
    }
    if !replay {
        for artifacts in project.source_artifacts.values_mut() {
            artifacts.rewrite_syntax = None;
        }
    }
    project.register_reachable_dependencies().unwrap();
    project.finalize_package_routes().unwrap();
    assert!(
        project
            .source_artifacts
            .values()
            .all(|artifacts| artifacts.rewrite_syntax.is_none()),
        "a registration parse must not outlive its preparation"
    );
    project.preparation_import_facts_snapshot()
}

#[test]
fn replaying_the_registration_parse_matches_parsing_again() {
    for jsx_typecheck in [true, false] {
        let root = case_dir(vize_carton::cstr!("rewrite-syntax-replay-{jsx_typecheck}").as_str());
        let _ = fs::remove_dir_all(&root);
        let replayed = prepare(&root, jsx_typecheck, true);
        let parsed = prepare(&root, jsx_typecheck, false);
        assert_eq!(replayed, parsed, "jsx_typecheck={jsx_typecheck}");
        let _ = fs::remove_dir_all(&root);
    }
}
