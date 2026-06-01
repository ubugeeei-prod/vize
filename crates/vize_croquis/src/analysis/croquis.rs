//! `Croquis` method implementations and analysis statistics.
//!
//! Contains the query methods on `Croquis` for checking bindings,
//! props, emits, models, and unused template variables.

use super::Croquis;
use super::bindings::UnusedTemplateVar;
use super::bindings::UnusedVarContext;
use vize_carton::CompactString;
use vize_relief::BindingType;

impl Croquis {
    /// Create a new empty analysis summary
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Shift script-relative offsets after embedding this analysis into a
    /// larger synthetic script. Call this before adding template analysis.
    pub fn shift_script_offsets(&mut self, delta: u32) {
        if delta == 0 {
            return;
        }

        self.scopes.shift_script_offsets(delta);
        self.symbols.shift_offsets(delta);
        self.macros.shift_offsets(delta);
        self.reactivity.shift_offsets(delta);
        self.race_conditions.shift_offsets(delta);
        self.provide_inject.shift_offsets(delta);
        self.setup_context.shift_offsets(delta);

        for type_export in &mut self.type_exports {
            type_export.start = type_export.start.saturating_add(delta);
            type_export.end = type_export.end.saturating_add(delta);
        }
        for invalid_export in &mut self.invalid_exports {
            invalid_export.start = invalid_export.start.saturating_add(delta);
            invalid_export.end = invalid_export.end.saturating_add(delta);
        }
        for import in &mut self.import_statements {
            import.start = import.start.saturating_add(delta);
            import.end = import.end.saturating_add(delta);
        }
        for re_export in &mut self.re_exports {
            re_export.start = re_export.start.saturating_add(delta);
            re_export.end = re_export.end.saturating_add(delta);
        }
        for span in self.binding_spans.values_mut() {
            span.0 = span.0.saturating_add(delta);
            span.1 = span.1.saturating_add(delta);
        }
    }

    /// Merge a regular `<script>` analysis into a `<script setup>` analysis.
    ///
    /// The receiver keeps precedence for setup-local data, while module-level
    /// facts from the regular script are retained for virtual TS, lint, and
    /// cross-file consumers.
    pub fn merge_plain_script(&mut self, plain: Self) {
        let plain_bindings = plain.bindings;
        self.bindings.is_script_setup |= plain_bindings.is_script_setup;
        for (name, binding_type) in plain_bindings.bindings {
            self.bindings.bindings.entry(name).or_insert(binding_type);
        }
        for (local, prop) in plain_bindings.props_aliases {
            self.bindings.props_aliases.entry(local).or_insert(prop);
        }

        self.reactivity.extend(plain.reactivity);
        self.race_conditions.extend(plain.race_conditions);
        self.provide_inject.extend(plain.provide_inject);
        self.setup_context.extend(plain.setup_context);
        self.type_exports.extend(plain.type_exports);
        self.import_statements.extend(plain.import_statements);
        self.re_exports.extend(plain.re_exports);
        self.component_registrations
            .extend(plain.component_registrations);

        for (name, span) in plain.binding_spans {
            self.binding_spans.entry(name).or_insert(span);
        }
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
                            if data
                                .value_bindings
                                .iter()
                                .any(|value| value.as_str() == name)
                            {
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
