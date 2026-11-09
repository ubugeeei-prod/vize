//! a11y/anchor-has-content
//!
//! Require anchor elements to have accessible content.
//!
//! Anchor elements without content are not accessible to screen reader users.
//! Content can be text, images with alt text, or elements with aria-label.
//!
//! Based on eslint-plugin-vuejs-accessibility anchor-has-content rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupNode, MarkupRule,
    MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::a11y::helpers::{
    canonical_component_name, get_static_or_bound_literal_attribute_value, is_aria_hidden_true,
    is_slot_element, matches_configured_component, string_literal_value,
};
use crate::rules::a11y::markup_helpers;
use vize_relief::{ElementNode, ExpressionNode, PropNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "a11y/anchor-has-content",
    description: "Require anchor elements to have accessible content",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require anchor elements to have accessible content
pub struct AnchorHasContent {
    accessible_children: Vec<vize_l0::String>,
}

#[expect(
    non_upper_case_globals,
    reason = "the unit-like rule value intentionally shares the rule type name"
)]
/// Default `a11y/anchor-has-content` rule with no project-local components.
pub const AnchorHasContent: AnchorHasContent = AnchorHasContent {
    accessible_children: Vec::new(),
};

impl Default for AnchorHasContent {
    fn default() -> Self {
        AnchorHasContent
    }
}

impl AnchorHasContent {
    pub(crate) fn with_accessible_children(children: Vec<vize_l0::String>) -> Self {
        Self {
            accessible_children: children
                .into_iter()
                .map(|name| canonical_component_name(&name))
                .collect(),
        }
    }

