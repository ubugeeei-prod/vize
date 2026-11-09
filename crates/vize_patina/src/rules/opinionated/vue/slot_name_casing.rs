//! vue/slot-name-casing
//!
//! Enforce configured casing for names on rendered `<slot>` outlets.
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
    description: "Enforce configured casing for rendered slot names",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Casing accepted for a rendered slot outlet name.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SlotNameCasingStyle {
    /// Lower camel case, such as `rowActions`.
    #[default]
    CamelCase,
    /// Lowercase hyphenated words, such as `row-actions`.
    KebabCase,
    /// One lowercase ASCII word, such as `actions`.
    Singleword,
}

impl SlotNameCasingStyle {
    const fn name(self) -> &'static str {
        match self {
            Self::CamelCase => "camelCase",
            Self::KebabCase => "kebab-case",
            Self::Singleword => "singleword",
        }
    }

    fn accepts(self, name: &str) -> bool {
        match self {
            Self::CamelCase => is_camel_case_name(name),
            Self::KebabCase => is_kebab_case_name(name),
            Self::Singleword => name.bytes().all(|byte| byte.is_ascii_lowercase()),
        }
    }
}

/// Enforce configured casing for rendered slot names.
#[derive(Default)]
pub struct SlotNameCasing {
    style: SlotNameCasingStyle,
}

impl SlotNameCasing {
    /// Create the rule with the configured casing policy.
    pub fn new(style: SlotNameCasingStyle) -> Self {
        Self { style }
    }
}

fn is_camel_case_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_alphanumeric())
}

fn is_kebab_case_name(name: &str) -> bool {
    let mut segments = name.split('-');
    segments.next().is_some_and(is_lowercase_segment) && segments.all(is_lowercase_segment)
}

fn is_lowercase_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
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
        if name.is_empty() || self.style.accepts(name) {
            return;
        }

        let casing = self.style.name();

        ctx.warn_with_help(
            ctx.t_fmt(
                "vue/slot-name-casing.message",
                &[("name", name), ("casing", casing)],
            ),
            &attribute.loc,
            ctx.t_fmt("vue/slot-name-casing.help", &[("casing", casing)]),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{SlotNameCasing, SlotNameCasingStyle};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(SlotNameCasing::default()));
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

    #[test]
    fn supports_kebab_case_and_singleword_options() {
        for (style, valid, invalid) in [
            (SlotNameCasingStyle::KebabCase, "row-actions", "rowActions"),
            (SlotNameCasingStyle::Singleword, "actions", "row-actions"),
        ] {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(SlotNameCasing::new(style)));
            let linter = Linter::with_registry(registry);
            let source = format!(r#"<slot name="{valid}" /><slot name="{invalid}" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(
                result.warning_count, 1,
                "{style:?}: {:?}",
                result.diagnostics
            );
            assert!(result.diagnostics[0].message.contains(style.name()));
            assert!(
                result.diagnostics[0]
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains(style.name()))
            );
        }
    }
}
