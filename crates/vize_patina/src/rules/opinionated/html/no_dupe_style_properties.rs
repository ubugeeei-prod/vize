//! html/no-dupe-style-properties
//!
//! Disallow duplicate CSS properties inside an inline `style` attribute.
//! Cross-framework analogue of svelte's `no-dupe-style-properties` rule.
//!
//! Only the static `style` attribute is inspected. Dynamic `:style` bindings
//! are objects or expressions and are intentionally ignored.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div style="color: red; color: blue">text</div>
//!   <div style="margin: 0; MARGIN: 1px">text</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div style="color: red; background: blue">text</div>
//!   <div :style="{ color: a, color: b }">text</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;
use vize_s0::FxHashSet;
use vize_s0::String;
use vize_s0::ToCompactString;

static META: RuleMeta = RuleMeta {
    name: "html/no-dupe-style-properties",
    description: "Disallow duplicate properties in inline style attributes",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoDupeStyleProperties;

impl NoDupeStyleProperties {
    fn check_style_value(ctx: &mut LintContext<'_>, value: &str, range: ByteRange) {
        let mut seen: FxHashSet<String> = FxHashSet::default();
        for declaration in value.split(';') {
            // A declaration is `property: value`; the property name is the
            // text before the first colon.
            let property = match declaration.split_once(':') {
                Some((name, _)) => name,
                None => continue,
            };
            let normalized = property.trim().to_lowercase();
            if normalized.is_empty() {
                continue;
            }
            let normalized = normalized.to_compact_string();

            if !seen.insert(normalized.clone()) {
                let message = ctx.t_fmt(
                    "html/no-dupe-style-properties.message",
                    &[("property", normalized.as_str())],
                );
                let help = ctx.t("html/no-dupe-style-properties.help");
                ctx.warn_at_with_help(message, range, help);
            }
        }
    }

    fn check_binding(ctx: &mut LintContext<'_>, binding: &MarkupBinding<'_>) {
        if binding.kind() == MarkupBindingKind::Attribute
            && binding.is_unqualified_arg_exact("style")
            && let Some(value) = binding.static_value()
        {
            Self::check_style_value(ctx, value, binding.range());
        }
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_component() {
            return;
        }

        element.walk_bindings(&mut |binding| {
            Self::check_binding(ctx, &binding);
        });
    }
}

impl MarkupRule for NoDupeStyleProperties {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        if element.is_component() {
            return;
        }

        Self::check_binding(ctx.lint(), binding);
    }
}

impl Rule for NoDupeStyleProperties {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::NoDupeStyleProperties;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDupeStyleProperties));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_properties() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div style="color: red; background: blue">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_property() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div style="color: red">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_style() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_style_ignored() {
        let linter = create_linter();
        // Dynamic :style bindings are objects/expressions and must be ignored.
        let result = linter.lint_template(
            r#"<div :style="{ color: a, color: b }">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_property() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div style="color: red; color: blue">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_case_insensitive() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div style="margin: 0; MARGIN: 1px">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_reports_attribute_range_and_normalized_property() {
        let linter = create_linter();
        let source = r#"<div style="margin: 0; MARGIN: 1px">x</div>"#;
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1);

        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.rule_name, "html/no-dupe-style-properties");
        assert_eq!(
            diagnostic.message.as_str(),
            "Duplicate property 'margin' in inline style"
        );
        assert_eq!(
            &source[diagnostic.start as usize..diagnostic.end as usize],
            r#"style="margin: 0; MARGIN: 1px""#,
            "template diagnostics stay on the written style attribute"
        );
    }

    #[test]
    fn test_invalid_duplicate_with_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div style="  color :red ;  color : blue ">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_custom_property_names_keep_existing_case_folding() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div style="--Gap: 0; --gap: 1px">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].message.as_str(),
            "Duplicate property '--gap' in inline style"
        );
    }

    #[test]
    fn test_invalid_triple_duplicate() {
        let linter = create_linter();
        // Two warnings: second and third occurrences of `color`.
        let result = linter.lint_template(
            r#"<div style="color: red; color: blue; color: green">x</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }
}
