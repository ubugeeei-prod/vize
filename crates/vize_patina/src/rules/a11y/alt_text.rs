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
use crate::markup::{MarkupContext, MarkupElement, MarkupElementKind, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use super::markup_helpers::get_static_or_bound_literal_markup_value;
use super::reference_values::{has_accessible_child, has_aria_label, has_attribute_text};

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

fn lint(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
    if element.kind() == MarkupElementKind::Component {
        return;
    }
    let keys = if element.is_unqualified_tag_exact("img") {
        (!has_attribute_text(element, "alt", true))
            .then_some(("a11y/alt-text.message_img", "a11y/alt-text.help_img"))
    } else if element.is_unqualified_tag_exact("area") {
        (!has_attribute_text(element, "alt", false) && !has_aria_label(element))
            .then_some(("a11y/alt-text.message_area", "a11y/alt-text.help_area"))
    } else if element.is_unqualified_tag_exact("input") {
        (get_static_or_bound_literal_markup_value(element, "type") == Some("image")
            && !has_attribute_text(element, "alt", false)
            && !has_aria_label(element))
        .then_some((
            "a11y/alt-text.message_input_image",
            "a11y/alt-text.help_input_image",
        ))
    } else if element.is_unqualified_tag_exact("object") {
        (!has_attribute_text(element, "title", false)
            && !has_aria_label(element)
            && !has_accessible_child(ctx.source, element))
        .then_some(("a11y/alt-text.message_object", "a11y/alt-text.help_object"))
    } else {
        None
    };
    if let Some((message, help)) = keys {
        let message = ctx.t(message);
        let help = ctx.t(help);
        ctx.warn_at_with_help(message, element.range(), help);
    }
}

impl MarkupRule for AltText {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        lint(ctx.lint(), element);
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
        lint(ctx, &MarkupElement::new(element));
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

    #[test]
    fn matches_eslint_plugin_vuejs_accessibility_text_boundaries() {
        let result = create_linter().lint_template(
            r#"<div>
                <img src="/logo.png" alt />
                <img src="/logo.png" :alt="null" />
                <object data="/report.pdf" type="application/pdf">Report</object>
                <object data="/report.pdf" type="application/pdf"><p>Report</p></object>
                <object data="/report.pdf" type="application/pdf">{{ label }}</object>
                <object data="/report.pdf" title=""></object>
                <map name="m"><area href="/a" alt="" /></map>
                <map name="m"><area href="/a" aria-label="" /></map>
                <input type="image" src="/go.png" alt="" />
                <input type="image" src="/go.png" alt />
                <input :type="'image'" src="/go.png" />
            </div>"#,
            "test.vue",
        );

        assert_eq!(result.warning_count, 8, "{:?}", result.diagnostics);
    }

    #[test]
    fn reads_bound_literals_and_hidden_children_like_the_reference() {
        let result = create_linter().lint_template(
            r#"<div>
                <img src="/a.png" :alt="false" />
                <img src="/a.png" :alt="(0)" />
                <img src="/a.png" :alt="undefined" />
                <object data="/a.pdf"><span aria-hidden="true">Report</span></object>
                <object data="/a.pdf">{{ undefined }}</object>
                <img src="/a.png" :alt="''" />
                <object data="/a.pdf"><slot /></object>
                <object data="/a.pdf" title=" "></object>
                <map name="m"><area href="/a" alt=" " /></map>
                <object data="/a.pdf"><span :aria-hidden="false">Report</span></object>
            </div>"#,
            "test.vue",
        );

        assert_eq!(result.warning_count, 7, "{:?}", result.diagnostics);
    }
}
