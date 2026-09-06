//! vue/no-template-target-blank
//!
//! Disallow `target="_blank"` on links without `rel="noopener noreferrer"`.
//!
//! A link that opens in a new tab with `target="_blank"` gives the opened page
//! a reference to the opener via `window.opener`, which it can use to redirect
//! the original tab (reverse tabnabbing). Adding `rel="noopener noreferrer"`
//! severs that reference while also matching eslint-plugin-vue's default
//! referrer policy.
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
//! <a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-target-blank",
    description: "Disallow target=\"_blank\" without rel=\"noopener noreferrer\"",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow target="_blank" without rel="noopener noreferrer"
#[derive(Default)]
pub struct NoTemplateTargetBlank;

/// Whether a `rel` value safely opts out of `window.opener` access.
fn rel_is_safe(rel: &str) -> bool {
    let mut has_noopener = false;
    let mut has_noreferrer = false;
    for token in rel.split_whitespace() {
        if token.eq_ignore_ascii_case("noopener") {
            has_noopener = true;
        } else if token.eq_ignore_ascii_case("noreferrer") {
            has_noreferrer = true;
        }
    }
    has_noopener && has_noreferrer
}

fn is_external_href(value: &str) -> bool {
    if value.starts_with("//") {
        return true;
    }
    let Some(colon) = value.as_bytes().iter().position(|&byte| byte == b':') else {
        return false;
    };
    colon > 0
        && value.as_bytes()[..colon]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
}

fn markup_has_dangerous_href(element: &MarkupElement<'_>) -> bool {
    element
        .static_attribute("href")
        .and_then(|attr| attr.value())
        .is_some_and(is_external_href)
        || element.has_bound_attribute("href")
}

impl MarkupRule for NoTemplateTargetBlank {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        // Only static `target="_blank"`; a dynamic `:target` cannot be checked.
        let Some(target) = element.static_attribute("target") else {
            return;
        };
        // Exact match, like the eslint-plugin-vue baseline: a padded
        // `target=" _blank "` is a browsing-context name, not the keyword.
        if target.value().is_none_or(|value| value != "_blank") {
            return;
        }
        // Match eslint-plugin-vue: static relative links are clean, while
        // external static links and dynamic href bindings are checked.
        if !markup_has_dangerous_href(element) {
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
        ctx.lint().warn_at_with_help(message, target.range(), help);
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
        let Some(target) = static_attribute(element, "target") else {
            return;
        };
        if attribute_value(target) != "_blank" {
            return;
        }
        if !has_dangerous_href(element) {
            return;
        }
        if static_attribute_value(element, "rel").is_some_and(rel_is_safe) {
            return;
        }
        ctx.warn_with_help(
            ctx.t("vue/no-template-target-blank.message"),
            &target.loc,
            ctx.t("vue/no-template-target-blank.help"),
        );
    }
}

fn static_attribute<'a>(
    element: &'a ElementNode<'a>,
    name: &str,
) -> Option<&'a vize_relief::AttributeNode<'a>> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == name => Some(&**attr),
        _ => None,
    })
}

/// The value of a static attribute, or empty string when it is valueless.
fn attribute_value<'a>(attr: &'a vize_relief::AttributeNode<'a>) -> &'a str {
    attr.value.as_ref().map(|v| v.content).unwrap_or("")
}

/// The value of a static `name` attribute, or empty string when valueless.
fn static_attribute_value<'a>(element: &'a ElementNode<'a>, name: &str) -> Option<&'a str> {
    static_attribute(element, name).map(attribute_value)
}

fn has_dangerous_href(element: &ElementNode) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => {
            attr.name == "href"
                && attr
                    .value
                    .as_ref()
                    .is_some_and(|value| is_external_href(value.content))
        }
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(&dir.arg, Some(ExpressionNode::Simple(s)) if s.content == "href")
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

    fn assert_single_diagnostic_covers(source: &str, needle: &str, result: crate::LintResult) {
        assert_eq!(
            result.warning_count, 1,
            "expected one warning for source: {source}"
        );
        assert_eq!(
            result.diagnostics.len(),
            1,
            "expected one diagnostic for source: {source}"
        );
        let diagnostic = &result.diagnostics[0];
        let start = source.find(needle).expect("needle must exist") as u32;
        let end = start + needle.len() as u32;
        assert_eq!(diagnostic.start, start);
        assert_eq!(diagnostic.end, end);
    }

    #[test]
    fn test_valid_with_noopener_noreferrer() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_with_noopener_only() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target="_blank" rel="noopener">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_with_noreferrer_only() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target="_blank" rel="noreferrer nofollow">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
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
    fn test_valid_relative_href() {
        let linter = create_linter();
        let result =
            linter.lint_template(r##"<a href="/guide" target="_blank">x</a>"##, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_missing_rel_reports_target_attribute_range() {
        // The non-ASCII label keeps the reported range honest about byte
        // offsets: a code-point or UTF-16 based span would drift here.
        let source = r#"<a aria-label="日本語" href="https://example.com" target="_blank">x</a>"#;
        let linter = create_linter();
        let result = linter.lint_template(source, "test.vue");
        assert_single_diagnostic_covers(source, r#"target="_blank""#, result);
    }

    #[test]
    fn test_valid_padded_target_value() {
        // Only the exact `_blank` keyword opens a new browsing context; a
        // padded value is a context name, and the eslint-plugin-vue baseline
        // compares exactly too.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a href="https://example.com" target=" _blank ">x</a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_padded_target_value_jsx() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <a href="https://example.com" target=" _blank ">x</a>;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 0);
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
    fn test_jsx_relative_href_is_clean() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <a href="/guide" target="_blank">x</a>;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_jsx_missing_rel_reports_target_attribute_range() {
        let source = r#"const A = () => <a aria-label="日本語" href="https://example.com" target="_blank">x</a>;"#;
        let linter = create_linter();
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_single_diagnostic_covers(source, r#"target="_blank""#, result);
    }

    #[test]
    fn test_jsx_with_noopener_only_reports() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <a href="https://example.com" target="_blank" rel="noopener">x</a>;"#,
            "test.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 1);
    }
}
