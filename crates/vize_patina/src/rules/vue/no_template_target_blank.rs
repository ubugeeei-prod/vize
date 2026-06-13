//! vue/no-template-target-blank
//!
//! Disallow `target="_blank"` on links without `rel="noopener"` /
//! `rel="noreferrer"`.
//!
//! A link that opens in a new tab with `target="_blank"` gives the opened page
//! a reference to the opener via `window.opener`, which it can use to redirect
//! the original tab (reverse tabnabbing). Adding `rel="noopener"` (or
//! `noreferrer`) severs that reference.
//!
//! This is the cross-framework analogue of `react/jsx-no-target-blank` and
//! `svelte/no-target-blank`. The same logic runs over a Vue template and over
//! JSX/TSX.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <a href="https://example.com" target="_blank">x</a>
//! ```
//!
//! ### Valid
//! ```vue
//! <a href="https://example.com" target="_blank" rel="noopener">x</a>
//! <a href="https://example.com" target="_blank" rel="noreferrer">x</a>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-target-blank",
    description: "Disallow target=\"_blank\" without rel=\"noopener\"",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow target="_blank" without rel="noopener"/"noreferrer"
#[derive(Default)]
pub struct NoTemplateTargetBlank;

/// Whether a `rel` value safely opts out of `window.opener` access.
fn rel_is_safe(rel: &str) -> bool {
    rel.split_whitespace().any(|token| {
        token.eq_ignore_ascii_case("noopener") || token.eq_ignore_ascii_case("noreferrer")
    })
}

impl MarkupRule for NoTemplateTargetBlank {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        // Only static `target="_blank"`; a dynamic `:target` cannot be checked.
        let is_blank = element
            .static_attribute("target")
            .and_then(|attr| attr.value())
            .is_some_and(|value| value.trim() == "_blank");
        if !is_blank {
            return;
        }
        // The reverse-tabnabbing risk only applies to links that navigate.
        if !element.has_static_attribute("href") && !element.has_bound_attribute("href") {
            return;
        }
        let rel_is_safe = element
            .static_attribute("rel")
            .and_then(|attr| attr.value())
            .is_some_and(rel_is_safe);
        if rel_is_safe {
            return;
        }
        let message = ctx.lint().t("vue/no-template-target-blank.message");
        let help = ctx.lint().t("vue/no-template-target-blank.help");
        ctx.lint().warn_at_with_help(message, element.range(), help);
    }
}

impl Rule for NoTemplateTargetBlank {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let is_blank =
            static_attribute_value(element, "target").is_some_and(|value| value.trim() == "_blank");
        if !is_blank {
            return;
        }
        if !has_attribute_or_binding(element, "href") {
            return;
        }
        if static_attribute_value(element, "rel").is_some_and(rel_is_safe) {
            return;
        }
        ctx.warn_with_help(
            ctx.t("vue/no-template-target-blank.message"),
            &element.loc,
            ctx.t("vue/no-template-target-blank.help"),
        );
    }
}

/// The value of a static `name` attribute (empty string when valueless), or
/// `None` when no such static attribute exists.
fn static_attribute_value<'a>(element: &'a ElementNode<'a>, name: &str) -> Option<&'a str> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == name => Some(
            attr.value
                .as_ref()
                .map(|v| v.content.as_str())
                .unwrap_or(""),
        ),
        _ => None,
    })
}

/// Whether `element` has a static `name` attribute or a `v-bind:name` directive.
fn has_attribute_or_binding(element: &ElementNode, name: &str) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == name,
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(&dir.arg, Some(ExpressionNode::Simple(s)) if s.content == name)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::NoTemplateTargetBlank;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoTemplateTargetBlank));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_noopener() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target="_blank" rel="noopener">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_noreferrer() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target="_blank" rel="noreferrer nofollow">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_same_tab() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="https://example.com">x</a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_missing_rel() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target="_blank">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_bound_href() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a :href="url" target="_blank">x</a>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_jsx_missing_rel_reports() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <a href="https://example.com" target="_blank">x</a>;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_jsx_with_noopener_ok() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <a href="https://example.com" target="_blank" rel="noopener">x</a>;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 0);
    }
}
