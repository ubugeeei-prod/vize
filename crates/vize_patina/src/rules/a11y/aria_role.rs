//! a11y/aria-role
//!
//! Elements with ARIA roles must use a valid, non-abstract ARIA role.
//!
//! This rule enforces that `role` attributes contain only valid ARIA roles
//! from the WAI-ARIA specification. Abstract roles are not allowed as they
//! are meant to be used as base concepts, not directly on elements.
//!
//! Incorrect roles can prevent assistive technologies from conveying the
//! intended meaning to users.
//!
//! Based on eslint-plugin-jsx-a11y aria-role rule.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, PropNode, SourceLocation};

static META: RuleMeta = RuleMeta {
    name: "a11y/aria-role",
    description: "Elements with ARIA roles must use a valid, non-abstract ARIA role",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Error,
};

/// Valid ARIA roles from WAI-ARIA specification.
/// https://www.w3.org/TR/wai-aria/#role_definitions
///
/// This list includes:
/// - Widget roles
/// - Composite widget roles
/// - Document structure roles
/// - Landmark roles
/// - Live region roles
/// - Window roles
const VALID_ARIA_ROLES: &[&str] = &[
    // === Widget Roles ===
    // https://www.w3.org/TR/wai-aria/#widget_roles
    "alert",
    "alertdialog",
    "button",
    "checkbox",
    "dialog",
    "gridcell",
    "link",
    "log",
    "marquee",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "option",
    "progressbar",
    "radio",
    "scrollbar",
    "searchbox",
    "slider",
    "spinbutton",
    "status",
    "switch",
    "tab",
    "tabpanel",
    "textbox",
    "timer",
    "tooltip",
    "treeitem",
    // === Composite Widget Roles ===
    // https://www.w3.org/TR/wai-aria/#composite_roles
    "combobox",
    "grid",
    "listbox",
    "menu",
    "menubar",
    "radiogroup",
    "tablist",
    "tree",
    "treegrid",
    // === Document Structure Roles ===
    // https://www.w3.org/TR/wai-aria/#document_structure_roles
    "application",
    "article",
    "blockquote",
    "caption",
    "cell",
    "code",
    "columnheader",
    "definition",
    "deletion",
    "directory", // Deprecated but still valid
    "document",
    "emphasis",
    "feed",
    "figure",
    "generic",
    "group",
    "heading",
    "img",
    "insertion",
    "list",
    "listitem",
    "math",
    "meter",
    "none",
    "note",
    "paragraph",
    "presentation",
    "row",
    "rowgroup",
    "rowheader",
    "separator",
    "strong",
    "subscript",
    "superscript",
    "table",
    "term",
    "time",
    "toolbar",
    // === Landmark Roles ===
    // https://www.w3.org/TR/wai-aria/#landmark_roles
    "banner",
    "complementary",
    "contentinfo",
    "form",
    "main",
    "navigation",
    "region",
    "search",
    // === Live Region Roles ===
    // (alert, log, marquee, status, timer already listed above)
    // === Window Roles ===
    // (alertdialog, dialog already listed above)
    // === ARIA 1.3 Additions ===
    "comment",
    "mark",
    "suggestion",
];

/// Abstract ARIA roles that should NOT be used directly.
/// https://www.w3.org/TR/wai-aria/#abstract_roles
///
/// These are base concepts that other roles inherit from.
/// They must not be used by web authors.
const ABSTRACT_ARIA_ROLES: &[&str] = &[
    "command",
    "composite",
    "input",
    "landmark",
    "range",
    "roletype",
    "section",
    "sectionhead",
    "select",
    "structure",
    "widget",
    "window",
];

/// Elements with ARIA roles must use a valid, non-abstract ARIA role
#[derive(Default)]
pub struct AriaRole {
    /// Whether to ignore non-DOM elements (custom components)
    pub ignore_non_dom: bool,
}

impl AriaRole {
    /// Check if a role is a valid ARIA role
    #[inline]
    fn is_valid_role(role: &str) -> bool {
        VALID_ARIA_ROLES.contains(&role)
    }

    /// Check if a role is an abstract ARIA role
    #[inline]
    fn is_abstract_role(role: &str) -> bool {
        ABSTRACT_ARIA_ROLES.contains(&role)
    }

    /// Check if an element is a DOM element (not a custom component)
    #[inline]
    fn is_dom_element(tag: &str) -> bool {
        // Custom components typically start with uppercase or contain hyphens
        // DOM elements are lowercase without hyphens (except for custom elements)
        let first_char = tag.chars().next().unwrap_or('a');
        first_char.is_ascii_lowercase() && !tag.contains('-')
    }

    /// Find a similar valid role for suggestions
    fn find_similar(invalid: &str) -> Option<&'static str> {
        // Common mistakes mapping
        let typo_fixes: &[(&str, &str)] = &[
            ("date", "textbox"),
            ("datepicker", "textbox"),
            ("dropdown", "listbox"),
            ("input", "textbox"),
            ("item", "listitem"),
            ("listitem", "listitem"), // Already valid but catch typos
            ("modal", "dialog"),
            ("popup", "dialog"),
            ("text", "textbox"),
            ("titlebar", "heading"),
        ];

        // Check direct typo mapping first
        let invalid_lower = invalid.to_ascii_lowercase();
        for (typo, fix) in typo_fixes {
            if *typo == invalid_lower {
                return Some(fix);
            }
        }

