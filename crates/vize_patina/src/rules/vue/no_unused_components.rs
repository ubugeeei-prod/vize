//! vue/no-unused-components
//!
//! Disallow registering components that are not used inside templates.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! import MyButton from './MyButton.vue'  // imported but never used
//! </script>
//!
//! <template>
//!   <div>Hello</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! import MyButton from './MyButton.vue'
//! </script>
//!
//! <template>
//!   <MyButton>Click me</MyButton>
//! </template>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_croquis::naming::{is_pascal_case, to_pascal_case};
use vize_croquis::{Croquis, ScopeData};
use vize_relief::ast::RootNode;
use vize_relief::BindingType;

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-components",
    description: "Disallow registering components that are not used inside templates",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow unused components
#[derive(Default)]
pub struct NoUnusedComponents {
    /// Pattern for components to ignore (e.g., starts with '_')
    pub ignore_pattern: Option<String>,
}

impl NoUnusedComponents {
    /// Check if a component name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        // Ignore components starting with underscore
        if name.starts_with('_') {
            return true;
        }

        // Check custom ignore pattern
        if let Some(ref pattern) = self.ignore_pattern {
            if name.starts_with(pattern.as_str()) {
                return true;
            }
        }

        false
    }

    /// Check if an import source should be treated as a Vue component module.
    fn is_component_import_source(source: &str) -> bool {
        let path = source.split(['?', '#']).next().unwrap_or(source);
        path.ends_with(".vue")
    }

    /// Check if an imported binding type indicates a runtime component value.
    fn is_component_binding(binding_type: BindingType) -> bool {
        matches!(binding_type, BindingType::SetupConst)
    }

    fn imported_component_names(analysis: &Croquis) -> Vec<&str> {
        let mut names: Vec<_> = analysis
            .scopes
            .iter()
            .filter(|scope| {
                matches!(
                    scope.data(),
                    ScopeData::ExternalModule(data)
                        if !data.is_type_only
                            && Self::is_component_import_source(data.source.as_str())
                )
            })
            .flat_map(|scope| {
                scope.bindings().filter_map(|(name, binding)| {
                    if Self::is_component_binding(binding.binding_type) && is_pascal_case(name) {
                        Some(name)
                    } else {
                        None
                    }
                })
            })
            .collect();

        names.sort_unstable();
        names.dedup();
        names
    }

    fn component_name_matches(used: &str, registered: &str) -> bool {
        used == registered
            || vize_croquis::naming::names_match(used, registered)
            || to_pascal_case(used).as_str() == registered
    }

    fn matches_registered_alias(analysis: &Croquis, used: &str, local_name: &str) -> bool {
        analysis
            .component_registrations
            .iter()
            .filter(|registration| registration.local_name == local_name)
            .any(|registration| Self::component_name_matches(used, registration.name.as_str()))
    }
}

impl Rule for NoUnusedComponents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            return;
        }

        // Collect unused components first (to avoid borrow conflicts)
        let unused_components: Vec<String> = {
            let analysis = ctx.analysis().unwrap();

            let registered_components = Self::imported_component_names(analysis);

            // Find unused components
            registered_components
                .into_iter()
                .filter(|name| {
                    if self.should_ignore(name) {
                        return false;
                    }

                    // Check if used in template (case-insensitive matching for kebab-case)
                    !analysis.used_components.iter().any(|used| {
                        Self::component_name_matches(used.as_str(), name)
                            || Self::matches_registered_alias(analysis, used.as_str(), name)
                    })
                })
                .map(|name| name.to_compact_string())
                .collect()
        };

        // Report unused components
        for name in unused_components {
            ctx.report(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    format!(
                        "Component '{}' is registered but never used in template",
                        name
                    ),
                    0,
                    name.len() as u32,
                )
                .with_help("Remove the unused import or use the component in your template"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoUnusedComponents;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoUnusedComponents::default();
        assert_eq!(rule.meta().name, "vue/no-unused-components");
        assert_eq!(rule.meta().category, RuleCategory::Essential);
    }

    #[test]
    fn test_should_ignore() {
        let rule = NoUnusedComponents::default();
        assert!(rule.should_ignore("_Internal"));
        assert!(!rule.should_ignore("MyComponent"));
    }
}
