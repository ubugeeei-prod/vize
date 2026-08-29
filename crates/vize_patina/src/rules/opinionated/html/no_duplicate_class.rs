//! html/no-duplicate-class
//!
//! Disallow duplicate class names inside a static `class` attribute.
//!
//! Only the static `class="..."` attribute is inspected. Dynamic `:class`
//! bindings are ignored because their effective value is not known statically.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div class="btn btn primary">click</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div class="btn primary">click</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;
use vize_s0::FxHashSet;

static META: RuleMeta = RuleMeta {
    name: "html/no-duplicate-class",
    description: "Disallow duplicate class names in a static class attribute",
    category: RuleCategory::HtmlConformance,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoDuplicateClass;

impl NoDuplicateClass {
    fn check_class_value(ctx: &mut LintContext<'_>, class_value: &str, range: ByteRange) {
        let mut seen: FxHashSet<&str> = FxHashSet::default();
        let mut reported: FxHashSet<&str> = FxHashSet::default();

        for class_name in class_value.split_ascii_whitespace() {
            if !seen.insert(class_name) && reported.insert(class_name) {
                let message = ctx.t_fmt("html/no-duplicate-class.message", &[("name", class_name)]);
                let help = ctx.t("html/no-duplicate-class.help");
                ctx.warn_at_with_help(message, range, help);
            }
        }
    }

    fn check_binding(ctx: &mut LintContext<'_>, binding: &MarkupBinding<'_>) {
        if binding.kind() == MarkupBindingKind::Attribute
            && binding.is_unqualified_arg_exact("class")
            && let Some(class_value) = binding.static_value()
        {
            Self::check_class_value(ctx, class_value, binding.range());
        }
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        element.walk_bindings(&mut |binding| {
            Self::check_binding(ctx, &binding);
        });
    }
}

impl MarkupRule for NoDuplicateClass {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        Self::check_binding(ctx.lint(), binding);
    }
}

impl Rule for NoDuplicateClass {
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
    use super::NoDuplicateClass;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDuplicateClass));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_classes() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="btn primary">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="btn">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_class_ignored() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="['btn', 'btn']">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_class() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="btn btn primary">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_two_distinct_duplicates() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="a a b b">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_invalid_triple_duplicate_reports_once() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="x x x">y</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_extra_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="  btn   primary  ">x</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
