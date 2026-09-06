use super::Linter;

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

/// `Child` really is unused here — the `computed` parameter shadows the import —
/// but a dynamic `:is` suppresses the whole file, matching
/// `eslint-plugin-vue`'s `ignoreWhenBindingPresent: true` default (#3223).
///
/// Vize used to report it, and the extra strictness cost far more than it
/// found: over the pinned corpus it produced 2,235 false positives, because a
/// dynamic `<component :is>` normally *does* reach its registered components —
/// through a string map, or a registration under a computed key that leaves no
/// statically matchable name. Restoring this detection needs the upstream escape
/// hatch (`ignoreWhenBindingPresent: false`), not a different default.
#[test]
fn test_lint_sfc_no_unused_components_ignores_shadowed_import_under_dynamic_is() {
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

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

/// The suppression is scoped to *dynamic* bindings. A literal `:is` names its
/// component, so the registration stays checkable.
#[test]
fn test_lint_sfc_no_unused_components_still_reports_under_literal_is() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import Child from './DynamicChild.vue'
</script>

<template>
  <component :is="'SomethingElse'" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].rule_name, "vue/no-unused-components");
    assert!(result.diagnostics[0].message.contains("Child"));
}

/// Parentheses do not make a literal dynamic: the expression is parsed, not
/// matched on its delimiters.
#[test]
fn test_lint_sfc_no_unused_components_still_reports_under_parenthesized_literal_is() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import Child from './DynamicChild.vue'
</script>

<template>
  <component :is="('SomethingElse')" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert!(result.diagnostics[0].message.contains("Child"));
}

/// An interpolated template literal only names its component at runtime, so it
/// is a dynamic binding and suppresses like any other.
#[test]
fn test_lint_sfc_no_unused_components_ignores_interpolated_template_literal_is() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import Child from './DynamicChild.vue'

const name = 'SomethingElse'
</script>

<template>
  <component :is="`${name}`" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

/// A substitution-free template literal still names its component.
#[test]
fn test_lint_sfc_no_unused_components_still_reports_under_plain_template_literal_is() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import Child from './DynamicChild.vue'
</script>

<template>
  <component :is="`SomethingElse`" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert!(result.diagnostics[0].message.contains("Child"));
}

/// A static `is="..."` attribute is not a binding and suppresses nothing.
#[test]
fn test_lint_sfc_no_unused_components_still_reports_under_static_is() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import Child from './DynamicChild.vue'
</script>

<template>
  <component is="SomethingElse" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    assert!(result.diagnostics[0].message.contains("Child"));
}

/// Pinned reproduction from `tests/_fixtures/_git/element`
/// (`packages/result/src/index.vue`, revision `1ede...`): four icons registered
/// under computed keys and rendered through `<component :is="iconElement">`. All
/// four were reported unused.
#[test]
fn test_lint_sfc_no_unused_components_pinned_computed_key_registration_stays_clean() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<template>
  <div class="el-result">
    <div class="el-result__icon">
      <component :is="iconElement" :class="iconElement" />
    </div>
  </div>
</template>

<script>
import IconSuccess from './icon-success.vue';
import IconError from './icon-error.vue';

const IconMap = { success: 'icon-success', error: 'icon-error' };

export default {
  name: 'ElResult',
  components: {
    [IconSuccess.name]: IconSuccess,
    [IconError.name]: IconError
  },
  props: { icon: { type: String, default: 'info' } },
  computed: {
    iconElement() {
      return IconMap[this.icon] || 'icon-info';
    }
  }
};
</script>
"#;
    let result = linter.lint_sfc(sfc, "index.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

/// A dynamic directive argument can resolve to `is`, so it suppresses too.
#[test]
fn test_lint_sfc_no_unused_components_ignores_dynamic_directive_argument() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["vue/no-unused-components".into()]));
    let sfc = r#"<script setup lang="ts">
import Child from './DynamicChild.vue'

const key = 'is'
const value = 'Other'
</script>

<template>
  <component :[key]="value" />
</template>
"#;
    let result = linter.lint_sfc(sfc, "test.vue");

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
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
    assert!(result.diagnostics[0].message.contains("FourStyle"));
}
