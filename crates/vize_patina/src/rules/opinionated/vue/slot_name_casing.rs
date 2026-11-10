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
use crate::markup::{MarkupContext, MarkupElement, MarkupElementKind, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::vue::SlotNameCasingStyle;
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/slot-name-casing",
    description: "Enforce configured casing for rendered slot names",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce configured casing for rendered slot names.
pub struct SlotNameCasing;

struct ConfiguredSlotNameCasing {
    style: SlotNameCasingStyle,
}

impl SlotNameCasing {
    /// Create the rule with the configured casing policy.
    pub fn configured(style: SlotNameCasingStyle) -> impl Rule {
        ConfiguredSlotNameCasing { style }
    }
}

fn is_camel_case_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_alphanumeric())
}

fn is_kebab_case_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && !name.contains("--")
        && !name.chars().any(|character| {
            character.is_ascii_uppercase()
                || character.is_whitespace()
                || matches!(
                    character,
                    '!' | '"'
                        | '#'
                        | '%'
                        | '&'
                        | '\''
                        | '('
                        | ')'
                        | '*'
                        | '+'
                        | ','
                        | '.'
                        | '/'
                        | ':'
                        | ';'
                        | '<'
                        | '='
                        | '>'
                        | '?'
                        | '@'
                        | '['
                        | '\\'
                        | ']'
                        | '^'
                        | '_'
                        | '`'
                        | '{'
                        | '|'
                        | '}'
                )
        })
}

fn accepts(style: SlotNameCasingStyle, name: &str) -> bool {
    match style {
        SlotNameCasingStyle::CamelCase => is_camel_case_name(name),
        SlotNameCasingStyle::KebabCase => is_kebab_case_name(name),
        SlotNameCasingStyle::Singleword => name.bytes().all(|byte| byte.is_ascii_lowercase()),
    }
}

fn lint(style: SlotNameCasingStyle, ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
    if element.kind() != MarkupElementKind::Slot {
        return;
    }
    let Some(attribute) = element.static_attribute("name") else {
        return;
    };
    let Some(name) = attribute.value() else {
        return;
    };
    if name.is_empty() || accepts(style, name) {
        return;
    }

    let casing = style.name();
    ctx.warn_at_with_help(
        ctx.t_fmt(
            "vue/slot-name-casing.message",
            &[("name", name), ("casing", casing)],
        ),
        attribute.range(),
        ctx.t_fmt("vue/slot-name-casing.help", &[("casing", casing)]),
    );
}

impl MarkupRule for SlotNameCasing {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        lint(SlotNameCasingStyle::CamelCase, ctx.lint(), element);
    }
}

impl MarkupRule for ConfiguredSlotNameCasing {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        lint(self.style, ctx.lint(), element);
    }
}

impl Rule for SlotNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        lint(
            SlotNameCasingStyle::CamelCase,
            ctx,
            &MarkupElement::new(element),
        );
    }
}

impl Rule for ConfiguredSlotNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        lint(self.style, ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::{SlotNameCasing, SlotNameCasingStyle};
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

    #[test]
    fn supports_kebab_case_and_singleword_options() {
        for (style, valid, invalid) in [
            (SlotNameCasingStyle::KebabCase, "row-actions", "rowActions"),
            (SlotNameCasingStyle::Singleword, "actions", "row-actions"),
        ] {
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(SlotNameCasing::configured(style)));
            let linter = Linter::with_registry(registry);
            let source = format!(r#"<slot name="{valid}" /><slot name="{invalid}" />"#);
            let result = linter.lint_template(&source, "test.vue");
            assert_eq!(
                result.warning_count, 1,
                "{style:?}: {:?}",
                result.diagnostics
            );
            assert_eq!(
                result.diagnostics[0].message,
                format!("Slot name '{invalid}' is not {}", style.name())
            );
            assert_eq!(
                result.diagnostics[0].help.as_deref(),
                Some(format!("Rename the rendered slot outlet to {}.", style.name()).as_str())
            );
        }
    }

    #[test]
    fn kebab_case_matches_eslint_plugin_vue_boundaries() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(SlotNameCasing::configured(
            SlotNameCasingStyle::KebabCase,
        )));
        let result = Linter::with_registry(registry).lint_template(
            r#"<div>
                <slot name="row-" />
                <slot name="$row" />
                <slot name="row$" />
                <slot name="ro~w" />
                <slot name="a$b-c" />
                <slot name="ñame" />
                <slot name="名前" />
                <slot name="rowActions" />
                <slot name="-row" />
                <slot name="row--actions" />
                <slot name="row_actions" />
            </div>"#,
            "test.vue",
        );

        assert_eq!(result.warning_count, 4, "{:?}", result.diagnostics);
    }
}
