//! vue/no-root-v-if
//!
//! Disallow `v-if` on the single root element of a component template.
//!
//! When a template has exactly one root element and that element carries a
//! `v-if`, the whole component can render nothing when the condition is false.
//! That is rarely the intent and is easy to miss in review. Wrapping the
//! conditional content in an always-present root element, or using `v-show`
//! instead, makes the rendering behavior explicit.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <div v-if="show">content</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <div>
//!     <p v-if="show">content</p>
//!   </div>
//! </template>
//! ```
//!
//! ```vue
//! <template>
//!   <div v-show="show">content</div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode, RootNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-root-v-if",
    description: "Disallow v-if on the single root element of a template",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow `v-if` on the single root element of a template.
pub struct NoRootVIf;

impl NoRootVIf {
    /// A text node that is only whitespace is layout noise between root
    /// elements and is ignored when deciding how many roots a template has.
    fn is_ignorable(child: &TemplateChildNode) -> bool {
        match child {
            TemplateChildNode::Comment(_) => true,
            TemplateChildNode::Text(text) => text.content.trim().is_empty(),
            _ => false,
        }
    }

    /// Return the `v-if` directive location on this element, if present.
    fn v_if_location<'e>(element: &'e ElementNode<'_>) -> Option<&'e vize_relief::SourceLocation> {
        for prop in &element.props {
            if let PropNode::Directive(dir) = prop
                && dir.name.as_str() == "if"
            {
                return Some(&dir.loc);
            }
        }
        None
    }
}

impl Rule for NoRootVIf {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Collect the element children, skipping whitespace and comment nodes.
        // If anything other than a single element node remains (multiple roots,
        // or non-element content), the "single root" condition does not hold.
        let mut sole_element: Option<&ElementNode<'a>> = None;

        for child in &root.children {
            if Self::is_ignorable(child) {
                continue;
            }

            match child {
                TemplateChildNode::Element(element) => {
                    if sole_element.is_some() {
                        // More than one root element: not a single-root template.
                        return;
                    }
                    sole_element = Some(element);
                }
                // Meaningful non-element content (text, interpolation, ...) at
                // the root means the root is not a lone element; bail out.
                _ => return,
            }
        }

        let Some(element) = sole_element else {
            return;
        };

        if let Some(loc) = Self::v_if_location(element) {
            ctx.warn_with_help(
                ctx.t("vue/no-root-v-if.message"),
                loc,
                ctx.t("vue/no-root-v-if.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoRootVIf;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoRootVIf));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_invalid_single_root_with_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="show">content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].rule_name, "vue/no-root-v-if");
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_single_root_with_v_if_surrounded_by_whitespace() {
        let linter = create_linter();
        let result = linter.lint_template("\n  <div v-if=\"show\">content</div>\n  ", "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_single_root_with_v_if_and_comment() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<!-- header --><div v-if="show">content</div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_root_without_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_nested_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><p v-if="show">content</p></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_root_with_v_show() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-show="show">content</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_multiple_roots_with_v_if_v_else() {
        let linter = create_linter();
        // Multiple root elements: v-if pairs with v-else, so the component
        // always renders something. This rule should not fire.
        let result =
            linter.lint_template(r#"<div v-if="show">a</div><div v-else>b</div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_multiple_roots() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<header>h</header><main v-if="show">m</main>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }
}
