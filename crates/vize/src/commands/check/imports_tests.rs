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
fn collects_absolute_project_imports_transitively() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-abs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    let schema = write(
        &root,
        "types/codegen/schema.ts",
        "export enum DisplayKind { List = 'List' }\n",
    );
    let schema_specifier = schema.with_extension("");
    let entry = write(
        &root,
        "src/entry.ts",
        &format!(
            "import type {{ DisplayKind }} from '{}'\nexport type Props = {{ kind: DisplayKind }}\n",
            schema_specifier.display()
        ),
    );

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    assert_eq!(discovered, vec![canonicalize_non_verbatim(&schema)]);

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

    assert_eq!(discovered, vec![canonicalize_non_verbatim(&index)]);

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

#[test]
fn collects_project_import_graph_edges_without_package_boundaries() {
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
    let lib = write(
        &root,
        "src/lib/index.ts",
        r#"export { leaf } from "./leaf";
export const fromSrcRoot = leaf;
"#,
    );
    let root_shared = write(
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
    let widget = write(
        &root,
        "src/components/Widget.tsx",
        "export const widget = () => null;\n",
    );
    let cycle_a = write(
        &root,
        "src/cycles/a.ts",
        r#"import { cycleB } from "./b";
export const cycleA = cycleB;
"#,
    );
    let js_alias = write(&root, "src/js-alias.ts", "export const jsAlias = true;\n");
    let leaf = write(&root, "src/lib/leaf.ts", "export const leaf = 1;\n");
    let cycle_b = write(
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
        discovered,
        vec![
            canonicalize_non_verbatim(&lib),
            canonicalize_non_verbatim(&root_shared),
            canonicalize_non_verbatim(&panel),
            canonicalize_non_verbatim(&widget),
            canonicalize_non_verbatim(&cycle_a),
            canonicalize_non_verbatim(&js_alias),
            canonicalize_non_verbatim(&cycle_b),
            canonicalize_non_verbatim(&leaf),
        ]
    );

    let _ = std::fs::remove_dir_all(&root);
}
