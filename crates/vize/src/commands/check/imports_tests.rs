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
    assert_eq!(
        discovered.registrations,
        vec![canon(&types), canon(&child), canon(&util)]
    );

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
    assert!(discovered.registrations.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn collects_current_directory_index_imports() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-dot-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/meter")).unwrap();

    let entry = write(
        &root,
        "src/meter/AfMeterBar.vue",
        r#"<script setup lang="ts">
import { calcPercentage } from "."

const percent = calcPercentage(2, 4)
void percent
</script>
"#,
    );
    let index = write(
        &root,
        "src/meter/index.ts",
        "export const calcPercentage = (num: number, max: number) => num / max\n",
    );

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    assert_eq!(
        discovered.registrations,
        vec![canonicalize_non_verbatim(&index)]
    );

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

    assert_eq!(disabled.registrations, Vec::<PathBuf>::new());
    assert_eq!(
        enabled.registrations,
        vec![canonicalize_non_verbatim(&panel)]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn module_js_specifiers_prefer_module_source_extensions() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-module-js-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    let entry = write(
        &root,
        "src/entry.ts",
        "import { esm } from './esm.mjs'\nimport { common } from './common.cjs'\nvoid esm\nvoid common\n",
    );
    let esm_mts = write(&root, "src/esm.mts", "export const esm = 'mts'\n");
    write(&root, "src/esm.ts", "export const esm = 'ts'\n");
    let common_cts = write(&root, "src/common.cts", "export const common = 'cts'\n");
    write(&root, "src/common.ts", "export const common = 'ts'\n");

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    assert_eq!(
        discovered.registrations,
        vec![
            canonicalize_non_verbatim(&esm_mts),
            canonicalize_non_verbatim(&common_cts),
        ]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn js_specifiers_can_follow_tsx_when_jsx_typecheck_is_enabled() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-js-to-tsx-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    let entry = write(
        &root,
        "src/entry.ts",
        "import { Panel } from './Panel.js'\nvoid Panel\n",
    );
    let panel = write(
        &root,
        "src/Panel.tsx",
        "export const Panel = () => <section />\n",
    );

    let disabled = collect_transitive_local_imports(
        std::slice::from_ref(&entry),
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

    assert_eq!(disabled.registrations, Vec::<PathBuf>::new());
    assert_eq!(
        enabled.registrations,
        vec![canonicalize_non_verbatim(&panel)]
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn collects_only_non_relative_imports_that_need_virtual_rewrites() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-matrix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "~/*": ["src/*"],
      "@root/*": ["*"]
    }
  }
}"#,
    )
    .unwrap();

    let entry = write(
        &root,
        "src/entry.ts",
        r#"import { fromSrcRoot } from "~/lib";
import { fromProjectRoot } from "@root/shared/root";
import type { PanelProps } from "~/components/Panel.vue";
import { widget } from "~/components/Widget";
import { cycleA } from "~/cycles/a";
import { jsAlias } from "~/js-alias.js";
import { ignoredPackage } from "pkg";
import { ignoredMissing } from "@root/node_modules/pkg/index";

void fromSrcRoot;
void fromProjectRoot;
void widget;
void cycleA;
void jsAlias;
void ignoredPackage;
void ignoredMissing;
type _PanelProps = PanelProps;
"#,
    );
    let _lib = write(
        &root,
        "src/lib/index.ts",
        r#"export { leaf } from "./leaf";
export const fromSrcRoot = leaf;
"#,
    );
    let _root_shared = write(
        &root,
        "shared/root.ts",
        "export const fromProjectRoot = 'root';\n",
    );
    let panel = write(
        &root,
        "src/components/Panel.vue",
        r#"<script setup lang="ts">
export interface PanelProps {
  title: string;
}
</script>
"#,
    );
    let _widget = write(
        &root,
        "src/components/Widget.tsx",
        "export const widget = () => null;\n",
    );
    let _cycle_a = write(
        &root,
        "src/cycles/a.ts",
        r#"import { cycleB } from "./b";
export const cycleA = cycleB;
"#,
    );
    let _js_alias = write(&root, "src/js-alias.ts", "export const jsAlias = true;\n");
    let _leaf = write(&root, "src/lib/leaf.ts", "export const leaf = 1;\n");
    let _cycle_b = write(
        &root,
        "src/cycles/b.ts",
        r#"import { cycleA } from "./a";
export const cycleB = cycleA;
"#,
    );
    write(
        &root,
        "node_modules/pkg/index.ts",
        "export const ignoredPackage = 1;\n",
    );

    let aliases = PathAliasResolver::from_tsconfig(Some(&root.join("tsconfig.json")));
    let discovered = collect_transitive_local_imports(
        std::slice::from_ref(&entry),
        &root,
        &mut CanonicalPathCache::default(),
        true,
        Some(&aliases),
    );

    assert_eq!(
        discovered.registrations,
        vec![canonicalize_non_verbatim(&panel)]
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// Every SFC in a project imports the same handful of packages, so a package's
/// declaration closure must be walked once per package source, not once per
/// importer. Without the memo a 1,000-file check re-read and re-scanned Vue's
/// whole declaration graph 1,000 times (#4137). Deleting the closure after the
/// first call proves later callers replay the memo instead of walking again.
#[test]
fn package_aware_registration_walks_a_package_closure_once_per_source() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-package-memo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    write(
        &root,
        "App.vue",
        "<script setup lang=\"ts\"></script>\n<template><div /></template>\n",
    );
    write(
        &root,
        "node_modules/widgets/package.json",
        "{\"name\":\"widgets\",\"types\":\"index.d.ts\"}",
    );
    let entry = write(
        &root,
        "node_modules/widgets/index.d.ts",
        "import './internal';\nexport declare const widget: number;\n",
    );
    write(
        &root,
        "node_modules/widgets/internal.d.ts",
        "import '../../App.vue';\nexport declare const internal: number;\n",
    );

    let mut canonical_paths = CanonicalPathCache::default();
    let entry = canonical_paths.canonicalize(&entry);
    let mut packages = PackageRouteResolver::default();
    let mut cache = registration::VirtualRegistrationCache::default();

    let mut answers = Vec::new();
    for call in 0..8 {
        if call == 1 {
            // A fresh walk would now answer differently, so any later change of
            // answer means the closure was walked again.
            let _ = std::fs::remove_dir_all(root.join("node_modules/widgets"));
        }
        let mut discovery = registration::VirtualRegistrationDiscovery::default();
        let needs_registration = registration::non_relative_import_needs_virtual_registration(
            &entry,
            &mut canonical_paths,
            ImportFileOptions::from(false),
            None,
            Some(&mut packages),
            &mut cache,
            &mut discovery,
        );
        answers.push((
            needs_registration,
            discovery.package_routes.len(),
            discovery.package_sources.len(),
        ));
    }

    assert!(answers[0].0, "the closure reaches an SFC: {:?}", answers[0]);
    assert!(
        answers.windows(2).all(|pair| pair[0] == pair[1]),
        "memoized answers must replay the walked answer: {answers:?}"
    );
    assert_eq!(
        cache.len(),
        1,
        "one package closure keeps one memo entry, however many importers reach it"
    );

    let _ = std::fs::remove_dir_all(&root);
}
