//! a11y/heading-has-content
//!
//! Require heading elements (h1-h6) to have accessible content.
//!
//! Empty headings are not accessible to screen reader users.
//!
//! Based on eslint-plugin-vuejs-accessibility heading-has-content rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupNode, MarkupRule,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::markup_helpers::get_static_or_bound_literal_markup_value;

static META: RuleMeta = RuleMeta {
    name: "a11y/heading-has-content",
    description: "Require heading elements to have accessible content",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require heading elements to have accessible content
#[derive(Default)]
pub struct HeadingHasContent;

impl HeadingHasContent {
    fn heading_tag<'a>(element: &MarkupElement<'a>) -> Option<&'a str> {
        const HEADINGS: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];
        if let Some(tag) = HEADINGS
            .iter()
            .copied()
            .find(|tag| element.is_unqualified_tag_exact(tag))
        {
            return Some(tag);
        }
        if !element.is_unqualified_tag_exact("component") {
            return None;
        }
        get_static_or_bound_literal_markup_value(element, "is").filter(|tag| HEADINGS.contains(tag))
    }

    fn is_hidden_from_accessibility_tree(element: &MarkupElement<'_>) -> bool {
        let mut hidden = false;
        element.walk_bindings(&mut |binding| {
            if hidden {
                return;
            }

            if matches!(
                binding.kind(),
                MarkupBindingKind::Attribute | MarkupBindingKind::Bind
            ) && binding.is_static_unqualified_arg_eq_ignore_ascii_case("aria-hidden")
            {
                let false_literal = binding.literal_string_value() == Some("false")
                    || binding.kind() == MarkupBindingKind::Bind
                        && binding
                            .expression()
                            .is_some_and(|expression| expression.trim() == "false");
                hidden = !false_literal;
            }
        });
        hidden
    }

    fn has_rendered_text_directive(element: &MarkupElement<'_>) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            found |= binding.kind() == MarkupBindingKind::Custom
                && (binding.is_static_unqualified_arg_exact("text")
                    || binding.is_static_unqualified_arg_exact("html"))
                && binding.has_value();
        });
        element.walk_children(&mut |child| {
            if !found && let MarkupNode::Element(child) = child {
                found = Self::has_rendered_text_directive(&child);
            }
        });
        found
    }

    fn interpolation_has_content(source: &str, range: crate::ir::ByteRange) -> bool {
        let Some(authored) = source.get(range.start as usize..range.end as usize) else {
            return true;
        };
        let expression = authored
            .strip_prefix("{{")
            .and_then(|value| value.strip_suffix("}}"))
            .unwrap_or(authored)
            .trim();
        expression != "undefined"
    }

    fn has_accessible_content(source: &str, element: &MarkupElement<'_>) -> bool {
        if Self::has_rendered_text_directive(element) {
            return true;
        }

        if element.is_unqualified_tag_exact("img") {
            let mut named = false;
            element.walk_bindings(&mut |binding| {
                if binding.is_unqualified_arg_exact("alt") {
                    named |= match binding.literal_string_value() {
                        Some(value) => !value.trim().is_empty(),
                        None if binding.kind() == MarkupBindingKind::Bind => true,
                        None => false,
                    };
                }
            });
            return named;
        }

        let mut has_content = false;
        element.walk_children(&mut |child| match child {
            MarkupNode::Text(text) if text.is_significant() => {
                has_content = true;
            }
            MarkupNode::Interpolation(range) => {
                has_content |= Self::interpolation_has_content(source, range);
            }
            MarkupNode::Element(child_element)
                if child_element.kind() == MarkupElementKind::Slot
                    || child_element.is_unqualified_tag_exact("slot") =>
            {
                has_content = true;
            }
            MarkupNode::Element(child_element)
                if !Self::is_hidden_from_accessibility_tree(&child_element)
                    && Self::has_accessible_content(source, &child_element) =>
            {
                has_content = true;
            }
            MarkupNode::Text(_)
            | MarkupNode::Element(_)
            | MarkupNode::Comment(_)
            | MarkupNode::If(_)
            | MarkupNode::For(_)
            | MarkupNode::Other(_) => {}
        });

        has_content
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        let Some(tag) = Self::heading_tag(element) else {
            return;
        };

        if !Self::has_accessible_content(ctx.source, element) {
            ctx.warn_at_with_help(
                ctx.t_fmt("a11y/heading-has-content.message", &[("tag", tag)]),
                element.range(),
                ctx.t("a11y/heading-has-content.help"),
            );
        }
    }
}

impl MarkupRule for HeadingHasContent {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> crate::markup::MarkupTraversal {
        crate::markup::MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ctx.is_jsx_attribute_value() {
            return;
        }

        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for HeadingHasContent {
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
    use super::HeadingHasContent;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(HeadingHasContent));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_with_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1>Hello World</h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h2>{{ title }}</h2>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_empty_aria_hidden_heading() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 aria-hidden="true"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_static_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 aria-label="Dashboard"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_bound_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 :aria-label="title"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_bound_aria_labelledby() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1 :aria-labelledby="labelId"></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_with_default_slot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<h1><slot></slot></h1>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn matches_eslint_plugin_vuejs_accessibility_content_boundaries() {
        let result = create_linter().lint_template(
            r#"<div>
                <h2 v-text="title"></h2>
                <h2 v-html="html"></h2>
                <h2><span v-text="title"></span></h2>
                <h2><span v-html="html"></span></h2>
                <h2 aria-label="Title"></h2>
                <h2 :aria-label="title"></h2>
                <h2 aria-labelledby="other"></h2>
                <h2 aria-hidden="true"></h2>
                <h2><MyIcon name="star" aria-label="Title" /></h2>
                <h2><span aria-label="Title"></span></h2>
                <h2><svg role="img" aria-label="Title"><path d="M0 0h1" /></svg></h2>
                <h2><span :aria-hidden="true">Title</span></h2>
                <h2><span :aria-hidden="hidden">Title</span></h2>
                <h2><span aria-hidden>Title</span></h2>
                <h2>{{ undefined }}</h2>
                <component is="h2"></component>
            </div>"#,
            "test.vue",
        );

        assert_eq!(result.warning_count, 12, "{:?}", result.diagnostics);
    }

    #[test]
    fn content_survives_false_hidden_bindings_and_empty_later_interpolations() {
        let result = create_linter().lint_template(
            r#"<div>
                <h2><span :aria-hidden="false">Title</span></h2>
                <h2>Title {{ undefined }}</h2>
            </div>"#,
            "test.vue",
        );

        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }
}
