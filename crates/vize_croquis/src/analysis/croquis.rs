//! `Croquis` method implementations and analysis statistics.
//!
//! Contains the query methods on `Croquis` for checking bindings,
//! props, emits, models, and unused template variables.

use super::bindings::UnusedTemplateVar;
use super::bindings::UnusedVarContext;
use super::Croquis;
use vize_carton::CompactString;
use vize_relief::BindingType;

impl Croquis {
    /// Create a new empty analysis summary
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a variable is defined in any scope
    #[inline]
    pub fn is_defined(&self, name: &str) -> bool {
        self.scopes.is_defined(name) || self.bindings.contains(name)
    }

    /// Get the binding type for a name
    #[inline]
    pub fn get_binding_type(&self, name: &str) -> Option<BindingType> {
        // First check scope chain (template-local variables)
        if let Some((_, binding)) = self.scopes.lookup(name) {
            return Some(binding.binding_type);
        }
        // Then check script bindings
        self.bindings.get(name)
    }

    /// Check if a name needs .value access in template
    ///
    /// In templates, refs are auto-unwrapped, so this returns false.
    /// Use `needs_value_in_script` for script context.
    #[inline]
    pub fn needs_value_in_template(&self, _name: &str) -> bool {
        // Templates auto-unwrap refs
        false
    }

    /// Check if a name needs .value access in script
    #[inline]
    pub fn needs_value_in_script(&self, name: &str) -> bool {
        self.reactivity.needs_value_access(name)
    }

    /// Check if a component is registered/imported
    #[inline]
    pub fn is_component_registered(&self, name: &str) -> bool {
        // Check if it's in used_components or is a known const binding
        // Components are typically imported as SetupConst
        self.used_components.contains(name)
            || self
                .bindings
                .get(name)
                .is_some_and(|t| matches!(t, BindingType::SetupConst))
    }

    /// Get props defined via defineProps
    pub fn get_props(&self) -> impl Iterator<Item = (&str, bool)> {
        self.macros
            .props()
            .iter()
            .map(|p| (p.name.as_str(), p.required))
    }

    /// Get emits defined via defineEmits
    pub fn get_emits(&self) -> impl Iterator<Item = &str> {
        self.macros.emits().iter().map(|e| e.name.as_str())
    }

    /// Get models defined via defineModel
    pub fn get_models(&self) -> impl Iterator<Item = &str> {
        self.macros.models().iter().map(|m| m.name.as_str())
    }

    /// Check if component uses async setup (top-level await)
    #[inline]
    pub fn is_async(&self) -> bool {
        self.macros.is_async()
    }

    /// Get unused template variables (v-for, v-slot variables that are not used)
    pub fn unused_template_vars(&self) -> Vec<UnusedTemplateVar> {
        use crate::scope::{ScopeData, ScopeKind};

        let mut unused = Vec::new();

        for scope in self.scopes.iter() {
            // Only check v-for and v-slot scopes
            if !matches!(scope.kind, ScopeKind::VFor | ScopeKind::VSlot) {
                continue;
            }

            for (name, binding) in scope.bindings() {
                if !binding.is_used() {
                    let context = match scope.data() {
                        ScopeData::VFor(data) => {
                            // Determine which kind of variable this is
                            if data.value_alias.as_str() == name {
                                UnusedVarContext::VForValue
                            } else if data.key_alias.as_ref().is_some_and(|k| k.as_str() == name) {
                                UnusedVarContext::VForKey
                            } else if data
                                .index_alias
                                .as_ref()
                                .is_some_and(|i| i.as_str() == name)
                            {
                                UnusedVarContext::VForIndex
                            } else {
                                UnusedVarContext::VForValue
                            }
                        }
                        ScopeData::VSlot(data) => UnusedVarContext::VSlot {
                            slot_name: data.name.clone(),
                        },
                        _ => continue,
                    };

                    unused.push(UnusedTemplateVar {
                        name: CompactString::new(name),
                        offset: binding.declaration_offset,
                        context,
                    });
                }
            }
        }

        unused
    }

    /// Get analysis statistics for debugging
    pub fn stats(&self) -> AnalysisStats {
        AnalysisStats {
            scope_count: self.scopes.len(),
            symbol_count: self.symbols.len(),
            binding_count: self.bindings.bindings.len(),
            macro_count: self.macros.all_calls().len(),
            prop_count: self.macros.props().len(),
            emit_count: self.macros.emits().len(),
            model_count: self.macros.models().len(),
            hoist_count: self.hoists.count(),
            used_components: self.used_components.len(),
            used_directives: self.used_directives.len(),
            undefined_ref_count: self.undefined_refs.len(),
            unused_binding_count: self.unused_bindings.len(),
        }
    }
}

/// Statistics about the analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    pub scope_count: usize,
    pub symbol_count: usize,
    pub binding_count: usize,
    pub macro_count: usize,
    pub prop_count: usize,
    pub emit_count: usize,
    pub model_count: usize,
    pub hoist_count: usize,
    pub used_components: usize,
    pub used_directives: usize,
    pub undefined_ref_count: usize,
    pub unused_binding_count: usize,
}
