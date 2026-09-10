//! a11y/interactive-supports-focus
//!
//! Require elements with interactive roles to be focusable.
//!
//! Elements that have interactive ARIA roles (button, link, etc.) must
//! be focusable either natively or via tabindex.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div role="button" @click="handle">Click</div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div role="button" tabindex="0" @click="handle">Click</div>
//! <button @click="handle">Click</button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, RootNode, TemplateChildNode};
use vize_s0::FxHashSet;

use super::{
    helpers::{
        get_static_or_bound_literal_attribute_value, has_named_attribute_or_bind,
        is_interactive_role,
    },
    markup_helpers,
};

static META: RuleMeta = RuleMeta {
    name: "a11y/interactive-supports-focus",
    description: "Require interactive role elements to be focusable",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require interactive role elements to be focusable
#[derive(Default)]
pub struct InteractiveSupportsFocus;

impl InteractiveSupportsFocus {
    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        controlled_listboxes: Option<&FxHashSet<&str>>,
    ) {
        if element.is_component() {
            return;
        }

        // Skip natively interactive elements - they're already focusable
        if markup_helpers::is_interactive_markup_element(element) {
            return;
        }

        // Check if element has an interactive role
        let role = match markup_helpers::get_static_markup_attribute_value(element, "role") {
            Some(r) => r,
            None => return,
        };

        if !is_interactive_role(role) {
            return;
        }

        if role == "listbox"
            && let Some(controlled_listboxes) = controlled_listboxes
            && let Some(id) = markup_helpers::get_static_markup_attribute_value(element, "id")
            && controlled_listboxes.contains(id)
        {
            return;
        }

        // Element has interactive role but is not natively interactive
        // Check if it's focusable
        if !markup_helpers::is_focusable_markup_element(element) {
            ctx.warn_at_with_help(
                ctx.t_fmt("a11y/interactive-supports-focus.message", &[("role", role)]),
                element.range(),
                ctx.t("a11y/interactive-supports-focus.help"),
            );
        }
    }

    fn collect_controlled_listboxes<'a>(root: &'a RootNode<'a>) -> FxHashSet<&'a str> {
        let mut controlled = FxHashSet::default();
        walk_template_elements(root, &mut |element| {
            if has_named_attribute_or_bind(element, "aria-activedescendant")
                && let Some(id) =
                    get_static_or_bound_literal_attribute_value(element, "aria-controls")
            {
                controlled.insert(id);
            }
        });
        controlled
    }
}

fn walk_template_elements<'a>(
    root: &'a RootNode<'a>,
    visitor: &mut impl FnMut(&'a ElementNode<'a>),
) {
    for child in &root.children {
        walk_template_child(child, visitor);
    }
}

fn walk_template_child<'a>(
    child: &'a TemplateChildNode<'a>,
    visitor: &mut impl FnMut(&'a ElementNode<'a>),
) {
    if let TemplateChildNode::Element(element) = child {
        visitor(element);
        for child in &element.children {
            walk_template_child(child, visitor);
        }
    }
}

impl MarkupRule for InteractiveSupportsFocus {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element, None);
    }
}

impl Rule for InteractiveSupportsFocus {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let controlled_listboxes = Self::collect_controlled_listboxes(root);
        walk_template_elements(root, &mut |element| {
            Self::check_element(
                ctx,
                &MarkupElement::new(element),
                Some(&controlled_listboxes),
            );
        });
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let _ = (ctx, element);
    }
}

#[cfg(test)]
mod tests {
    use super::InteractiveSupportsFocus;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(InteractiveSupportsFocus));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_native_button() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button @click="handle">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_div_role_button_with_tabindex() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" tabindex="0" @click="handle">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_div_role_button_no_tabindex() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div role="button" @click="handle">Click</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_non_interactive_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="presentation">Content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_span_role_link() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<span role="link">Link</span>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_activedescendant_listbox() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div>
  <button
    type="button"
    role="combobox"
    aria-controls="lb"
    :aria-activedescendant="`lb-option-${focused}`"
  >Select</button>
  <div id="lb" role="listbox">
    <button role="option" tabindex="-1">A</button>
  </div>
</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<MyButton role="button">Click</MyButton>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
