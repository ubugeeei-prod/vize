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

use self::self_name::{define_options_name, file_stem};
use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::builtins::is_builtin_component;
use vize_croquis::naming::{names_match, to_pascal_case};
use vize_croquis::{Croquis, ScopeData};
use vize_relief::BindingType;
use vize_relief::{ElementNode, RootNode};
use vize_s0::String;
use vize_s0::ToCompactString;

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

    /// Whether `tag` refers to the component itself.
    ///
    /// A `<script setup>` component can reference itself by its
    /// filename-derived name or by the name declared via
    /// `defineOptions({ name })` — a documented Vue feature for recursive
    /// components that needs no import (importing yourself is not possible
    /// without an alias).
    fn is_self_reference(&self, ctx: &LintContext<'_>, tag: &str) -> bool {
        let stem = file_stem(ctx.filename);
        if !stem.is_empty() {
            let self_name = to_pascal_case(stem);
            if component_name_matches(tag, self_name.as_str()) {
                return true;
            }
        }
        ctx.analysis()
            .and_then(define_options_name)
            .is_some_and(|name| component_name_matches(tag, name))
    }

    fn is_script_setup_imported_component(&self, analysis: &Croquis, tag: &str) -> bool {
        analysis
            .scopes
            .iter()
            .filter(|scope| {
                matches!(
                    scope.data(),
                    ScopeData::ExternalModule(data) if !data.is_type_only
                )
            })
            .flat_map(|scope| scope.bindings())
            .any(|(name, binding)| {
                matches!(
                    binding.binding_type,
                    BindingType::SetupConst | BindingType::SetupMaybeRef
                ) && component_name_matches(tag, name)
            })
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
        for (tag, start, end) in used_components {
            if self.is_custom_component(&tag)
                && !self.is_builtin(&tag)
                && !self.is_framework_global(&tag)
            {
                // Recursive self-reference needs no registration (#4953).
                if self.is_self_reference(ctx, &tag) {
                    continue;
                }

                if ctx
                    .analysis()
                    .is_some_and(|analysis| self.is_script_setup_imported_component(analysis, &tag))
                {
                    continue;
                }

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
        let start = element.loc.span.start;
        let tag_str = element.tag;
        result.push((
            tag_str.to_compact_string(),
            start,
            start + tag_str.len() as u32,
        ));

        for child in element.children.iter() {
            if let vize_relief::TemplateChildNode::Element(el) = child {
                visit_element(el, result);
            }
        }
    }

    for child in root.children.iter() {
        if let vize_relief::TemplateChildNode::Element(el) = child {
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

fn component_name_matches(used: &str, registered: &str) -> bool {
    used == registered
        || names_match(used, registered)
        || to_pascal_case(used).as_str() == registered
}

mod self_name;

#[cfg(test)]
mod tests;
