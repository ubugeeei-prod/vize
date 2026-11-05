//! a11y/alt-text
//!
//! Require alternative text for images and other media elements.
//!
//! Applies to: `<img>`, `<area>`, `<input type="image">`, `<object>`.
//! This is a more comprehensive version of `a11y/img-alt`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <img src="photo.jpg" />
//! <area href="/link" />
//! <input type="image" />
//! <object data="file.swf" />
//! ```
//!
//! ### Valid
//! ```vue
//! <img src="photo.jpg" alt="A sunset" />
//! <area href="/link" alt="Link description" />
//! <input type="image" alt="Submit" />
//! <object data="file.swf" title="Flash content" aria-label="Content" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{
    MarkupBindingKind, MarkupContext, MarkupElement, MarkupElementKind, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

use super::helpers::get_static_attribute_value;
use super::markup_helpers::get_static_markup_attribute_value;

static META: RuleMeta = RuleMeta {
    name: "a11y/alt-text",
    description: "Require alternative text for media elements",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require alternative text for media elements
#[derive(Default)]
pub struct AltText;

fn has_attribute_or_binding(element: &ElementNode, name: &str) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == name,
        PropNode::Directive(dir) => {
            if dir.name == "bind" {
                matches!(
                    &dir.arg,
                    Some(ExpressionNode::Simple(s)) if s.content == name
                )
            } else {
                false
            }
        }
    })
}

/// A static attribute, or a bind with a static or dynamic argument.
fn has_markup_attribute_or_binding(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut found = false;
    element.walk_bindings(&mut |binding| {
        found |= matches!(
            binding.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) && binding.is_unqualified_arg_exact(name);
    });
    found
}

fn has_markup_label(element: &MarkupElement<'_>) -> bool {
    has_markup_attribute_or_binding(element, "aria-label")
        || has_markup_attribute_or_binding(element, "aria-labelledby")
}

impl MarkupRule for AltText {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.kind() == MarkupElementKind::Component {
            return;
        }
        let keys = if element.is_unqualified_tag_exact("img") {
            (!has_markup_attribute_or_binding(element, "alt"))
                .then_some(("a11y/alt-text.message_img", "a11y/alt-text.help_img"))
        } else if element.is_unqualified_tag_exact("area") {
            (!has_markup_attribute_or_binding(element, "alt") && !has_markup_label(element))
                .then_some(("a11y/alt-text.message_area", "a11y/alt-text.help_area"))
        } else if element.is_unqualified_tag_exact("input") {
            (get_static_markup_attribute_value(element, "type") == Some("image")
                && !has_markup_attribute_or_binding(element, "alt")
                && !has_markup_label(element))
            .then_some((
                "a11y/alt-text.message_input_image",
                "a11y/alt-text.help_input_image",
            ))
        } else if element.is_unqualified_tag_exact("object") {
            (!has_markup_attribute_or_binding(element, "title") && !has_markup_label(element))
                .then_some(("a11y/alt-text.message_object", "a11y/alt-text.help_object"))
        } else {
            None
        };
        if let Some((message, help)) = keys {
            let message = ctx.lint().t(message);
            let help = ctx.lint().t(help);
            ctx.lint().warn_at_with_help(message, element.range(), help);
        }
    }
}

impl Rule for AltText {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }

        match element.tag {
            "img" if !has_attribute_or_binding(element, "alt") => {
                ctx.warn_with_help(
                    ctx.t("a11y/alt-text.message_img"),
                    &element.loc,
                    ctx.t("a11y/alt-text.help_img"),
                );
            }
            "area"
                if !has_attribute_or_binding(element, "alt")
                    && !has_attribute_or_binding(element, "aria-label")
                    && !has_attribute_or_binding(element, "aria-labelledby") =>
            {
                ctx.warn_with_help(
                    ctx.t("a11y/alt-text.message_area"),
                    &element.loc,
                    ctx.t("a11y/alt-text.help_area"),
                );
            }
            "input" => {
                let input_type = get_static_attribute_value(element, "type");
                if input_type == Some("image")
                    && !has_attribute_or_binding(element, "alt")
                    && !has_attribute_or_binding(element, "aria-label")
                    && !has_attribute_or_binding(element, "aria-labelledby")
                {
                    ctx.warn_with_help(
                        ctx.t("a11y/alt-text.message_input_image"),
                        &element.loc,
                        ctx.t("a11y/alt-text.help_input_image"),
                    );
                }
            }
            "object"
                if !has_attribute_or_binding(element, "title")
                    && !has_attribute_or_binding(element, "aria-label")
                    && !has_attribute_or_binding(element, "aria-labelledby") =>
            {
                ctx.warn_with_help(
                    ctx.t("a11y/alt-text.message_object"),
                    &element.loc,
                    ctx.t("a11y/alt-text.help_object"),
                );
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AltText;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AltText));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_img_with_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img src="photo.jpg" alt="Photo" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_img_with_dynamic_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img src="photo.jpg" :alt="desc" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_img_no_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img src="photo.jpg" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_area_with_alt() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<area href="/link" alt="Link description" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_area_no_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<area href="/link" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_input_image_with_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="image" alt="Submit" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_input_image_no_alt() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="image" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_input_text_no_alt() {
        let linter = create_linter();
        // Regular text inputs don't need alt
        let result = linter.lint_template(r#"<input type="text" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_object_with_title() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<object data="file.swf" title="Flash content"></object>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_object_no_title() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<object data="file.swf"></object>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_area_with_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<area href="/link" aria-label="Link description" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
