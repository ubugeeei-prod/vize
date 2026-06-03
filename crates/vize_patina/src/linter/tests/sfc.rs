use super::{LintPreset, Linter};
use crate::telegraph::LspEmitter;

#[test]
fn test_lint_sfc_opinionated_reports_no_next_tick() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let sfc = r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-next-tick")
    );
}

#[test]
fn test_lint_sfc_opinionated_reports_no_get_current_instance() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let sfc = r#"<script setup lang="ts">
import { getCurrentInstance } from 'vue'

const instance = getCurrentInstance()
</script>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-get-current-instance")
    );
}

#[test]
fn test_lint_sfc_byte_offset() {
    let linter = Linter::new();
    let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(result.error_count > 0, "Should detect v-for without key");

    if let Some(diag) = result.diagnostics.first() {
        assert!(
            diag.start > 50,
            "Byte offset should be adjusted for template position"
        );
    }
}

#[test]
fn test_lint_sfc_uses_script_analysis_for_prop_mutation() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>

<template>
  <input v-model="count" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-mutating-props");
    assert!(
        result.diagnostics[0].start > 70,
        "diagnostic should be reported in template coordinates"
    );
}

#[test]
fn test_lint_sfc_reports_nested_prop_v_model_mutation() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ user: { name: string }, count: number }>()
</script>

<template>
  <input v-model="props.user.name" />
  <input v-model="props['count']" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.error_count, 2);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
    );
}

#[test]
fn test_lint_sfc_reports_dynamic_props_object_v_model_mutation() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ user: { name: string } }>()
const key = 'user' as keyof typeof props
</script>

<template>
  <input v-model="props[key].name" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-mutating-props");
}

#[test]
fn test_lint_sfc_reports_unlisted_props_object_v_model_mutation() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
const props = defineProps<Record<string, string>>()
const field = 'title'
</script>

<template>
  <input v-model="props.title" />
  <input v-model="props[field]" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.error_count, 2);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
    );
}

#[test]
fn test_lint_sfc_reports_destructured_prop_member_v_model_mutation() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
const { user } = defineProps<{ user: { name: string } }>()
</script>

<template>
  <input v-model="user.name" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.error_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-mutating-props");
}

#[test]
fn test_lint_sfc_allows_local_v_model_names_that_start_like_props() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
import { reactive } from 'vue'

defineProps<{ count: number }>()
const propsState = reactive({ count: 0 })
const counter = reactive({ value: 0 })
</script>

<template>
  <input v-model="propsState.count" />
  <input v-model="counter.value" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "local state should not be treated as prop mutation: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_sfc_allows_local_props_object_v_model_member() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]));
    let sfc = r#"<script setup lang="ts">
import { reactive } from 'vue'

defineProps<{ user: { name: string } }>()
const props = reactive({ user: { name: '' } })
</script>

<template>
  <input v-model="props.user.name" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "local props object should not be treated as defineProps output: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_sfc_no_unused_components_reports_unused_vue_import() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup>
import MyButton from './MyButton.vue'
</script>

<template>
  <div>Hello</div>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-unused-components");
    assert!(result.diagnostics[0].message.contains("MyButton"));
}

#[test]
fn test_lint_sfc_no_unused_components_allows_local_pascal_case_constants() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
const GapList = [4, 3, 2, 1]
const gap = GapList[0]
</script>

<template>
  <div :data-gap="gap" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "local PascalCase constants are not component registrations: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_sfc_no_unused_components_matches_kebab_case_vue_import() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup>
import MyButton from './MyButton.vue'
</script>

<template>
  <my-button />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "kebab-case component usage should mark the import as used: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_sfc_no_unused_components_allows_dynamic_component_import_reference() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import { computed } from 'vue'
import Child from './DynamicChild.vue'

const current = computed(() => Child)
</script>

<template>
  <component :is="current" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "component imports referenced from script setup should be treated as used: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_sfc_no_unused_components_reports_shadowed_dynamic_component_import() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import { computed } from 'vue'
import Child from './DynamicChild.vue'

const current = computed((Child) => Child)
</script>

<template>
  <component :is="current" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-unused-components");
    assert!(result.diagnostics[0].message.contains("Child"));
}

#[test]
fn test_lint_sfc_no_unused_components_matches_options_api_component_alias() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script lang="ts">
import Style from './style.vue'
import { defineComponent } from 'vue'

export default defineComponent({
  components: {
    FourStyle: Style,
  },
})
</script>

<template>
  <four-style />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "Options API component aliases should be matched by registered name: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lint_sfc_no_unused_components_reports_unused_options_api_component_alias() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script lang="ts">
import Style from './style.vue'
import { defineComponent } from 'vue'

export default defineComponent({
  components: {
    FourStyle: Style,
  },
})
</script>

<template>
  <div />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-unused-components");
    assert!(result.diagnostics[0].message.contains("Style"));
}

#[test]
fn test_lint_sfc_offset_line_conversion() {
    let linter = Linter::new();
    let sfc = r#"<script setup lang="ts">
const foo = 'bar';
</script>

<template>
  <ul><li v-for="item in items">{{ item }}</li></ul>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert!(result.error_count > 0);

    let lsp_diags = LspEmitter::to_lsp_diagnostics_with_source(&result, sfc);
    if let Some(lsp) = lsp_diags.first() {
        assert_eq!(
            lsp.range.start.line, 5,
            "First diagnostic should be on line 5 (0-indexed)"
        );
    }
}

#[test]
fn test_lint_sfc_with_nested_templates() {
    let linter = Linter::new();
    let sfc = r#"<script setup lang="ts">
const show = true;
</script>

<template>
  <div>
    <template v-if="show">
      <span>Visible</span>
    </template>
    <template v-else>
      <span>Hidden</span>
    </template>
  </div>
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(
        result.error_count, 0,
        "Should not report errors for valid nested templates with directives"
    );
}

#[test]
fn test_lint_sfc_with_nested_template_extraction() {
    let linter = Linter::new();
    let sfc = r#"<script></script>
<template>
  <div>
    <template v-if="x">nested</template>
  </div>
</template>"#;

    let result = linter.lint_sfc(sfc, "test.vue");
    assert_eq!(
        result.error_count, 0,
        "Should properly extract and lint nested templates"
    );
}
