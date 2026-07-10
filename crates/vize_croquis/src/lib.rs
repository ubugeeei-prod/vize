//! # vize_croquis
//!
//! Croquis - The semantic analysis layer for Vize.
//!
//! ## Name Origin
//!
//! **Croquis** (/kʁɔ.ki/) is a French term for a quick, sketchy drawing that captures
//! the essential features of a subject. Like how artists use croquis to rapidly
//! capture the essence of a pose or scene, `vize_croquis` quickly analyzes Vue
//! templates to extract semantic meaning from the syntactic structure.
//!
//! ## Purpose
//!
//! Relief records what syntax was written and where. Croquis derives what that
//! syntax means and how it relates: identity, scopes, dependencies, and graphs.
//! It provides:
//!
//! - **Scope Analysis**: Track variable scopes across templates and scripts
//! - **Binding Resolution**: Resolve identifiers to their declarations
//! - **Reactivity Tracking**: Understand ref/reactive dependencies
//! - **Symbol Tables**: Fast lookup of bindings and their metadata
//!
//! ## Architecture
//!
//! ```text
//! SFC/Relief provider ─┐
//!                     ├─> CroquisSemanticSnapshot (this crate)
//! JSX/OXC provider ───┘
//!                                ├─> Patina / Canon / project queries
//!                                └─> other semantic consumers
//! ```
//!
//! The owned `semantic` contract is always available and contains no frontend
//! AST references. The `analysis` feature adds Relief-free script/scope/
//! reactivity analysis. The default `relief-compat` feature retains legacy
//! Vue-template Drawer and Virtual TS adapters while production callers are
//! migrated to frontend providers.

#![cfg_attr(test, allow(clippy::disallowed_macros, clippy::disallowed_types))]

// Core modules
mod product;
#[cfg(feature = "analysis")]
mod scope;
pub mod semantic;
#[cfg(feature = "analysis")]
mod symbol;

// Croquis modules
#[cfg(feature = "analysis")]
pub mod analysis;
#[cfg(feature = "analysis")]
pub mod analyzer;
#[cfg(feature = "analysis")]
pub mod builtins;
#[cfg(feature = "analysis")]
pub mod call_graph;
#[cfg(feature = "analysis")]
pub mod croquis;
#[cfg(feature = "analysis")]
pub mod css;
#[cfg(feature = "analysis")]
pub mod declaration_ts;
#[cfg(feature = "analysis")]
pub mod display;
#[cfg(feature = "analysis")]
pub mod drawer;
#[cfg(feature = "analysis")]
pub mod effect_graph;
#[cfg(feature = "analysis")]
pub mod facts;
#[cfg(feature = "analysis")]
pub mod hoist;
#[cfg(feature = "analysis")]
pub mod import_resolver;
#[cfg(feature = "analysis")]
pub mod macros;
#[cfg(feature = "analysis")]
pub mod naming;
#[cfg(feature = "analysis")]
pub mod optimization;
#[cfg(feature = "analysis")]
pub mod provide;
#[cfg(feature = "analysis")]
pub mod race;
#[cfg(feature = "analysis")]
pub mod reactivity;
#[cfg(feature = "analysis")]
pub mod reactivity_overlay;
#[cfg(feature = "analysis")]
pub mod reactivity_tracking;
#[cfg(feature = "analysis")]
pub mod render_tree;
#[cfg(feature = "analysis")]
pub mod script_parser;
#[cfg(feature = "analysis")]
pub mod setup_context;
#[cfg(feature = "analysis")]
pub mod types;
#[cfg(feature = "relief-compat")]
pub mod virtual_ts;

#[cfg(all(test, feature = "analysis"))]
mod effect_graph_builder_tests;

#[cfg(all(test, feature = "analysis"))]
mod reactivity_overlay_tests;

// Re-export commonly used utilities from vize_carton for convenience
pub use vize_carton::{
    is_builtin_directive, is_builtin_tag, is_html_tag, is_math_ml_tag, is_native_tag,
    is_reserved_prop, is_svg_tag, is_void_tag,
};

// Re-export core types
pub use product::CroquisSemanticProduct;
#[cfg(feature = "analysis")]
pub use scope::{
    BindingFlags, BlockKind, BlockScopeData, CallbackScopeData, ClientOnlyScopeData,
    ClosureScopeData, EventHandlerScopeData, ExternalModuleScopeData, JsGlobalScopeData, JsRuntime,
    NonScriptSetupScopeData, PARAM_INLINE_CAP, ParamNames, ParentScopes, Scope, ScopeBinding,
    ScopeChain, ScopeData, ScopeId, ScopeKind, ScriptSetupScopeData, Span, UniversalScopeData,
    VForScopeData, VSlotScopeData, VueGlobalScopeData,
};
pub use semantic::{
    CroquisSemanticSnapshot, CroquisSemanticSnapshotBuilder, CroquisSemanticSummary,
    SemanticBindingSnapshot, SemanticComponentUsageSnapshot, SemanticEventListenerSnapshot,
    SemanticInjectSnapshot, SemanticPassedPropSnapshot, SemanticProvideSnapshot,
    SemanticReactiveSourceSnapshot, SemanticReactivityLossSnapshot, SemanticScopeBindingSnapshot,
    SemanticScopeSnapshot, SemanticSlotUsageSnapshot, SemanticSourceRange,
    SemanticTemplateExpressionSnapshot,
};
#[cfg(feature = "analysis")]
pub use symbol::{Symbol, SymbolFlags, SymbolId, SymbolTable};

// Re-export analysis types
#[cfg(feature = "analysis")]
pub use analyzer::{Analyzer, AnalyzerOptions};
#[cfg(feature = "analysis")]
pub use croquis::{
    AnalysisStats, BindingMetadata, COMPILER_MACRO_NAMES, ComponentShape, Croquis, CroquisStats,
    ImportStatementInfo, InvalidExport, InvalidExportKind, OptionGroup, OptionKey, OptionMember,
    OptionsDescriptor, ReExportInfo, TemplateExpression, TemplateExpressionKind, TypeExport,
    TypeExportKind, UndefinedRef, UnusedTemplateVar, UnusedVarContext,
};
#[cfg(feature = "analysis")]
pub use drawer::{Drawer, DrawerOptions};
#[cfg(feature = "analysis")]
pub use effect_graph::{
    EffectGraph, EffectGraphScript, EffectGraphSummary, build_effect_graph_from_script,
    build_effect_graph_from_script_setup, build_effect_graph_from_sfc_scripts,
};
#[cfg(feature = "analysis")]
pub use facts::{CroquisFact, CroquisFactSet};
#[cfg(feature = "analysis")]
pub use reactivity_overlay::{
    ReactivityEffectEdgeOverlay, ReactivityEffectGraphOverlay, ReactivityLossOverlay,
    ReactivityOverlay, ReactivityOverlaySummary, ReactivitySourceOverlay,
};

// Re-export common types
pub use vize_carton::BindingType;
