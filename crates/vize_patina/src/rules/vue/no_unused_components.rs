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
use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, ImportDeclaration, ImportDeclarationSpecifier, TSType};
use oxc_ast_visit::{Visit, walk::walk_ts_type};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, ToCompactString};
use vize_croquis::naming::{is_pascal_case, to_pascal_case};
use vize_croquis::{Croquis, ScopeData};
use vize_relief::BindingType;
use vize_relief::RootNode;

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

        // Collect template-unused components first (to avoid borrow conflicts)
        let (template_unused_components, import_statement_ranges): (Vec<String>, Vec<(u32, u32)>) = {
            let Some(analysis) = ctx.analysis() else {
                return;
            };

            let registered_components = Self::imported_component_names(analysis);

            let import_statement_ranges = analysis
                .import_statements
                .iter()
                .map(|import| (import.start, import.end))
                .collect();

            let template_unused_components = registered_components
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
                .collect();

            (template_unused_components, import_statement_ranges)
        };

        let script_used_components = script_setup_component_import_references(
            ctx,
            &template_unused_components,
            &import_statement_ranges,
        );
        let unused_components: Vec<String> = template_unused_components
            .into_iter()
            .filter(|name| !script_used_components.contains(name.as_str()))
            .collect();

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

fn script_setup_component_import_references(
    ctx: &LintContext<'_>,
    candidate_names: &[String],
    import_statement_ranges: &[(u32, u32)],
) -> FxHashSet<CompactString> {
    if candidate_names.is_empty() {
        return FxHashSet::default();
    }

    let Some(script_setup) = ctx
        .sfc_descriptor()
        .and_then(|descriptor| descriptor.script_setup.as_ref())
    else {
        return FxHashSet::default();
    };

    let source = script_setup.content.as_ref();
    if !candidate_names.iter().any(|name| {
        has_non_import_identifier_reference(source, name.as_str(), import_statement_ranges)
    }) {
        return FxHashSet::default();
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path("component.ts").unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return FxHashSet::default();
    }

    let mut visitor = ScriptSetupComponentImportVisitor {
        component_imports: FxHashSet::default(),
        referenced_imports: FxHashSet::default(),
        scopes: Vec::new(),
        type_depth: 0,
    };
    visitor.visit_program(&parsed.program);
    visitor.referenced_imports
}

fn has_non_import_identifier_reference(
    source: &str,
    name: &str,
    import_statement_ranges: &[(u32, u32)],
) -> bool {
    source.match_indices(name).any(|(index, _)| {
        let start = index as u32;
        let end = (index + name.len()) as u32;
        !import_statement_ranges
            .iter()
            .any(|(import_start, import_end)| start >= *import_start && end <= *import_end)
            && is_identifier_boundary(source.as_bytes().get(index.wrapping_sub(1)).copied())
            && is_identifier_boundary(source.as_bytes().get(index + name.len()).copied())
    })
}

fn is_identifier_boundary(byte: Option<u8>) -> bool {
    !byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$')
}

struct ScriptSetupComponentImportVisitor {
    component_imports: FxHashSet<CompactString>,
    referenced_imports: FxHashSet<CompactString>,
    scopes: Vec<FxHashSet<CompactString>>,
    type_depth: usize,
}

impl<'a> Visit<'a> for ScriptSetupComponentImportVisitor {
    fn enter_scope(
        &mut self,
        _flags: oxc_syntax::scope::ScopeFlags,
        _scope_id: &std::cell::Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        self.scopes.push(FxHashSet::default());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.import_kind.is_type()
            || !NoUnusedComponents::is_component_import_source(it.source.value.as_str())
        {
            return;
        }

        let Some(specifiers) = &it.specifiers else {
            return;
        };

        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if !specifier.import_kind.is_type() =>
                {
                    self.component_imports
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.component_imports
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.component_imports
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
                _ => {}
            }
        }
    }

    fn visit_binding_identifier(&mut self, it: &oxc_ast::ast::BindingIdentifier<'a>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(CompactString::new(it.name.as_str()));
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.type_depth > 0 {
            return;
        }

        let name = it.name.as_str();
        if self.component_imports.contains(name) && !self.is_shadowed(name) {
            self.referenced_imports.insert(CompactString::new(name));
        }
    }

    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        self.type_depth += 1;
        walk_ts_type(self, it);
        self.type_depth -= 1;
    }
}

impl ScriptSetupComponentImportVisitor {
    fn is_shadowed(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
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
