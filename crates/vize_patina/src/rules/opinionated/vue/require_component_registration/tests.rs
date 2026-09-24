use super::{RequireComponentRegistration, pascal_to_kebab};
use crate::{LintPreset, Linter};

fn create_linter() -> Linter {
    Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec!["vue/require-component-registration".into()]))
}

#[test]
fn test_pascal_to_kebab() {
    assert_eq!(pascal_to_kebab("MyButton"), "my-button");
    assert_eq!(pascal_to_kebab("NuxtLink"), "nuxt-link");
    assert_eq!(pascal_to_kebab("RouterView"), "router-view");
}

#[test]
fn test_is_custom_component() {
    let rule = RequireComponentRegistration::default();
    assert!(rule.is_custom_component("MyButton"));
    assert!(rule.is_custom_component("my-button"));
    assert!(!rule.is_custom_component("div"));
    assert!(!rule.is_custom_component("span"));
}

#[test]
fn test_is_builtin() {
    let rule = RequireComponentRegistration::default();
    assert!(rule.is_builtin("component"));
    assert!(rule.is_builtin("Transition"));
    assert!(rule.is_builtin("keep-alive"));
    assert!(!rule.is_builtin("MyButton"));
}

#[test]
fn test_allows_script_setup_component_imports() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import Child from './Child.vue'
import { NamedWidget } from './widgets'
import { LibraryWidget as RenamedWidget } from '@example/widgets'
</script>

<template>
  <Child />
  <NamedWidget />
  <renamed-widget />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert!(
        result.diagnostics.is_empty(),
        "script setup imports should be recognized as registered components: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_allows_async_component_defined_in_script_setup() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import { defineAsyncComponent, ref } from 'vue'

const MonacoEditor = defineAsyncComponent(() => import('../MonacoEditor.vue'))
const shown = ref(true)
</script>

<template>
  <MonacoEditor v-if="shown" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ObjectControl.vue");

    assert!(
        result.diagnostics.is_empty(),
        "a Vue async component bound at setup scope is template-visible: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_allows_aliased_and_namespace_async_component_factories() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import { defineAsyncComponent as asyncComponent } from 'vue'
import * as Vue from 'vue'

const LoadingPanel = asyncComponent(() => import('./LoadingPanel.vue'))
const StatusBadge = Vue.defineAsyncComponent(() => import('./StatusBadge.vue'))
</script>

<template>
  <loading-panel />
  <StatusBadge />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert!(
        result.diagnostics.is_empty(),
        "aliased and namespaced Vue factories should register their setup bindings: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_reports_non_component_setup_bindings_and_missing_components() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import { defineAsyncComponent } from 'vue'

const AsyncPanel = defineAsyncComponent(() => import('./AsyncPanel.vue'))
const OrdinaryValue = 1
const LocalFactory = (() => 'not a Vue component')()
</script>

<template>
  <AsyncPanel />
  <OrdinaryValue />
  <LocalFactory />
  <MissingWidget />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert_eq!(result.warning_count, 3, "{:?}", result.diagnostics);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name == "vue/require-component-registration")
    );
}

#[test]
fn test_does_not_treat_unrelated_factory_as_vue_async_component() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
function defineAsyncComponent(loader: () => unknown) { return loader }
const LocalPanel = defineAsyncComponent(() => import('./LocalPanel.vue'))
</script>

<template>
  <LocalPanel />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn test_does_not_treat_non_vue_or_type_only_import_as_async_factory() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import { defineAsyncComponent as unrelatedFactory } from './factory'
import type { defineAsyncComponent } from 'vue'

const LocalPanel = unrelatedFactory(() => import('./LocalPanel.vue'))
const TypeOnlyPanel = defineAsyncComponent(() => import('./TypeOnlyPanel.vue'))
</script>

<template>
  <LocalPanel />
  <TypeOnlyPanel />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert_eq!(result.warning_count, 2, "{:?}", result.diagnostics);
}

#[test]
fn test_allows_options_api_components_registration() {
    let linter = create_linter();
    let sfc = r#"<script lang="ts">
import Child from './Child.vue'
import LocalPanel from './LocalPanel.vue'

export default {
  components: {
    Child,
    RegisteredPanel: LocalPanel,
  },
}
</script>

<template>
  <Child />
  <registered-panel />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert!(
        result.diagnostics.is_empty(),
        "Options API `components` registrations should count as registered: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_allows_plain_script_component_imports_when_script_setup_is_present() {
    let linter = create_linter();
    let sfc = r#"<script lang="ts">
import LoadingBar from './LoadingBar.vue'

export const helper = () => 1
</script>

<script setup lang="ts">
const shown = true
</script>

<template>
  <div v-if="shown">
    <LoadingBar />
  </div>
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert!(
        result.diagnostics.is_empty(),
        "plain script imports are template-visible in split-script setup SFCs: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_reports_normal_script_import_without_components_registration() {
    let linter = create_linter();
    let sfc = r#"<script lang="ts">
import Child from './Child.vue'

export default {}
</script>

<template>
  <Child />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "vue/require-component-registration"
    );
}

#[test]
fn test_reports_unimported_script_setup_component() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child />
  <MissingWidget />
</template>
"#;
    let result = linter.lint_sfc(sfc, "ParentWidget.vue");

    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "vue/require-component-registration"
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains("Component is used but not explicitly imported")
    );
}

#[test]
fn test_allows_recursive_self_reference_by_filename() {
    // #4953: a `<script setup>` component may reference itself by its
    // filename-derived name for recursion; no import is needed (or possible).
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
defineProps<{ node: { children: unknown[] } }>();
</script>

<template>
  <ul>
    <li v-for="(child, i) in node.children" :key="i">
      <TreeItem :node="child" />
    </li>
  </ul>
</template>
"#;
    let result = linter.lint_sfc(sfc, "src/components/tree-item.vue");

    assert!(
        result.diagnostics.is_empty(),
        "the component's own filename-derived name should count as registered: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_allows_recursive_self_reference_by_define_options_name() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
defineOptions({ name: "TreeItem" });
defineProps<{ node: { children: unknown[] } }>();
</script>

<template>
  <ul>
    <li v-for="(child, i) in node.children" :key="i">
      <TreeItem :node="child" />
    </li>
  </ul>
</template>
"#;
    let result = linter.lint_sfc(sfc, "item.vue");

    assert!(
        result.diagnostics.is_empty(),
        "the `defineOptions` name should count as registered: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_reports_component_not_matching_self_name() {
    let linter = create_linter();
    let sfc = r#"<script setup lang="ts">
defineOptions({ name: "TreeItem" });
</script>

<template>
  <OtherWidget />
</template>
"#;
    let result = linter.lint_sfc(sfc, "tree-item.vue");

    assert_eq!(result.warning_count, 1);
}
