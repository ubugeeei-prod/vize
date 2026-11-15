//! #13021: old vue-tsc library declarations lack `__VLS_PROPS_FALLBACK`.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use std::path::Path;
use vize_carton::String;

const LIBS: &str = r#"import type { PublicProps, VNode } from "vue";

type ItemSizeValue<T> = number | ((item: T) => number);
type Props<T> = { items: T[]; itemSize?: ItemSizeValue<T> } & PublicProps;
type Fallback = typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P } ? P : {};

export declare const NoFallback: <T>(
  props: Props<T>,
  ctx?: { attrs: any; slots: {}; emit: () => void },
  expose?: unknown,
  setup?: Promise<{ props: Props<T> }>,
) => VNode & { __ctx?: { props: Props<T>; attrs: any } };

export declare const WithFallback: <T>(
  props: Props<T> & Fallback,
  ctx?: { attrs: any; slots: {}; emit: () => void },
  expose?: unknown,
  setup?: Promise<{ props: Props<T> & Fallback }>,
) => VNode & { __ctx?: { props: Props<T> & Fallback; attrs: any } };

export declare const NoCtx: <T>(props: Props<T>) => VNode;
"#;

const APP: &str = r#"<script setup lang="ts">
import { NoCtx, NoFallback, WithFallback } from "./libs";
</script>

<template>
  <NoFallback :items="[1, 2]" :buffer="200" />
  <WithFallback :items="[1, 2]" :buffer="200" />
  <NoCtx :items="[1, 2]" :buffer="200" />

  <NoFallback :items="[1, 2]" :data-index="0" />
  <WithFallback :items="[1, 2]" :data-index="0" />
  <NoCtx :items="[1, 2]" :data-index="0" />
</template>
"#;

const WRONG: &str = r#"<script setup lang="ts">
import { NoFallback } from "./libs";
</script>

<template>
  <NoFallback :items="1" />
</template>
"#;

const WRONG_INFERRED_PROP: &str = r#"<script setup lang="ts">
import { NoFallback } from "./libs";

const items = [{ id: "one", name: "Alpha", downloads: 1 }];
</script>

<template>
  <NoFallback :items="items" :item-size="'forty-four'" :buffer="200" />
</template>
"#;

const MISSING_REQUIRED: &str = r#"<script setup lang="ts">
import { NoFallback } from "./libs";
</script>

<template>
  <NoFallback :buffer="200" />
</template>
"#;

const REAL_LIST: &str = r#"<script setup lang="ts">
import { DynamicScroller, DynamicScrollerItem } from "vue-virtual-scroller";

const items = [{ id: 1 }, { id: 2 }];
</script>

<template>
  <DynamicScroller :items="items" :minItemSize="32" :buffer="200">
    <template #default="{ item, index, active }">
      <DynamicScrollerItem :item="item" :active="active" :data-index="index" />
    </template>
  </DynamicScroller>
</template>
"#;

type Diagnostics = Vec<(String, Option<u32>, String)>;

fn check(name: &str, vue_options: &str) -> Option<Diagnostics> {
    let root = create_project_case(
        name,
        &[
            ("src/libs.ts", LIBS),
            ("src/App.vue", APP),
            ("src/Wrong.vue", WRONG),
            ("src/WrongInferredProp.vue", WRONG_INFERRED_PROP),
            ("src/MissingRequired.vue", MISSING_REQUIRED),
        ],
    );
    std::fs::write(
        root.join("tsconfig.json"),
        vize_carton::cstr!(
            r#"{{
  "compilerOptions": {{
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "types": []
  }},{vue_options}
  "include": ["src/**/*.vue", "src/**/*.ts"]
}}"#
        ),
    )
    .unwrap();
    let diagnostics = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(root);
    diagnostics
}

fn expected_wrong_value() -> Diagnostics {
    vec![
        (
            "src/Wrong.vue".into(),
            Some(2322),
            "6:16:error Type 'number' is not assignable to type 'unknown[]'.".into(),
        ),
        (
            "src/WrongInferredProp.vue".into(),
            Some(2322),
            "8:31:error Type 'string' is not assignable to type 'ItemSizeValue<{ id: string; name: string; downloads: number; }> | undefined'.".into(),
        ),
    ]
}

fn assert_loose_packet(actual: Option<Diagnostics>) {
    let actual = actual.expect("the native checker should be available");
    // Keep the whole packet exact. In particular, the old generic declaration
    // shape does not report its missing `items` prop; changing that unrelated
    // behavior is outside this compatibility fix.
    assert_eq!(
        actual,
        expected_wrong_value(),
        "the loose diagnostic packet must retain every row and no others"
    );
}

fn check_strict_packet() -> Option<Diagnostics> {
    let root = create_project_case(
        "library-generic-fallback-strict",
        &[("src/libs.ts", LIBS), ("src/App.vue", APP)],
    );
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "types": []
  },
  "vueCompilerOptions": { "checkUnknownProps": true },
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#,
    )
    .unwrap();
    let diagnostics = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(root);
    diagnostics
}

#[test]
fn old_library_generic_props_follow_the_loose_unknown_props_policy() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    assert_loose_packet(check("library-generic-fallback-default", ""));
    assert_loose_packet(check(
        "library-generic-fallback-explicit-loose",
        r#"
  "vueCompilerOptions": { "checkUnknownProps": false },"#,
    ));
}

#[test]
fn strict_mode_retains_the_complete_excess_property_packet() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    assert_eq!(
        check_strict_packet().expect("the native checker should be available"),
        vec![
            (
                "src/App.vue".into(),
                Some(2353),
                r#"10:32:error Object literal may only specify known properties, and '"dataIndex"' does not exist in type 'Props<number>'."#.into(),
            ),
            (
                "src/App.vue".into(),
                Some(2353),
                r#"11:34:error Object literal may only specify known properties, and '"dataIndex"' does not exist in type '{ items: number[]; itemSize?: ItemSizeValue<number> | undefined; } & VNodeProps & AllowedComponentProps & ComponentCustomProps'."#.into(),
            ),
            (
                "src/App.vue".into(),
                Some(2353),
                r#"6:32:error Object literal may only specify known properties, and '"buffer"' does not exist in type 'Props<number>'."#.into(),
            ),
            (
                "src/App.vue".into(),
                Some(2353),
                r#"7:34:error Object literal may only specify known properties, and '"buffer"' does not exist in type '{ items: number[]; itemSize?: ItemSizeValue<number> | undefined; } & VNodeProps & AllowedComponentProps & ComponentCustomProps'."#.into(),
            ),
        ],
        "strict mode must retain the complete authored TS2353 packet and exact spans",
    );
}

#[test]
fn vue_virtual_scroller_305_accepts_documented_fallthrough_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let test_modules = workspace_root.join("tests/node_modules");
    let package = test_modules.join("vue-virtual-scroller");
    assert!(
        package.join("package.json").is_file(),
        "install the pinned real-project dependencies before this test",
    );
    let root = super::real_vue::create_project_case(
        "library-generic-fallback-virtual-scroller",
        &[("src/List.vue", REAL_LIST)],
    );
    super::super::symlink::symlink_path(&package, &root.join("node_modules/vue-virtual-scroller"))
        .unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "types": []
  },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    let diagnostics = snapshot_project_diagnostics(&root)
        .expect("the native checker should return a complete diagnostic packet");
    let _ = std::fs::remove_dir_all(root);
    assert_eq!(
        diagnostics,
        Vec::new(),
        "the pinned vue-virtual-scroller consumer must match vue-tsc: {diagnostics:#?}",
    );
}
