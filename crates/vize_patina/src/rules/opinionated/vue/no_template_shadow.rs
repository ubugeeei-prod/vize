//! vue/no-template-shadow
//!
//! Disallow template declarations that shadow template or script-setup bindings.
//!
//! v-for and scoped-slot declarations use authored ranges and lexical template
//! scopes. Shared semantic facts supply script-setup values and declared props.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <!-- Nested v-for with same variable name -->
//! <div v-for="item in items">
//!   <div v-for="item in item.children">{{ item }}</div>
//! </div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div v-for="item in items">
//!   <div v-for="child in item.children">{{ child }}</div>
//! </div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::RootNode;

mod bindings;
mod frozen;
mod scopes;
mod script_bindings;

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-shadow",
    description: "Disallow variable names that shadow variables in outer scope",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow template variable shadowing
pub struct NoTemplateShadow;

impl Rule for NoTemplateShadow {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        scopes::check(ctx, &root.children);
    }
}

#[cfg(test)]
mod tests {
    use super::NoTemplateShadow;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTemplateShadow));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_shadow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_nested_shadow() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_different_names() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div v-for="(item, index) in items" :key="index"><span v-for="(child, childIndex) in item.children" :key="childIndex">{{ child }}</span></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_slot_binding_shadows_options_api_prop() {
        let result = create_linter().lint_sfc(
            r#"<template><Widget #default="{ files }">{{ files }}</Widget></template>
<script>export default { props: { files: Array } }</script>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_slot_binding_shadows_template_scope() {
        let result = create_linter().lint_template(
            r#"<div v-for="files in groups"><Widget #default="{ files }">{{ files }}</Widget></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_invalid_v_for_binding_shadows_nested_component_prop() {
        let result = create_linter().lint_sfc(
            r#"<template><Widget v-for="item in items" :key="item.id" /></template>
<script setup lang="ts">
import { defineComponent } from 'vue'
const Widget = defineComponent({
  props: { item: Object }
})
const items = []
</script>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
        assert_eq!(
            result.diagnostics[0].message,
            "Variable 'item' shadows a variable in an outer scope"
        );
    }

    #[test]
    fn component_registration_name_is_not_a_template_value_binding() {
        let result = create_linter().lint_sfc(
            r#"<template><notification v-for="notification in notifications" /></template>
<script>
import notification from './notification'
export default {
  components: { notification },
  data: () => ({ notifications: [] })
}
</script>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }
}
