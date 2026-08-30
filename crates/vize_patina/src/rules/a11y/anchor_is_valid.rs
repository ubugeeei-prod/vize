//! a11y/anchor-is-valid
//!
//! Enforce that anchor elements have valid href attributes.
//!
//! Anchors with empty, `#`, or `javascript:void(0)` href values are
//! not valid and may cause accessibility issues.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <a href="">Link</a>
//! <a href="#">Link</a>
//! <a href="javascript:void(0)">Link</a>
//! ```
//!
//! ### Valid
//! ```vue
//! <a href="/about">About</a>
//! <a :href="url">Dynamic Link</a>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::url::has_javascript_scheme;
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/anchor-is-valid",
    description: "Enforce valid href on anchor elements",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce valid href on anchor elements
#[derive(Default)]
pub struct AnchorIsValid;

impl AnchorIsValid {
    fn check_static_href(ctx: &mut LintContext<'_>, binding: MarkupBinding<'_>, value: &str) {
        let trimmed = value.trim();
        let help = ctx.t("a11y/anchor-is-valid.help");

        if trimmed.is_empty() {
            ctx.warn_at_with_help(
                ctx.t("a11y/anchor-is-valid.message_empty"),
                binding.range(),
                help,
            );
        } else if trimmed == "#" {
            ctx.warn_at_with_help(
                ctx.t("a11y/anchor-is-valid.message_hash"),
                binding.range(),
                help,
            );
        } else if has_javascript_scheme(trimmed) {
            ctx.warn_at_with_help(
                ctx.t("a11y/anchor-is-valid.message_javascript"),
                binding.range(),
                help,
            );
        }
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_component() {
            return;
        }

        if !element.is_unqualified_tag_exact("a") {
            return;
        }

        let mut saw_href = false;
        element.walk_bindings(&mut |binding| {
            if saw_href {
                return;
            }

            match binding.kind() {
                MarkupBindingKind::Attribute if binding.is_unqualified_arg_exact("href") => {
                    saw_href = true;
                    Self::check_static_href(ctx, binding, binding.static_value().unwrap_or(""));
                }
                MarkupBindingKind::Bind if binding.is_static_unqualified_arg_exact("href") => {
                    saw_href = true;
                }
                MarkupBindingKind::On | MarkupBindingKind::Model | MarkupBindingKind::Custom => {}
                MarkupBindingKind::Attribute => {}
                MarkupBindingKind::Bind => {}
            }
        });

        if !saw_href {
            ctx.warn_at_with_help(
                ctx.t("a11y/anchor-is-valid.message_missing"),
                element.range(),
                ctx.t("a11y/anchor-is-valid.help"),
            );
        }
    }
}

impl MarkupRule for AnchorIsValid {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for AnchorIsValid {
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
    use super::AnchorIsValid;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AnchorIsValid));
        Linter::with_registry(registry)
    }

    fn assert_template_warnings(source: &str, expected: usize) {
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, expected, "{:?}", result.diagnostics);
    }

    fn assert_jsx_warnings(source: &str, expected: usize) {
        let result = create_linter().lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(result.warning_count, expected, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_valid_href() {
        assert_template_warnings(r#"<a href="/about">About</a>"#, 0);
    }

    #[test]
    fn test_valid_dynamic_href() {
        assert_template_warnings(r#"<a :href="url">Link</a>"#, 0);
    }

    #[test]
    fn test_invalid_dynamic_href_argument() {
        assert_template_warnings(r#"<a :[href]="url">Link</a>"#, 1);
    }

    #[test]
    fn test_invalid_empty_href() {
        assert_template_warnings(r#"<a href="">Link</a>"#, 1);
    }

    #[test]
    fn test_invalid_hash_href() {
        assert_template_warnings(r##"<a href="#">Link</a>"##, 1);
    }

    #[test]
    fn test_invalid_javascript_href() {
        assert_template_warnings(r#"<a href="javascript:void(0)">Link</a>"#, 1);
    }

    #[test]
    fn test_invalid_mixed_case_javascript_href() {
        assert_template_warnings(r#"<a href="JaVaScRiPt:void(0)">Link</a>"#, 1);
    }

    #[test]
    fn test_invalid_obfuscated_javascript_href() {
        assert_template_warnings(r#"<a href="java&#x0A;script:void(0)">Link</a>"#, 1);
    }

    #[test]
    fn test_valid_similar_javascript_scheme() {
        assert_template_warnings(r#"<a href="javascriptx:void(0)">Link</a>"#, 0);
    }

    #[test]
    fn test_invalid_no_href() {
        assert_template_warnings(r#"<a>Link</a>"#, 1);
    }

    #[test]
    fn test_valid_component_skipped() {
        assert_template_warnings(r#"<NuxtLink>Link</NuxtLink>"#, 0);
    }

    #[test]
    fn test_valid_jsx_href() {
        assert_jsx_warnings(r#"const Link = () => <a href="/about">About</a>;"#, 0);
    }

    #[test]
    fn test_valid_jsx_dynamic_href() {
        assert_jsx_warnings(r#"const Link = () => <a href={url}>Link</a>;"#, 0);
    }

    #[test]
    fn test_invalid_jsx_hash_href() {
        assert_jsx_warnings(r##"const Link = () => <a href="#">Link</a>;"##, 1);
    }

    #[test]
    fn test_invalid_jsx_no_href() {
        assert_jsx_warnings(r#"const Link = () => <a>Link</a>;"#, 1);
    }

    #[test]
    fn test_invalid_jsx_spread_does_not_prove_href() {
        assert_jsx_warnings(r#"const Link = () => <a {...props}>Link</a>;"#, 1);
    }
}
