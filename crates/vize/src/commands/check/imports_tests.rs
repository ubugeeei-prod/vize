use super::*;
use vize_carton::path::canonicalize_non_verbatim;

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn collects_relative_ts_and_vue_imports_transitively() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    let app = write(
        &root,
        "src/App.vue",
        "<script setup lang=\"ts\">\nimport type { Sibling } from './types'\nimport Child from './Child.vue'\nconst x: Sibling = { a: 1 }\n</script>\n<template><Child /></template>\n",
    );
    let types = write(
        &root,
        "src/types.ts",
        "export interface Sibling { a: number }\n",
    );
    let child = write(
        &root,
        "src/Child.vue",
        "<script setup lang=\"ts\">\nimport { helper } from './nested/util'\n</script>\n<template><div /></template>\n",
    );
    let util = write(&root, "src/nested/util.ts", "export const helper = 1\n");

    let discovered = collect_transitive_local_imports(
        std::slice::from_ref(&app),
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    let canon = canonicalize_non_verbatim;
    assert_eq!(discovered, vec![canon(&types), canon(&child), canon(&util)]);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ignores_bare_and_missing_specifiers() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-bare-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let entry = write(
        &root,
        "entry.ts",
        "import { ref } from 'vue'\nimport { gone } from './missing'\nexport const a = ref(0)\nvoid gone\n",
    );

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );
    assert!(discovered.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn jsx_imports_are_resolved_only_when_jsx_typecheck_is_enabled() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-jsx-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    let entry = write(&root, "src/entry.tsx", "import './Panel.jsx'\n");
    let panel = write(&root, "src/Panel.jsx", "const Panel = () => <div />\n");

    let disabled = collect_transitive_local_imports(
        &[entry.clone()],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );
    let enabled = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        true,
        None,
    );

    assert_eq!(disabled, Vec::<PathBuf>::new());
    assert_eq!(enabled, vec![canonicalize_non_verbatim(&panel)]);

    let _ = std::fs::remove_dir_all(&root);
}
