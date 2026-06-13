//! vue/html-button-has-type
//!
//! Require an explicit valid `type` on `<button>` elements.
//!
//! A `<button>` without a `type` attribute defaults to `type="submit"`, which
//! inside a `<form>` submits the form on every click. That implicit behavior is
//! a frequent source of bugs, so this rule asks for an explicit `type` whose
//! value is one of `button`, `submit`, or `reset`.
//!
//! This is the cross-framework analogue of `react/button-has-type` and
//! `svelte/button-has-type`. The same logic runs over a Vue `<button>` and a
//! JSX `<button />`.
//!
//! A bound `:type` / `type={…}` is skipped because its value cannot be
//! validated statically.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <button>Click</button>
//! <button type="foo">Click</button>
//! ```
//!
//! ### Valid
//! ```vue
//! <button type="button">Click</button>
//! <button type="submit">Save</button>
//! <button type="reset">Reset</button>
//! <button :type="dynamicType">Click</button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/html-button-has-type",
    description: "Require an explicit valid type on button elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// The set of valid `<button>` type values.
const VALID_TYPES: [&str; 3] = ["button", "submit", "reset"];

fn is_valid_type(value: &str) -> bool {
    VALID_TYPES.contains(&value)
}

/// Require an explicit valid type on button elements
#[derive(Default)]
pub struct HtmlButtonHasType;

impl MarkupRule for HtmlButtonHasType {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !element.is_tag("button") {
            return;
        }

        // A bound `:type` / `type={…}` cannot be validated statically; skip it.
        if element.has_bound_attribute("type") {
            return;
        }

        let range = element.range();
        match element.static_attribute("type") {
            Some(attr) => {
                let value = attr.value().unwrap_or("");
                if !is_valid_type(value) {
                    let message = ctx
                        .lint()
                        .t_fmt("vue/html-button-has-type.invalid", &[("type", value)]);
                    let help = ctx.lint().t("vue/html-button-has-type.help");
                    ctx.lint().warn_at_with_help(message, range, help);
                }
            }
            None => {
                let message = ctx.lint().t("vue/html-button-has-type.missing");
                let help = ctx.lint().t("vue/html-button-has-type.help");
                ctx.lint().warn_at_with_help(message, range, help);
            }
        }
    }
}

impl Rule for HtmlButtonHasType {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag != "button" {
            return;
        }

        // A bound `:type` cannot be validated statically; skip it.
        if has_bound_type(element) {
            return;
        }

        match static_type_value(element) {
            Some(value) => {
                if !is_valid_type(value) {
                    let message = ctx.t_fmt("vue/html-button-has-type.invalid", &[("type", value)]);
                    ctx.warn_with_help(
                        message,
                        &element.loc,
                        ctx.t("vue/html-button-has-type.help"),
                    );
                }
            }
            None => {
                ctx.warn_with_help(
                    ctx.t("vue/html-button-has-type.missing"),
                    &element.loc,
                    ctx.t("vue/html-button-has-type.help"),
                );
            }
        }
    }
}

/// The value of a static `type` attribute, if present.
fn static_type_value<'a>(element: &'a ElementNode) -> Option<&'a str> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == "type" => Some(
            attr.value
                .as_ref()
                .map(|v| v.content.as_str())
                .unwrap_or(""),
        ),
        _ => None,
    })
}

/// Whether the element has a `v-bind:type` / `:type` directive.
fn has_bound_type(element: &ElementNode) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Directive(dir) => {
            dir.name == "bind"
                && matches!(&dir.arg, Some(ExpressionNode::Simple(s)) if s.content == "type")
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::HtmlButtonHasType;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlButtonHasType));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_type_button() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button type="button">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_type_submit() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button type="submit">Save</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_type_reset() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button type="reset">Reset</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_bound_type_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button :type="dynamicType">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_missing_type() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button>Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_bad_type_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button type="foo">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_type_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button type="">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_non_button_ignored() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="foo" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_jsx_missing_type_reports() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <button>Click</button>;"#,
            "f.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_jsx_invalid_type_reports() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <button type="foo">Click</button>;"#,
            "f.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_jsx_valid_type_ok() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <button type="button">Click</button>;"#,
            "f.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_jsx_bound_type_skipped() {
        let linter = create_linter();
        let result = linter.lint_jsx(
            r#"const A = () => <button type={t}>Click</button>;"#,
            "f.jsx",
            JsxLang::Jsx,
        );
        assert_eq!(result.warning_count, 0);
    }
}
