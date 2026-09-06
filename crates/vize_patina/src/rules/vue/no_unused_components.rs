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

mod dynamic_is;
mod script_setup_refs;
#[cfg(test)]
mod tests;

use self::dynamic_is::has_dynamic_is_binding;
use self::script_setup_refs::script_setup_component_import_references;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::naming::{is_pascal_case, to_pascal_case};
use vize_croquis::{Croquis, Scope, ScopeData, ScopeKind};
use vize_relief::BindingType;
use vize_relief::RootNode;
use vize_s0::{CompactString, String, ToCompactString};

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

#[derive(Debug, Clone)]
struct ComponentCandidate {
    name: CompactString,
    local_name: CompactString,
    is_script_setup_import: bool,
}

impl NoUnusedComponents {
    /// Check if a component name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        // Ignore components starting with underscore
        if name.starts_with('_') {
            return true;
        }

        // Check custom ignore pattern
        if let Some(ref pattern) = self.ignore_pattern
            && name.starts_with(pattern.as_str())
        {
            return true;
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

    fn component_candidates(analysis: &Croquis) -> Vec<ComponentCandidate> {
        let mut candidates = Vec::new();

        for scope in analysis
            .scopes
            .iter()
            .filter(|scope| Self::is_script_setup_component_import(analysis, scope))
        {
            for (name, binding) in scope.bindings() {
                if Self::is_component_binding(binding.binding_type) && is_pascal_case(name) {
                    push_component_candidate(&mut candidates, name, name, true);
                }
            }
        }

        for registration in &analysis.component_registrations {
            push_component_candidate(
                &mut candidates,
                registration.name.as_str(),
                registration.local_name.as_str(),
                false,
            );
        }

        candidates.sort_unstable_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
        candidates
    }

    fn is_script_setup_component_import(analysis: &Croquis, scope: &Scope) -> bool {
        let ScopeData::ExternalModule(data) = scope.data() else {
            return false;
        };
        if data.is_type_only || !Self::is_component_import_source(data.source.as_str()) {
            return false;
        }
        scope
            .parent()
            .and_then(|id| analysis.scopes.get_scope(id))
            .is_some_and(|parent| parent.kind == ScopeKind::ScriptSetup)
    }

    fn component_name_matches(used: &str, registered: &str) -> bool {
        used == registered
            || vize_croquis::naming::names_match(used, registered)
            || to_pascal_case(used).as_str() == registered
    }
}

impl Rule for NoUnusedComponents {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            return;
        }
        if has_dynamic_is_binding(&root.children) {
            return;
        }

        // Collect template-unused components first (to avoid borrow conflicts)
        let (template_unused_components, import_statement_ranges): (
            Vec<ComponentCandidate>,
            Vec<(u32, u32)>,
        ) = {
            let Some(analysis) = ctx.analysis() else {
                return;
            };

            let registered_components = Self::component_candidates(analysis);

            let import_statement_ranges = analysis
                .import_statements
                .iter()
                .map(|import| (import.start, import.end))
                .collect();

            let template_unused_components = registered_components
                .into_iter()
                .filter(|component| {
                    let name = component.name.as_str();
                    if self.should_ignore(name) {
                        return false;
                    }

                    // Check if used in template (case-insensitive matching for kebab-case)
                    !analysis
                        .used_components
                        .iter()
                        .any(|used| Self::component_name_matches(used.as_str(), name))
                })
                .collect();

            (template_unused_components, import_statement_ranges)
        };

        let script_setup_candidate_names: Vec<_> = template_unused_components
            .iter()
            .filter(|component| component.is_script_setup_import)
            .map(|component| component.local_name.clone())
            .collect();
        let script_used_components = script_setup_component_import_references(
            ctx,
            &script_setup_candidate_names,
            &import_statement_ranges,
        );
        let unused_components: Vec<_> = template_unused_components
            .into_iter()
            .filter(|component| {
                !component.is_script_setup_import
                    || !script_used_components.contains(component.local_name.as_str())
            })
            .collect();

        // Report unused components
        for component in unused_components {
            let name = component.name.as_str();
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

fn push_component_candidate(
    candidates: &mut Vec<ComponentCandidate>,
    name: &str,
    local_name: &str,
    is_script_setup_import: bool,
) {
    if candidates
        .iter()
        .any(|candidate| candidate.name.as_str() == name)
    {
        return;
    }
    candidates.push(ComponentCandidate {
        name: name.to_compact_string(),
        local_name: local_name.to_compact_string(),
        is_script_setup_import,
    });
}
