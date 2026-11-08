//! vue/slot-name-casing
//!
//! Enforce camelCase for names on rendered `<slot>` outlets.
//!
//! This follows the default behavior of `vue/slot-name-casing`: it checks the
//! static `name` attribute of `<slot>`, not the `v-slot`/`#` syntax used by a
//! parent to provide slot content.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <slot name="my-slot" />
//! <slot name="MySlot" />
//! ```
//!
//! ### Valid
//! ```vue
//! <slot name="mySlot" />
//! <slot name="default" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/slot-name-casing",
    description: "Enforce camelCase for rendered slot names",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce camelCase for rendered slot names.
pub struct SlotNameCasing;

impl SlotNameCasing {
    fn is_camel_case_name(name: &str) -> bool {
        let mut chars = name.chars();
        matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
            && chars.all(|character| character.is_ascii_alphanumeric())
    }
}

impl Rule for SlotNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type != ElementType::Slot {
            return;
        }
        let Some(attribute) = element.props.iter().find_map(|prop| match prop {
            PropNode::Attribute(attribute) if attribute.name == "name" => Some(attribute),
            _ => None,
        }) else {
            return;
        };
        let Some(value) = &attribute.value else {
            return;
        };
        let name = value.content;
        if name.is_empty() || Self::is_camel_case_name(name) {
            return;
        }

        ctx.warn_with_help(
            ctx.t_fmt("vue/slot-name-casing.message", &[("name", name)]),
            &attribute.loc,
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
    fn provider_slot_names_are_outside_this_rule() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #mySlot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn provider_snake_case_is_outside_this_rule() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #my_slot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn provider_longform_is_outside_this_rule() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template v-slot:mySlot>Body</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_non_camel_case_slot_outlet_names_at_the_attribute() {
        let source = r#"<div><slot name="my-slot" /><slot name="MySlot" /></div>"#;
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 2);
        let first = source.find("name=\"my-slot\"").unwrap() as u32;
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (first, first + "name=\"my-slot\"".len() as u32)
        );
    }

    #[test]
    fn accepts_camel_case_slot_outlet_names() {
        let result = create_linter().lint_template(
            r#"<div><slot name="mySlot" /><slot name="default" /></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_dynamic_slot_outlet_names() {
        let result = create_linter().lint_template(r#"<slot :name="dynamicName" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
