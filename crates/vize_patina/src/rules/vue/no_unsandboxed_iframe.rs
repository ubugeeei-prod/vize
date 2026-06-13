//! vue/no-unsandboxed-iframe
//!
//! Require a `sandbox` attribute on `<iframe>` elements.
//!
//! An `<iframe>` without a `sandbox` attribute runs embedded content with full
//! privileges (scripts, forms, top-level navigation, popups). Adding `sandbox`
//! — even an empty `sandbox=""` — opts the frame into the most restrictive
//! policy and lets you re-grant only the capabilities you need.
//!
//! This is the cross-framework analogue of `react/iframe-missing-sandbox`. The
//! same logic runs over a Vue `<iframe>` and a JSX `<iframe />`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <iframe src="/embed"></iframe>
//! ```
//!
//! ### Valid
//! ```vue
//! <iframe src="/embed" sandbox></iframe>
//! <iframe src="/embed" sandbox="allow-scripts"></iframe>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unsandboxed-iframe",
    description: "Require a sandbox attribute on iframe elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require a sandbox attribute on iframe elements
#[derive(Default)]
pub struct NoUnsandboxedIframe;

impl MarkupRule for NoUnsandboxedIframe {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !element.is_tag("iframe") {
            return;
        }
        if element.has_static_attribute("sandbox") || element.has_bound_attribute("sandbox") {
            return;
        }
        let message = ctx.lint().t("vue/no-unsandboxed-iframe.message");
        let help = ctx.lint().t("vue/no-unsandboxed-iframe.help");
        ctx.lint().warn_at_with_help(message, element.range(), help);
    }
}

impl Rule for NoUnsandboxedIframe {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag != "iframe" {
            return;
        }
        if has_attribute_or_binding(element, "sandbox") {
            return;
        }
        ctx.warn_with_help(
            ctx.t("vue/no-unsandboxed-iframe.message"),
            &element.loc,
            ctx.t("vue/no-unsandboxed-iframe.help"),
        );
    }
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
    use super::NoUnsandboxedIframe;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoUnsandboxedIframe));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_sandboxed_iframe() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<iframe src="/embed" sandbox></iframe>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_sandbox_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<iframe src="/embed" sandbox="allow-scripts"></iframe>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_unsandboxed_iframe() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<iframe src="/embed"></iframe>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_jsx_unsandboxed_iframe_reports() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <iframe src="/embed" />;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_jsx_sandboxed_iframe_ok() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <iframe src="/embed" sandbox="" />;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 0);
    }
}
