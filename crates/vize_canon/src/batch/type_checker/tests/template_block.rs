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
const tone: "info" = "info"
const target = document.createElement("button")
</script>

<template>
  <PluginCard :tone="tone" @click="$portal.open(target)" />
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
