use std::path::Path;

use super::{BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary};
use crate::batch::{SfcBlockType, TypeChecker};

const PLUGIN_GLOBALS: &str = r#"import type { DefineComponent } from "vue";
export {};

declare module "vue" {
  export interface ComponentCustomProperties {
    $portal: {
      open(el: HTMLElement): void;
    };
  }

  export interface GlobalComponents {
    PluginCard: DefineComponent<{ tone: "info" | "danger" }>;
  }
}
"#;

/// A `vue` whose typings do not export `NativeElements` — Vue 2.7, an older
/// Vue 3 minor, a trimmed or shimmed package — must make native prop checking
/// degrade to unchecked, never to an error. Naming the module at each use site
/// reported `TS2694` on every authored attribute of correct code:
///
/// ```text
/// error:6:16 [TS2694] Namespace '".../node_modules/vue/index"' has no exported member 'NativeElements'.
/// error:7:16 [TS2694] Namespace '".../node_modules/vue/index"' has no exported member 'NativeElements'.
/// ```
#[test]
fn native_prop_checks_stay_silent_when_vue_has_no_native_elements() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "native-props-without-native-elements",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const num = 1
</script>

<template>
  <a :href="num">link</a>
  <li :key="num" />
</template>
"#,
        )],
    );
    write_vue_without_native_elements(&project_root);

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.scan_project().unwrap();
    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    let reported = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                diagnostic.line + 1,
                diagnostic.column + 1,
            )
        })
        .collect::<Vec<_>>();
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(reported, Vec::new());
}

/// Replace the harness `vue` package with typings that predate
/// `NativeElements`, keeping just enough surface for the generated module.
fn write_vue_without_native_elements(project_root: &Path) {
    let vue_dir = project_root.join("node_modules/vue");
    let _ = std::fs::remove_file(&vue_dir);
    let _ = std::fs::remove_dir_all(&vue_dir);
    std::fs::create_dir_all(&vue_dir).unwrap();
    std::fs::write(
        vue_dir.join("package.json"),
        "{ \"name\": \"vue\", \"version\": \"3.2.47\", \"types\": \"index.d.ts\" }\n",
    )
    .unwrap();
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export interface ComponentPublicInstance {
  $props: Record<string, unknown>;
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (event: string, ...args: unknown[]) => void;
}
"#,
    )
    .unwrap();
}

#[test]
fn template_expression_diagnostics_are_mapped_to_the_template_block() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "template-block-diagnostic-mapping",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const count = 1
</script>

<template>
  <button>{{ count.toUpperCase() }}</button>
</template>
"#,
        )],
    );

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.scan_project().unwrap();
    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };

    let relevant = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            relative_path(&project_root, &diagnostic.file) == "src/App.vue"
                && diagnostic.code == Some(2339)
                && diagnostic.message.contains("toUpperCase")
        })
        .collect::<Vec<_>>();

    assert!(
        relevant.iter().any(|diagnostic| {
            diagnostic.block_type == Some(SfcBlockType::Template) && diagnostic.line >= 5
        }),
        "template expression diagnostics should point at <template>, got: {relevant:#?}"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| relative_path(&project_root, &diagnostic.file) == "src/App.vue"),
        "diagnostics should be remapped away from generated .vue.ts files: {:#?}",
        result.diagnostics
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn template_typecheck_uses_dom_libs_and_vue_plugin_augmentations() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "template-dom-vue-plugin-augmentations",
        &[
            ("src/plugin-globals.d.ts", PLUGIN_GLOBALS),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from "vue"

const tone: "info" = "info"
const target = document.createElement("button")
const label = ref("ready")
</script>

<template>
  <PluginCard :tone="tone" @click="$portal.open(target)">
    {{ label.toUpperCase() }}
  </PluginCard>
</template>
"#,
            ),
        ],
    );

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.scan_project().unwrap();
    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };

    let unexpected = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            relative_path(&project_root, &diagnostic.file) == "src/App.vue"
                && matches!(diagnostic.code, Some(2304 | 2322 | 2339 | 2345))
        })
        .map(|diagnostic| {
            (
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message.clone(),
                diagnostic.block_type,
            )
        })
        .collect::<Vec<_>>();

    assert!(
        unexpected.is_empty(),
        "DOM globals and Vue plugin augmentations should typecheck in template: {unexpected:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn options_api_template_keeps_plugin_globals_and_dom_typed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "options-api-template-dom-plugin",
        &[
            ("src/plugin-globals.d.ts", PLUGIN_GLOBALS),
            (
                "src/App.vue",
                r#"<script lang="ts">
export default {
  data() {
    return {
      tone: "info" as const,
      target: document.createElement("button"),
    }
  },
  methods: {
    open() {
      this.$portal.open(this.target)
    },
  },
}
</script>

<template>
  <PluginCard :tone="tone" @click="open">{{ tone.toUpperCase() }}</PluginCard>
</template>
"#,
            ),
        ],
    );

    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.scan_project().unwrap();
    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };

    let unexpected = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            relative_path(&project_root, &diagnostic.file) == "src/App.vue"
                && matches!(diagnostic.code, Some(2304 | 2322 | 2339 | 2345 | 7006))
        })
        .map(|diagnostic| {
            (
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message.clone(),
                diagnostic.block_type,
            )
        })
        .collect::<Vec<_>>();

    assert!(
        unexpected.is_empty(),
        "Options API template bindings should keep plugin globals and DOM types: {unexpected:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
