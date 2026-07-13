//! Script compile context.
//!
//! Holds all state during script compilation.
//! Uses OXC for proper AST-based parsing instead of regex.

mod batch_epoch;
mod external_types;
mod helpers;
mod parse;
mod props;
pub use batch_epoch::{TypeResolutionBatchGuard, begin_type_resolution_batch};
pub(crate) use external_types::type_import_specifiers_from_program;

use crate::types::{BindingMetadata, BindingType};
use vize_carton::{CompactString, String, ToCompactString};
use vize_croquis::croquis::Croquis;
use vize_croquis::macros::{EmitDefinition, ModelDefinition, PropDefinition};

use super::ScriptSetupMacros;

/// Script compile context - holds all state during compilation
#[derive(Debug, Clone)]
pub struct ScriptCompileContext {
    /// Source content
    pub source: String,

    /// Binding metadata
    pub bindings: BindingMetadata,

    /// Extracted macros
    pub macros: ScriptSetupMacros,

    /// Whether defineProps was called
    pub has_define_props_call: bool,

    /// Whether defineEmits was called
    pub has_define_emits_call: bool,

    /// Whether defineExpose was called
    pub has_define_expose_call: bool,

    /// Whether defineOptions was called
    pub has_define_options_call: bool,

    /// Whether defineSlots was called
    pub has_define_slots_call: bool,

    /// Whether defineModel was called
    pub has_define_model_call: bool,

    // --- Emits related fields ---
    /// Runtime declaration for emits (the argument passed to defineEmits)
    pub emits_runtime_decl: Option<String>,

    /// Type declaration for emits (the type parameter)
    pub emits_type_decl: Option<String>,

    /// The variable name emits is assigned to (e.g., "emit")
    pub emit_decl_id: Option<String>,

    /// TypeScript interface definitions (name -> body)
    /// Used to resolve type references in defineProps<InterfaceName>()
    pub interfaces: vize_carton::FxHashMap<String, String>,

    /// TypeScript type alias definitions (name -> body)
    /// Used to resolve type references in defineProps<TypeName>()
    pub type_aliases: vize_carton::FxHashMap<String, String>,

    /// Reactive props-destructure default value spans, keyed by public prop name.
    ///
    /// These spans are captured while the authored Program is live so semantic
    /// diagnostics do not need to parse the script block again.
    pub(crate) props_destructure_default_spans: vize_carton::FxHashMap<String, (u32, u32)>,
}

impl ScriptCompileContext {
    /// Create a new context
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_compact_string(),
            bindings: BindingMetadata::default(),
            macros: ScriptSetupMacros::default(),
            has_define_props_call: false,
            has_define_emits_call: false,
            has_define_expose_call: false,
            has_define_options_call: false,
            has_define_slots_call: false,
            has_define_model_call: false,
            emits_runtime_decl: None,
            emits_type_decl: None,
            emit_decl_id: None,
            interfaces: vize_carton::FxHashMap::default(),
            type_aliases: vize_carton::FxHashMap::default(),
            props_destructure_default_spans: vize_carton::FxHashMap::default(),
        }
    }

    /// Analyze script setup and extract bindings
    pub fn analyze(&mut self) {
        // Temporarily take ownership of source to avoid borrow conflicts
        let source = std::mem::take(&mut self.source);
        self.parse_with_oxc(&source);
        self.source = source;
        // ScriptCompileContext is always used for <script setup>
        self.bindings.is_script_setup = true;
    }

    /// Analyze script setup from an already-parsed oxc program.
    ///
    /// Parse-free variant of [`Self::analyze`] for the SFC compiler's
    /// parse-once pipeline. `source` must be the exact text `program` was
    /// parsed from (and match the source this context was created with).
    pub fn analyze_program(&mut self, program: &oxc_ast::ast::Program<'_>, source: &str) {
        self.process_program(program, source);
        // ScriptCompileContext is always used for <script setup>
        self.bindings.is_script_setup = true;
    }

    /// Convert to an Croquis for use in transforms and linting.
    ///
    /// This bridges the atelier script context to the shared croquis analysis format.
    pub fn to_analysis_summary(&self) -> Croquis {
        let mut summary = Croquis::new();

        // Convert bindings
        summary.bindings.is_script_setup = true;
        for (name, binding_type) in &self.bindings.bindings {
            summary.bindings.add(name.as_str(), (*binding_type).into());
        }

        // Convert props aliases
        for (local, key) in &self.bindings.props_aliases {
            summary
                .bindings
                .props_aliases
                .insert(CompactString::new(local), CompactString::new(key));
        }

        // Convert props from macros
        if let Some(ref props_call) = self.macros.define_props {
            for (name, binding_type) in &self.bindings.bindings {
                if matches!(binding_type, BindingType::Props) {
                    summary.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required: false, // We don't track this in the current implementation
                        prop_type: None,
                        default_value: props_call.binding_name.clone().map(CompactString::new),
                    });
                }
            }
        }

        // Convert emits
        if let Some(ref emits_call) = self.macros.define_emits {
            // Parse emits from the macro call args if available
            let trimmed = emits_call.args.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Array syntax: ['click', 'update']
                let inner = &trimmed[1..trimmed.len() - 1];
                for part in inner.split(',') {
                    let part = part.trim();
                    if (part.starts_with('\'') && part.ends_with('\''))
                        || (part.starts_with('"') && part.ends_with('"'))
                    {
                        let name = &part[1..part.len() - 1];
                        summary.macros.add_emit(EmitDefinition {
                            name: CompactString::new(name),
                            payload_type: None,
                        });
                    }
                }
            }
        }

        // Convert models
        for model_call in &self.macros.define_models {
            if let Some(ref binding_name) = model_call.binding_name {
                let name = CompactString::new(
                    super::define_model_name(self.source.as_str(), model_call).as_str(),
                );

                summary.macros.add_model(ModelDefinition {
                    name: name.clone(),
                    local_name: CompactString::new(binding_name),
                    model_type: None,
                    required: false,
                    default_value: None,
                });
            }
        }

        summary
    }

    /// Extract all macros from the source
    pub fn extract_all_macros(&mut self) {
        let source = std::mem::take(&mut self.source);
        self.parse_with_oxc(&source);
        self.source = source;
    }
}

#[cfg(test)]
mod tests;
