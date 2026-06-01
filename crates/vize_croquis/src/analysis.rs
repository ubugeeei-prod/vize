//! Analysis summary for Vue SFC semantic analysis.
//!
//! This module provides the `Croquis` struct that aggregates all
//! semantic analysis results from a Vue SFC. It serves as the bridge between
//! the parser and downstream consumers (linter, transformer, codegen).
//!
//! ## Architecture
//!
//! ```text
//! vize_armature (Parse)
//!        |
//!   vize_relief (AST)
//!        |
//!  vize_croquis (Semantic Analysis)
//!        |
//!   Croquis  <--  This module
//!        |
//!  +-----+-----+
//!  |           |
//! patina    atelier
//! (lint)    (transform)
//! ```
//!
//! ## Submodules
//!
//! - [`bindings`] - Binding metadata, undefined refs, exports
//! - [`template`] - Template info, expressions, component usage
//! - [`croquis`]  - `Croquis` query methods and statistics
//! - [`vir`]      - VIR text format output

mod bindings;
mod croquis;
mod template;
mod vir;

// Re-export all public types so downstream `use analysis::*` still works.
pub use bindings::{
    BindingMetadata, COMPILER_MACRO_NAMES, ImportStatementInfo, InvalidExport, InvalidExportKind,
    ReExportInfo, TypeExport, TypeExportKind, UndefinedRef, UnusedTemplateVar, UnusedVarContext,
};
pub use croquis::AnalysisStats;
pub use template::{
    ComponentRegistration, ComponentUsage, ElementIdInfo, ElementIdKind, EventListener, PassedProp,
    SlotUsage, TemplateExpression, TemplateExpressionKind, TemplateInfo,
};

use crate::hoist::HoistTracker;
use crate::macros::MacroTracker;
use crate::provide::ProvideInjectTracker;
use crate::race::RaceConditionTracker;
use crate::reactivity::ReactivityTracker;
use crate::setup_context::SetupContextTracker;
use crate::types::TypeResolver;
use crate::{ScopeChain, SymbolTable};
use vize_carton::{CompactString, FxHashMap, FxHashSet};

/// Complete semantic analysis summary for a Vue SFC.
///
/// This struct aggregates all analysis results and provides a unified
/// interface for downstream consumers (linter, transformer).
#[derive(Debug, Default)]
pub struct Croquis {
    /// Scope chain for template expressions
    pub scopes: ScopeChain,

    /// Symbol table for script bindings
    pub symbols: SymbolTable,

    /// Compiler macro information (defineProps, defineEmits, etc.)
    pub macros: MacroTracker,

    /// Reactivity tracking (ref, reactive, computed)
    pub reactivity: ReactivityTracker,

    /// Race-condition risk tracking for async reactive writes
    pub race_conditions: RaceConditionTracker,

    /// Provide/Inject tracking
    pub provide_inject: ProvideInjectTracker,

    /// Setup context violation tracking (CSRP/memory leaks)
    pub setup_context: SetupContextTracker,

    /// TypeScript type resolution
    pub types: TypeResolver,

    /// Hoisting analysis for template optimization
    pub hoists: HoistTracker,

    /// Script binding metadata (for template access)
    pub bindings: BindingMetadata,

    /// Template-level metadata (root count, $attrs usage, etc.)
    pub template_info: TemplateInfo,

    /// Components used in template (names only, for quick lookup)
    pub used_components: FxHashSet<CompactString>,

    /// Components registered through Options API `components`.
    pub component_registrations: Vec<ComponentRegistration>,

    /// Detailed component usage information (props, events, slots)
    pub component_usages: Vec<ComponentUsage>,

    /// Directives used in template
    pub used_directives: FxHashSet<CompactString>,

    /// Variables referenced in template but not defined
    pub undefined_refs: Vec<UndefinedRef>,

    /// Unused bindings (defined but not referenced in template)
    pub unused_bindings: Vec<CompactString>,

    /// Type exports from script setup (hoisted to module level)
    pub type_exports: Vec<TypeExport>,

    /// Invalid non-type exports in script setup
    pub invalid_exports: Vec<InvalidExport>,

    /// Template expressions for type checking (interpolations, v-bind, etc.)
    pub template_expressions: Vec<TemplateExpression>,

    /// Element IDs found in template (for cross-file uniqueness checking)
    pub element_ids: Vec<ElementIdInfo>,

    /// Import statement spans in script content
    pub import_statements: Vec<ImportStatementInfo>,

    /// Re-export statement spans (`export { ... } from "..."`) in script content
    pub re_exports: Vec<ReExportInfo>,

    /// Definition spans for bindings (name -> (start, end) offset in script)
    /// Used for Go-to-Definition support.
    pub binding_spans: FxHashMap<CompactString, (u32, u32)>,
}

#[cfg(test)]
mod tests {
    use super::{BindingMetadata, Croquis};
    use crate::BindingType;

    #[test]
    fn test_binding_metadata() {
        let mut meta = BindingMetadata::script_setup();
        meta.add("count", BindingType::SetupRef);
        meta.add("state", BindingType::SetupReactiveConst);
        meta.add("msg", BindingType::Props);

        assert!(meta.is_script_setup);
        assert!(meta.is_ref("count"));
        assert!(!meta.is_ref("state"));
        assert!(meta.is_prop("msg"));
    }

    #[test]
    fn test_analysis_summary() {
        let mut summary = Croquis::new();
        summary.bindings.add("foo", BindingType::SetupRef);

        assert!(summary.is_defined("foo"));
        assert!(!summary.is_defined("bar"));
        assert_eq!(summary.get_binding_type("foo"), Some(BindingType::SetupRef));
    }
}