    fn has_accessible_content(&self, element: &ElementNode) -> bool {
        // Check for aria-label or aria-labelledby
        for prop in &element.props {
            if let PropNode::Attribute(attr) = prop
                && (attr.name == "aria-label" || attr.name == "aria-labelledby")
                && attr
                    .value
                    .as_ref()
                    .is_some_and(|value| !value.content.is_empty())
            {
                return true;
            }
            if let PropNode::Directive(dir) = prop
                && dir.name == "bind"
                && let Some(vize_relief::ExpressionNode::Simple(s)) = &dir.arg
                && (s.content == "aria-label" || s.content == "aria-labelledby")
                && dir.exp.as_ref().is_some_and(|expression| match expression {
                    ExpressionNode::Simple(simple) => nonempty_expression(simple.content),
                    ExpressionNode::Compound(compound) => !compound.children.is_empty(),
                })
            {
                return true;
            }
        }

        // Check for content in children
        for child in &element.children {
            match child {
                TemplateChildNode::Text(text) if !text.content.trim().is_empty() => {
                    return true;
                }
                TemplateChildNode::Interpolation(_) => {
                    return true;
                }
                TemplateChildNode::Element(el) => {
                    if is_aria_hidden_true(el) {
                        continue;
                    }
                    if has_content_directive(el) {
                        return true;
                    }
                    if is_slot_element(el) {
                        return true;
                    }
                    if matches_configured_component(el.tag, &self.accessible_children) {
                        return true;
                    }
                    // Check for img with alt
                    if el.tag == "img" {
                        for prop in &el.props {
                            match prop {
                                PropNode::Attribute(attr)
                                    if attr.name == "alt"
                                        && attr
                                            .value
                                            .as_ref()
                                            .is_some_and(|value| !value.content.is_empty()) =>
                                {
                                    return true;
                                }
                                PropNode::Directive(dir) if dir.name == "bind" => {
                                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg
                                        && arg.is_static
                                        && arg.content == "alt"
                                        && let Some(ExpressionNode::Simple(exp)) = &dir.exp
                                        && string_literal_value(exp.content)
                                            .is_none_or(|value| !value.is_empty())
                                    {
                                        return true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    // Recursively check other elements
                    if self.has_accessible_content(el) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }
}

impl AnchorHasContent {
    fn has_markup_content(&self, element: &MarkupElement<'_>) -> bool {
        let mut labelled = false;
        element.walk_bindings(&mut |binding| {
            labelled |= match binding.kind() {
                MarkupBindingKind::Attribute => {
                    (binding.is_unqualified_arg_exact("aria-label")
                        || binding.is_unqualified_arg_exact("aria-labelledby"))
                        && binding
                            .static_value()
                            .is_some_and(|value| !value.is_empty())
                }
                MarkupBindingKind::Bind => {
                    (binding.is_unqualified_arg_exact("aria-label")
                        || binding.is_unqualified_arg_exact("aria-labelledby"))
                        && binding.expression().is_some_and(nonempty_expression)
                }
                _ => false,
            };
        });
        if labelled {
            return true;
        }
        let mut found = false;
        element.walk_children(&mut |child| {
            if found {
                return;
            }
            found = match child {
                MarkupNode::Text(text) => text.is_significant(),
                MarkupNode::Interpolation(_) => true,
                MarkupNode::Element(child) => {
                    !markup_helpers::is_aria_hidden_true_markup(&child)
                        && (has_markup_content_directive(&child)
                            || child.kind() == MarkupElementKind::Slot
                            || child.is_unqualified_tag_exact("slot")
                            || matches_configured_component(child.tag(), &self.accessible_children)
                            || (child.is_unqualified_tag_exact("img")
                                && Self::has_markup_alt(&child))
                            || self.has_markup_content(&child))
                }
                _ => false,
            };
        });
        found
    }

    fn has_markup_alt(image: &MarkupElement<'_>) -> bool {
        let mut found = false;
        image.walk_bindings(&mut |binding| {
            found |= match binding.kind() {
                MarkupBindingKind::Attribute => {
                    binding.is_unqualified_arg_exact("alt")
                        && binding
                            .static_value()
                            .is_some_and(|value| !value.is_empty())
                }
                // A blank or non-literal bound alt counts, as on the template path.
                MarkupBindingKind::Bind => {
                    binding.is_static_unqualified_arg_exact("alt")
                        && binding
                            .literal_string_value()
                            .is_none_or(|value| !value.is_empty())
                }
                _ => false,
            };
        });
        found
    }
}

fn nonempty_expression(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || matches!(value, "null" | "undefined") {
        return false;
    }
    string_literal_value(value).is_none_or(|literal| !literal.is_empty())
}

fn has_content_directive(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| {
        matches!(prop, PropNode::Directive(directive) if matches!(directive.name, "text" | "html") && directive.exp.is_some())
    })
}

fn has_markup_content_directive(element: &MarkupElement<'_>) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= binding.kind() == MarkupBindingKind::Custom
            && (binding.is_unqualified_arg_exact("text")
                || binding.is_unqualified_arg_exact("html"))
            && binding.expression().is_some();
    });
    found
}

fn is_anchor_element(element: &ElementNode<'_>) -> bool {
    element.tag == "a"
        || (element.tag == "component"
            && get_static_or_bound_literal_attribute_value(element, "is") == Some("a"))
}

fn is_markup_anchor(element: &MarkupElement<'_>) -> bool {
    element.is_unqualified_tag_exact("a")
        || (element.is_unqualified_tag_exact("component")
            && markup_helpers::get_static_or_bound_literal_markup_value(element, "is") == Some("a"))
}

impl MarkupRule for AnchorHasContent {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !is_markup_anchor(element)
            || markup_helpers::is_aria_hidden_true_markup(element)
            || self.has_markup_content(element)
        {
            return;
        }
        let message = ctx.lint().t("a11y/anchor-has-content.message");
        let help = ctx.lint().t("a11y/anchor-has-content.help");
        ctx.lint().warn_at_with_help(message, element.range(), help);
    }
}

impl Rule for AnchorHasContent {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_anchor_element(element) || is_aria_hidden_true(element) {
            return;
        }

        if !self.has_accessible_content(element) {
            ctx.warn_with_help(
                ctx.t("a11y/anchor-has-content.message"),
                &element.loc,
                ctx.t("a11y/anchor-has-content.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AnchorHasContent;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AnchorHasContent));
        Linter::with_registry(registry)
    }

    #[test]
    fn configured_accessible_child_accepts_pascal_and_kebab_component_spellings() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AnchorHasContent::with_accessible_children(vec![
            "RetryImage".into(),
        ])));
        let linter = Linter::with_registry(registry);
        for source in [
            r#"<a href="/"><RetryImage /></a>"#,
            r#"<a href="/"><retry-image /></a>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, 0,
                "{source}: {:?}",
                result.diagnostics
            );
        }
        let result = linter.lint_template(r#"<a href="/"><OtherImage /></a>"#, "test.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_valid_with_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/">Home</a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/" aria-label="Home"></a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_empty() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/"></a>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_whitespace_only() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/">   </a>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_with_default_slot() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<a href="/"><slot></slot></a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_bound_img_alt() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<a :href="href"><img :src="src" :alt="alt" /></a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_empty_img_alt_does_not_name_anchor() {
        let linter = create_linter();
        for source in [
            r#"<a href="/"><img src="x.png" alt="" /></a>"#,
            r#"<a href="/"><img src="x.png" :alt="''" /></a>"#,
            r#"<a href="/"><img src="x.png" /></a>"#,
        ] {
            let result = linter.lint_template(source, "test.vue");
            assert_eq!(
                result.warning_count, 1,
                "{source}: {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn aligns_hidden_names_dynamic_content_and_component_anchors() {
        let result = create_linter().lint_template(
            r#"<a href="/a"><span aria-hidden="true">Settings</span></a>
<a href="/b" aria-label=""></a>
<component is="a" href="/c"></component>
<a href="/d"><span v-text="label"></span></a>
<a href="/e" aria-hidden="true"></a>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 3, "{:?}", result.diagnostics);
    }
}
