//! vue/prop-name-casing
//!
//! Enforce kebab-case for prop names in templates.
//!
//! In Vue templates, prop names should use kebab-case.
//! camelCase prop names will be warned.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <MyComponent myProp="value" />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent my-prop="value" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::is_html_tag;
use vize_croquis::naming::{hyphenate, is_camel_case};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/prop-name-casing",
    description: "Enforce kebab-case prop names in templates",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Enforce kebab-case prop names
#[derive(Default)]
pub struct PropNameCasing;

impl Rule for PropNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Only check props on components (native HTML elements use kebab-case attributes)
        if !is_component_like_tag(element) {
            return;
        }

        for prop in &element.props {
            match prop {
                PropNode::Attribute(attr) => {
                    let name = attr.name.as_str();
                    // Skip standard HTML attributes
                    if name == "class"
                        || name == "style"
                        || name == "key"
                        || name == "ref"
                        || name == "is"
                    {
                        continue;
                    }
                    if is_camel_case(name) {
                        let kebab = hyphenate(name);
                        ctx.warn_with_help(
                            ctx.t_fmt(
                                "vue/prop-name-casing.message",
                                &[("name", name), ("kebab", &kebab)],
                            ),
                            &attr.loc,
                            ctx.t("vue/prop-name-casing.help"),
                        );
                    }
                }
                PropNode::Directive(dir) => {
                    if dir.name == "bind"
                        && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                    {
                        let name = arg.content.as_ref();
                        // Skip standard bindings
                        if name == "class"
                            || name == "style"
                            || name == "key"
                            || name == "ref"
                            || name == "is"
                        {
                            continue;
                        }
                        if is_camel_case(name) {
                            let kebab = hyphenate(name);
                            ctx.warn_with_help(
                                ctx.t_fmt(
                                    "vue/prop-name-casing.message",
                                    &[("name", name), ("kebab", &kebab)],
                                ),
                                &dir.loc,
                                ctx.t("vue/prop-name-casing.help"),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn is_component_like_tag(element: &ElementNode<'_>) -> bool {
    if element.tag_type == ElementType::Component {
        return true;
    }

    let tag = element.tag.as_str();
    tag == "component" || (tag.contains('-') && !is_html_tag(tag))
}

#[cfg(test)]
mod tests {
    use super::PropNameCasing;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PropNameCasing));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_kebab_case_prop() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent my-prop="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_native_element() {
        let linter = create_linter();
        // Native elements are not checked
        let result = linter.lint_template(r#"<div data-my-attr="value"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_camel_case_prop() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent myProp="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_camel_case_prop_on_kebab_case_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<my-component myProp="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_camel_case_prop_on_dynamic_component() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<component :is="tagName" myProp="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_camel_case_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent :myProp="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_camel_case_binding_on_dynamic_component() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<component :is="tagName" :myProp="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_standard_attrs_skipped() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<MyComponent class="foo" :style="bar" key="1" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_aria_attributes() {
        let linter = create_linter();
        // aria-* attributes are already kebab-case, should not warn
        let result = linter.lint_template(r#"<MyComponent aria-label="close" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_single_word_prop() {
        let linter = create_linter();
        // Single word props like "disabled" are not camelCase, should not warn
        let result = linter.lint_template(r#"<MyComponent disabled />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
