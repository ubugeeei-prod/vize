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
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/html-button-has-type",
    description: "Require an explicit valid type on button elements",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Button type values accepted by `vue/html-button-has-type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HtmlButtonHasTypeOptions {
    pub button: bool,
    pub submit: bool,
    pub reset: bool,
}

impl Default for HtmlButtonHasTypeOptions {
    fn default() -> Self {
        Self {
            button: true,
            submit: true,
            reset: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypePolicy {
    Allowed,
    Empty,
    Invalid,
    Forbidden,
}

/// Require an explicit valid type on button elements
#[derive(Default)]
pub struct HtmlButtonHasType;

pub(crate) struct ConfiguredHtmlButtonHasType {
    options: HtmlButtonHasTypeOptions,
}

impl HtmlButtonHasType {
    pub(crate) fn configured(
        button: bool,
        submit: bool,
        reset: bool,
    ) -> ConfiguredHtmlButtonHasType {
        ConfiguredHtmlButtonHasType {
            options: HtmlButtonHasTypeOptions {
                button,
                submit,
                reset,
            },
        }
    }
}

fn classify(options: HtmlButtonHasTypeOptions, value: &str) -> TypePolicy {
    match value {
        "" => TypePolicy::Empty,
        "button" if options.button => TypePolicy::Allowed,
        "submit" if options.submit => TypePolicy::Allowed,
        "reset" if options.reset => TypePolicy::Allowed,
        "button" | "submit" | "reset" => TypePolicy::Forbidden,
        _ => TypePolicy::Invalid,
    }
}

fn lint_markup<'a>(
    options: HtmlButtonHasTypeOptions,
    ctx: &mut MarkupContext<'_, 'a>,
    element: &MarkupElement<'a>,
) {
    if !element.is_unqualified_tag_exact("button") {
        return;
    }

    // A populated bound `:type` / `type={…}` cannot be validated statically.
    // eslint-plugin-vue still reports an authored Vue binding with no expression.
    if let Some(binding) = element.binding(MarkupBindingKind::Bind, "type") {
        if ctx.is_jsx() || binding.expression().is_some() {
            return;
        }
        let message = ctx.lint().t("vue/html-button-has-type.empty");
        let help = ctx.lint().t("vue/html-button-has-type.help");
        ctx.lint().warn_at_with_help(message, element.range(), help);
        return;
    }

    let range = element.range();
    match element.static_attribute("type") {
        Some(attr) => {
            let value = attr.value().unwrap_or("");
            let message_key = match classify(options, value) {
                TypePolicy::Allowed => return,
                TypePolicy::Empty => "vue/html-button-has-type.empty",
                TypePolicy::Invalid => "vue/html-button-has-type.invalid",
                TypePolicy::Forbidden => "vue/html-button-has-type.forbidden",
            };
            let message = ctx.lint().t_fmt(message_key, &[("type", value)]);
            let help = ctx.lint().t("vue/html-button-has-type.help");
            ctx.lint().warn_at_with_help(message, range, help);
        }
        None => {
            let message = ctx.lint().t("vue/html-button-has-type.missing");
            let help = ctx.lint().t("vue/html-button-has-type.help");
            ctx.lint().warn_at_with_help(message, range, help);
        }
    }
}

fn lint_relief(
    options: HtmlButtonHasTypeOptions,
    ctx: &mut LintContext<'_>,
    element: &ElementNode<'_>,
) {
    if element.tag != "button" {
        return;
    }

    // A populated bound `:type` cannot be validated statically; skip it.
    if let Some(has_expression) = bound_type_has_expression(element) {
        if has_expression {
            return;
        }
        ctx.warn_with_help(
            ctx.t("vue/html-button-has-type.empty"),
            &element.loc,
            ctx.t("vue/html-button-has-type.help"),
        );
        return;
    }

    match static_type_value(element) {
        Some(value) => {
            let message_key = match classify(options, value) {
                TypePolicy::Allowed => return,
                TypePolicy::Empty => "vue/html-button-has-type.empty",
                TypePolicy::Invalid => "vue/html-button-has-type.invalid",
                TypePolicy::Forbidden => "vue/html-button-has-type.forbidden",
            };
            let message = ctx.t_fmt(message_key, &[("type", value)]);
            ctx.warn_with_help(
                message,
                &element.loc,
                ctx.t("vue/html-button-has-type.help"),
            );
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

trait ButtonTypePolicy: Send + Sync {
    fn options(&self) -> HtmlButtonHasTypeOptions;
}

impl ButtonTypePolicy for HtmlButtonHasType {
    fn options(&self) -> HtmlButtonHasTypeOptions {
        HtmlButtonHasTypeOptions::default()
    }
}

impl ButtonTypePolicy for ConfiguredHtmlButtonHasType {
    fn options(&self) -> HtmlButtonHasTypeOptions {
        self.options
    }
}

impl<T: ButtonTypePolicy> MarkupRule for T {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        lint_markup(self.options(), ctx, element);
    }
}

impl<T: ButtonTypePolicy> Rule for T {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        lint_relief(self.options(), ctx, element);
    }
}

/// The value of a static `type` attribute, if present.
fn static_type_value<'a>(element: &'a ElementNode) -> Option<&'a str> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == "type" => {
            Some(attr.value.as_ref().map(|v| v.content).unwrap_or(""))
        }
        _ => None,
    })
}

/// Whether a `v-bind:type` / `:type` directive has an authored expression.
fn bound_type_has_expression(element: &ElementNode) -> Option<bool> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) => (dir.name == "bind"
            && matches!(&dir.arg, Some(ExpressionNode::Simple(s)) if s.content == "type"))
        .then(|| match &dir.exp {
            Some(ExpressionNode::Simple(expression)) => !expression.content.trim().is_empty(),
            Some(_) => true,
            None => false,
        }),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::{HtmlButtonHasType, HtmlButtonHasTypeOptions};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;
    use vize_atelier_jsx::JsxLang;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlButtonHasType));
        Linter::with_registry(registry)
    }

    fn create_configured_linter(options: HtmlButtonHasTypeOptions) -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HtmlButtonHasType::configured(
            options.button,
            options.submit,
            options.reset,
        )));
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
    fn test_empty_bound_type_reports() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button :type="">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].message,
            "A value must be set for the '<button>' type attribute"
        );
    }

    #[test]
    fn configured_type_policy_allows_only_enabled_values() {
        let linter = create_configured_linter(HtmlButtonHasTypeOptions {
            button: false,
            submit: true,
            reset: false,
        });
        let result = linter.lint_template(
            r#"<div><button type="button">Button</button><button type="submit">Submit</button><button type="reset">Reset</button></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2, "{:#?}", result.diagnostics);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "'button' is forbidden by the configured '<button>' type policy",
                "'reset' is forbidden by the configured '<button>' type policy",
            ]
        );
    }

    #[test]
    fn configured_type_policy_keeps_missing_invalid_and_dynamic_boundaries() {
        let linter = create_configured_linter(HtmlButtonHasTypeOptions {
            button: false,
            submit: false,
            reset: false,
        });
        let result = linter.lint_template(
            r#"<div><button>Missing</button><button type="menu">Invalid</button><button :type="dynamicType">Dynamic</button></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2, "{:#?}", result.diagnostics);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "'<button>' is missing a 'type' attribute; add type=\"button\", \"submit\", or \"reset\"",
                "'menu' is not a valid 'type' for '<button>'; use \"button\", \"submit\", or \"reset\"",
            ]
        );
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
    fn test_pascal_case_button_component_ignored() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Button>Click</Button>"#, "test.vue");
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