        // Levenshtein-like similarity check for other cases
        VALID_ARIA_ROLES
            .iter()
            .find(|valid| {
                let valid_lower = valid.to_ascii_lowercase();
                // Simple similarity: same length +/- 2 and most chars match
                let len_diff = (invalid_lower.len() as i32 - valid_lower.len() as i32).abs();
                if len_diff > 2 {
                    return false;
                }
                // Count matching chars
                let matches = invalid_lower
                    .chars()
                    .zip(valid_lower.chars())
                    .filter(|(a, b)| a == b)
                    .count();
                matches >= invalid_lower.len().saturating_sub(2)
            })
            .copied()
    }
}

impl Rule for AriaRole {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // Skip non-DOM elements if configured
        if self.ignore_non_dom && !Self::is_dom_element(element.tag.as_str()) {
            return;
        }

        for prop in &element.props {
            match prop {
                PropNode::Attribute(attr) => {
                    if attr.name.as_str() == "role" {
                        if let Some(value) = &attr.value {
                            self.check_role(ctx, value.content.as_str(), &attr.loc);
                        }
                    }
                }
                PropNode::Directive(dir) => {
                    // Check :role or v-bind:role with static value
                    if dir.name == "bind" {
                        if let Some(vize_relief::ast::ExpressionNode::Simple(arg)) = &dir.arg {
                            if arg.content.as_str() == "role" {
                                // For dynamic roles, we can only check if it's a static string
                                if let Some(vize_relief::ast::ExpressionNode::Simple(expr)) =
                                    &dir.exp
                                {
                                    let content = expr.content.as_str().trim();
                                    // Check if it's a string literal like "'button'" or "\"button\""
                                    if (content.starts_with('\'') && content.ends_with('\''))
                                        || (content.starts_with('"') && content.ends_with('"'))
                                    {
                                        let role = &content[1..content.len() - 1];
                                        self.check_role(ctx, role, &dir.loc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl AriaRole {
    fn check_role(&self, ctx: &mut LintContext<'_>, role: &str, loc: &SourceLocation) {
        // Handle multiple roles (space-separated)
        for single_role in role.split_whitespace() {
            let role_lower = single_role.to_ascii_lowercase();

            // Check for abstract role
            if Self::is_abstract_role(&role_lower) {
                let message =
                    ctx.t_fmt("a11y/aria-role.message_abstract", &[("role", single_role)]);
                ctx.error_with_help(message, loc, ctx.t("a11y/aria-role.help_abstract"));
                continue;
            }

            // Check for invalid role
            if !Self::is_valid_role(&role_lower) {
                let message = ctx.t_fmt("a11y/aria-role.message", &[("role", single_role)]);

                if let Some(suggestion) = Self::find_similar(&role_lower) {
                    let help = ctx.t_fmt(
                        "a11y/aria-role.help_suggestion",
                        &[("invalid", single_role), ("valid", suggestion)],
                    );
                    ctx.error_with_help(message.clone(), loc, help);
                } else {
                    ctx.error_with_help(message, loc, ctx.t("a11y/aria-role.help"));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AriaRole;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AriaRole::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_button_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="button"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_navigation_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<nav role="navigation"></nav>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_dialog_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="dialog"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_multiple_roles() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="img presentation"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_dynamic_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :role="role"></div>"#, "test.vue");
        // Dynamic roles with variable values are not checked
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="test"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_datepicker_role() {
        let linter = create_linter();
        // "datepicker" is not an ARIA role
        let result = linter.lint_template(r#"<div role="datepicker"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_abstract_range_role() {
        let linter = create_linter();
        // "range" is an abstract ARIA role
        let result = linter.lint_template(r#"<div role="range"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_abstract_input_role() {
        let linter = create_linter();
        // "input" is an abstract ARIA role
        let result = linter.lint_template(r#"<div role="input"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_abstract_widget_role() {
        let linter = create_linter();
        // "widget" is an abstract ARIA role
        let result = linter.lint_template(r#"<div role="widget"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_made_up_role() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div role="foobar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_is_valid_role() {
        assert!(AriaRole::is_valid_role("button"));
        assert!(AriaRole::is_valid_role("dialog"));
        assert!(AriaRole::is_valid_role("navigation"));
        assert!(!AriaRole::is_valid_role("datepicker"));
        assert!(!AriaRole::is_valid_role("range")); // Abstract role
    }

    #[test]
    fn test_is_abstract_role() {
        assert!(AriaRole::is_abstract_role("range"));
        assert!(AriaRole::is_abstract_role("widget"));
        assert!(AriaRole::is_abstract_role("composite"));
        assert!(!AriaRole::is_abstract_role("button"));
        assert!(!AriaRole::is_abstract_role("dialog"));
    }

    #[test]
    fn test_is_dom_element() {
        assert!(AriaRole::is_dom_element("div"));
        assert!(AriaRole::is_dom_element("button"));
        assert!(AriaRole::is_dom_element("nav"));
        assert!(!AriaRole::is_dom_element("MyComponent"));
        assert!(!AriaRole::is_dom_element("custom-element"));
    }

    #[test]
    fn test_find_similar() {
        assert_eq!(AriaRole::find_similar("modal"), Some("dialog"));
        assert_eq!(AriaRole::find_similar("datepicker"), Some("textbox"));
        assert_eq!(AriaRole::find_similar("dropdown"), Some("listbox"));
    }
}
