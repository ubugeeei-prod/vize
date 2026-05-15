//! vue/require-component-registration
//!
//! Warn when using components that are not explicitly imported or registered.
//!
//! In Vue SFCs, components should be either:
//! - Imported in `<script setup>` (auto-registered)
//! - Registered via `components` option
//! - Global components registered via `app.component()`
//!
//! This rule helps catch typos and missing imports early.
//!
//! ## Configuration
//!
//! This rule can be configured to ignore certain global components:
//! - Built-in components: component, transition, keep-alive, etc.
//! - Common global components from frameworks like Nuxt
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <template>
//!   <MyButton>Click</MyButton> <!-- Not imported -->
//! </template>
//!
//! <script setup>
//! // MyButton is not imported
//! </script>
//! ```
//!
//! Good:
//! ```vue
//! <template>
//!   <MyButton>Click</MyButton>
//! </template>
//!
//! <script setup>
//! import MyButton from './MyButton.vue'
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_croquis::builtins::is_builtin_component;
use vize_relief::ast::{ElementNode, RootNode};

static META: RuleMeta = RuleMeta {
    name: "vue/require-component-registration",
    description: "Require explicit import or registration for components",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Components commonly provided by frameworks (Nuxt, etc.)
const FRAMEWORK_GLOBALS: &[&str] = &[
    // Nuxt components
    "nuxt-link",
    "nuxt",
    "nuxt-child",
    "nuxt-page",
    "client-only",
    "nuxt-loading-indicator",
    "nuxt-layout",
    "nuxt-error-boundary",
    // Vue Router
    "router-link",
    "router-view",
];

/// Require component registration rule
#[derive(Default)]
pub struct RequireComponentRegistration {
    /// Additional global components to ignore
    pub ignore_globals: Vec<String>,
    /// Whether to check Nuxt auto-imports
    pub nuxt_mode: bool,
}

impl RequireComponentRegistration {
    /// Create rule with Nuxt mode enabled
    pub fn nuxt() -> Self {
        Self {
            ignore_globals: Vec::new(),
            nuxt_mode: true,
        }
    }

    /// Check if a tag name is a custom component
    fn is_custom_component(&self, tag: &str) -> bool {
        // HTML elements are lowercase only
        // Custom components have uppercase or contain dash
        let first_char = tag.chars().next().unwrap_or('a');

        // PascalCase component
        if first_char.is_uppercase() {
            return true;
        }

        // kebab-case component with dash (but not HTML like <my-element>)
        // Actually, kebab-case with dash could be custom element or component
        // We'll be conservative and check if it looks like a component
        if tag.contains('-') {
            // Check against known HTML custom elements patterns
            // Most custom elements start with known prefixes
            let is_web_component = tag.starts_with("x-")
                || tag.starts_with("ion-")
                || tag.starts_with("md-")
                || tag.starts_with("mwc-");

            return !is_web_component;
        }

        false
    }

    /// Check if a component is a Vue built-in
    /// Uses croquis builtins for centralized builtin detection
    fn is_builtin(&self, tag: &str) -> bool {
        // Check exact match first (handles PascalCase like "Transition")
        if is_builtin_component(tag) {
            return true;
        }
        // Check lowercase (handles kebab-case like "keep-alive")
        let lower = tag.to_lowercase();
        is_builtin_component(&lower)
    }

    /// Check if a component is a framework global
    fn is_framework_global(&self, tag: &str) -> bool {
        let lower = tag.to_lowercase();
        // Convert PascalCase to kebab-case for comparison
        let kebab = pascal_to_kebab(tag);

        FRAMEWORK_GLOBALS.contains(&lower.as_str())
            || FRAMEWORK_GLOBALS.contains(&kebab.as_str())
            || self
                .ignore_globals
                .iter()
                .any(|g| g.eq_ignore_ascii_case(tag) || g.eq_ignore_ascii_case(&kebab))
    }
}

impl Rule for RequireComponentRegistration {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Collect all custom components used in template
        let mut used_components: Vec<(String, u32, u32)> = Vec::new();
        collect_components(root, &mut used_components);

        // For now, we warn on all custom components that aren't built-in or framework globals
        // A full implementation would parse the script block to find imports
        for (tag, start, end) in used_components {
            if self.is_custom_component(&tag)
                && !self.is_builtin(&tag)
                && !self.is_framework_global(&tag)
            {
                // In Nuxt mode, don't warn as components are auto-imported
                if self.nuxt_mode {
                    continue;
                }

                ctx.report(
                    LintDiagnostic::warn(
                        META.name,
                        "Component is used but not explicitly imported",
                        start,
                        end,
                    )
                    .with_help("Import the component in <script setup> or register it in components option"),
                );
            }
        }
    }
}

/// Collect all element tags from the template
fn collect_components<'a>(root: &RootNode<'a>, result: &mut Vec<(String, u32, u32)>) {
    fn visit_element<'a>(element: &ElementNode<'a>, result: &mut Vec<(String, u32, u32)>) {
        let start = element.loc.start.offset;
        let tag_str = element.tag.as_str();
        result.push((
            tag_str.to_compact_string(),
            start,
            start + tag_str.len() as u32,
        ));

        for child in element.children.iter() {
            if let vize_relief::ast::TemplateChildNode::Element(el) = child {
                visit_element(el, result);
            }
        }
    }

    for child in root.children.iter() {
        if let vize_relief::ast::TemplateChildNode::Element(el) = child {
            visit_element(el, result);
        }
    }
}

/// Convert PascalCase to kebab-case
fn pascal_to_kebab(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{pascal_to_kebab, RequireComponentRegistration};

    #[test]
    fn test_pascal_to_kebab() {
        assert_eq!(pascal_to_kebab("MyButton"), "my-button");
        assert_eq!(pascal_to_kebab("NuxtLink"), "nuxt-link");
        assert_eq!(pascal_to_kebab("RouterView"), "router-view");
    }

    #[test]
    fn test_is_custom_component() {
        let rule = RequireComponentRegistration::default();
        assert!(rule.is_custom_component("MyButton"));
        assert!(rule.is_custom_component("my-button"));
        assert!(!rule.is_custom_component("div"));
        assert!(!rule.is_custom_component("span"));
    }

    #[test]
    fn test_is_builtin() {
        let rule = RequireComponentRegistration::default();
        assert!(rule.is_builtin("component"));
        assert!(rule.is_builtin("Transition"));
        assert!(rule.is_builtin("keep-alive"));
        assert!(!rule.is_builtin("MyButton"));
    }
}
