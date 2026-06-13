//! vue/slot-name-casing
//!
//! Enforce kebab-case for named slots used via `v-slot`.
//!
//! Slot names passed to `v-slot` (or the `#` shorthand) should be written in
//! kebab-case, mirroring how slots are commonly named in Vue templates. A slot
//! name is flagged when it contains an uppercase letter or an underscore.
//!
//! The default slot and dynamic slot names (e.g. `#[dynamicName]`) are ignored.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent>
//!   <template #mySlot>...</template>
//!   <template #my_slot>...</template>
//! </MyComponent>
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent>
//!   <template #my-slot>...</template>
//!   <template #default>...</template>
//! </MyComponent>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/slot-name-casing",
    description: "Enforce kebab-case for named slots used via v-slot",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce kebab-case for named slots.
pub struct SlotNameCasing;

impl SlotNameCasing {
    /// A slot name is kebab-case friendly when it has no uppercase letters and
    /// no underscores. Single lowercase words such as `header` are allowed.
    fn is_kebab_case_name(name: &str) -> bool {
        !name.chars().any(|c| c.is_uppercase() || c == '_')
    }
}

impl Rule for SlotNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "slot" {
            return;
        }

        // No argument means the default slot, which has no user-chosen name.
        let Some(arg) = &directive.arg else {
            return;
        };

        // Only static slot names can be checked; dynamic names like
        // `#[dynamicName]` are computed at runtime and are skipped.
        let ExpressionNode::Simple(arg) = arg else {
            return;
        };
        if !arg.is_static {
            return;
        }

        let name = arg.content.as_str();
        if name.is_empty() || Self::is_kebab_case_name(name) {
            return;
        }

        ctx.warn_with_help(
            ctx.t_fmt("vue/slot-name-casing.message", &[("name", name)]),
            &directive.loc,
            ctx.t("vue/slot-name-casing.help"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::SlotNameCasing;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(SlotNameCasing));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_kebab_case_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #my-slot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_word_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #header>Header</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_default_slot_is_ignored() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template v-slot>Default</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_slot_name_is_ignored() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #[dynamicName]>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_camel_case_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #mySlot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_snake_case_shorthand() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #my_slot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_camel_case_longform() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template v-slot:mySlot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}
