//! vue/valid-v-slot
//!
//! Enforce valid `v-slot` directives.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-slot:header></div>                 <!-- not on component -->
//! <MyComponent v-slot v-slot:header />      <!-- duplicate -->
//! <template v-slot:header v-slot:footer />  <!-- multiple named slots -->
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
//! <MyComponent><template #header>Header</template></MyComponent>
//! <MyComponent><template v-slot:header>Header</template></MyComponent>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-slot",
    description: "Enforce valid `v-slot` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Valid v-slot rule
#[derive(Default)]
pub struct ValidVSlot;

impl ValidVSlot {
    fn is_custom_component(tag: &str) -> bool {
        // Custom components: PascalCase or kebab-case with hyphen
        tag.chars().next().is_some_and(|c| c.is_uppercase()) || tag.contains('-')
    }

    fn count_slot_directives(element: &ElementNode) -> (usize, usize) {
        let mut default_count = 0;
        let mut named_count = 0;

        for prop in &element.props {
            if let PropNode::Directive(dir) = prop {
                if dir.name.as_str() == "slot" {
                    if dir.arg.is_some() {
                        named_count += 1;
                    } else {
                        default_count += 1;
                    }
                }
            }
        }

        (default_count, named_count)
    }
}

impl Rule for ValidVSlot {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "slot" {
            return;
        }

        let tag = element.tag.as_str();

        // v-slot can only be used on components or <template>
        if tag != "template" && !Self::is_custom_component(tag) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
            return;
        }

        // Check for duplicate v-slot directives
        let (default_count, named_count) = Self::count_slot_directives(element);

        if default_count > 1 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // On <template>, can only have one named slot
        if tag == "template" && named_count > 1 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // Mixing default and named on same element
        if default_count > 0 && named_count > 0 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValidVSlot;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidVSlot));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_default_slot() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent v-slot="{ item }">{{ item }}</MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_named_slot_template() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent><template #header>Header</template></MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_on_html_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-slot:header></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_multiple_named_slots() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent>
                <template #header>Header</template>
                <template #footer>Footer</template>
            </MyComponent>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }
}
