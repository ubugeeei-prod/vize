//! Display types for VIR (Vize Intermediate Representation) output.
//!
//! Provides human-readable TOML-like format for semantic analysis results.
//!
//! This module is split into:
//! - Types and enums (this file)
//! - `formatters`: `SummaryBuilder` and `Croquis::to_vir()` implementation

mod formatters;

use crate::hoist::PatchFlags;
use vize_carton::String;
use vize_relief::BindingType;

pub use formatters::SummaryBuilder;

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Severity {
    Error = 0,
    Warning = 1,
    Info = 2,
    Hint = 3,
}

/// Related information for a diagnostic
#[derive(Debug, Clone)]
pub struct RelatedInfo {
    pub message: String,
    pub start: u32,
    pub end: u32,
}

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub start: u32,
    pub end: u32,
    pub code: Option<String>,
    pub related: Vec<RelatedInfo>,
}

mod scope_kind;
pub use scope_kind::ScopeKind;

/// Binding source for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingSource {
    ScriptSetup,
    Props,
    Data,
    Computed,
    Methods,
    Inject,
    Import,
    Local,
}

/// Binding metadata for display
#[derive(Debug, Clone)]
pub struct BindingMetadata {
    pub binding_type: BindingType,
    pub source: BindingSource,
    pub is_used: bool,
    pub is_mutated: bool,
}

/// Scope display info
#[derive(Debug, Clone)]
pub struct ScopeDisplay {
    pub id: u32,
    pub kind: ScopeKind,
    pub parent_ids: Vec<u32>,
    pub start: u32,
    pub end: u32,
    pub bindings: Vec<(String, BindingMetadata)>,
}

/// Binding display info
#[derive(Debug, Clone)]
pub struct BindingDisplay {
    pub name: String,
    pub binding_type: String,
    pub source: String,
}

/// Prop display info
#[derive(Debug, Clone)]
pub struct PropDisplay {
    pub name: String,
    pub prop_type: Option<String>,
    pub required: bool,
    pub has_default: bool,
}

/// Emit display info
#[derive(Debug, Clone)]
pub struct EmitDisplay {
    pub name: String,
    pub payload_type: Option<String>,
}

/// Macro display info
#[derive(Debug, Clone)]
pub struct MacroDisplay {
    pub name: String,
    pub kind: String,
    pub start: u32,
    pub end: u32,
}

/// Hoist display info
#[derive(Debug, Clone)]
pub struct HoistDisplay {
    pub id: u32,
    pub level: String,
    pub content: String,
}

/// Selector display info
#[derive(Debug, Clone)]
pub struct SelectorDisplay {
    pub raw: String,
    pub scoped: bool,
}

/// CSS display info
#[derive(Debug, Clone)]
pub struct CssDisplay {
    pub selectors: Vec<SelectorDisplay>,
    pub v_bind_count: u32,
    pub has_deep: bool,
    pub has_slotted: bool,
    pub has_global: bool,
}

/// Patch flag display info
#[derive(Debug, Clone)]
pub struct PatchFlagDisplay {
    pub value: i32,
    pub names: Vec<String>,
}

impl From<PatchFlags> for PatchFlagDisplay {
    fn from(flags: PatchFlags) -> Self {
        Self {
            value: flags.bits(),
            names: flags.flag_names().into_iter().map(String::from).collect(),
        }
    }
}

/// Block display info
#[derive(Debug, Clone)]
pub struct BlockDisplay {
    pub id: u32,
    pub block_type: String,
    pub parent_id: Option<u32>,
    pub dynamic_children: u32,
}

/// Event cache display info
#[derive(Debug, Clone)]
pub struct EventCacheDisplay {
    pub cache_index: u32,
    pub event_name: String,
    pub handler: String,
    pub is_inline: bool,
}

/// Once cache display info
#[derive(Debug, Clone)]
pub struct OnceCacheDisplay {
    pub cache_index: u32,
    pub content: String,
    pub start: u32,
    pub end: u32,
}

/// Memo cache display info
#[derive(Debug, Clone)]
pub struct MemoCacheDisplay {
    pub cache_index: u32,
    pub deps: String,
    pub content: String,
    pub start: u32,
    pub end: u32,
}

/// Top-level await display info
#[derive(Debug, Clone)]
pub struct TopLevelAwaitDisplay {
    pub expression: String,
    pub start: u32,
    pub end: u32,
}

/// Optimization display info
#[derive(Debug, Clone)]
pub struct OptimizationDisplay {
    pub patch_flags: Vec<PatchFlagDisplay>,
    pub blocks: Vec<BlockDisplay>,
    pub event_cache: Vec<EventCacheDisplay>,
    pub once_cache: Vec<OnceCacheDisplay>,
    pub memo_cache: Vec<MemoCacheDisplay>,
}

/// Analysis statistics
#[derive(Debug, Clone, Default)]
pub struct AnalysisStats {
    pub scope_count: u32,
    pub binding_count: u32,
    pub prop_count: u32,
    pub emit_count: u32,
    pub model_count: u32,
    pub hoist_count: u32,
    pub cache_count: u32,
}

/// Complete analysis summary
#[derive(Debug, Clone)]
pub struct Croquis {
    pub scopes: Vec<ScopeDisplay>,
    pub bindings: Vec<BindingDisplay>,
    pub props: Vec<PropDisplay>,
    pub emits: Vec<EmitDisplay>,
    pub macros: Vec<MacroDisplay>,
    pub hoists: Vec<HoistDisplay>,
    pub css: Option<CssDisplay>,
    pub optimization: OptimizationDisplay,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: AnalysisStats,
    pub is_async: bool,
    pub top_level_awaits: Vec<TopLevelAwaitDisplay>,
}

impl Default for Croquis {
    fn default() -> Self {
        Self {
            scopes: Vec::new(),
            bindings: Vec::new(),
            props: Vec::new(),
            emits: Vec::new(),
            macros: Vec::new(),
            hoists: Vec::new(),
            css: None,
            optimization: OptimizationDisplay {
                patch_flags: Vec::new(),
                blocks: Vec::new(),
                event_cache: Vec::new(),
                once_cache: Vec::new(),
                memo_cache: Vec::new(),
            },
            diagnostics: Vec::new(),
            stats: AnalysisStats::default(),
            is_async: false,
            top_level_awaits: Vec::new(),
        }
    }
}
