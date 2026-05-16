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

/// Scope kind for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Module,
    Function,
    Block,
    VFor,
    VSlot,
    EventHandler,
    Callback,
    ScriptSetup,
    NonScriptSetup,
    Universal,
    ClientOnly,
    JsGlobalUniversal,
    JsGlobalBrowser,
    JsGlobalNode,
    JsGlobalDeno,
    JsGlobalBun,
    VueGlobal,
    ExternalModule,
    Closure,
}

impl From<crate::scope::ScopeKind> for ScopeKind {
    fn from(kind: crate::scope::ScopeKind) -> Self {
        match kind {
            crate::scope::ScopeKind::Module => Self::Module,
            crate::scope::ScopeKind::Function => Self::Function,
            crate::scope::ScopeKind::Block => Self::Block,
            crate::scope::ScopeKind::VFor => Self::VFor,
            crate::scope::ScopeKind::VSlot => Self::VSlot,
            crate::scope::ScopeKind::EventHandler => Self::EventHandler,
            crate::scope::ScopeKind::Callback => Self::Callback,
            crate::scope::ScopeKind::ScriptSetup => Self::ScriptSetup,
            crate::scope::ScopeKind::NonScriptSetup => Self::NonScriptSetup,
            crate::scope::ScopeKind::Universal => Self::Universal,
            crate::scope::ScopeKind::ClientOnly => Self::ClientOnly,
            crate::scope::ScopeKind::JsGlobalUniversal => Self::JsGlobalUniversal,
            crate::scope::ScopeKind::JsGlobalBrowser => Self::JsGlobalBrowser,
            crate::scope::ScopeKind::JsGlobalNode => Self::JsGlobalNode,
            crate::scope::ScopeKind::JsGlobalDeno => Self::JsGlobalDeno,
            crate::scope::ScopeKind::JsGlobalBun => Self::JsGlobalBun,
            crate::scope::ScopeKind::VueGlobal => Self::VueGlobal,
            crate::scope::ScopeKind::ExternalModule => Self::ExternalModule,
            crate::scope::ScopeKind::Closure => Self::Closure,
        }
    }
}

impl ScopeKind {
    /// Get the display prefix for this scope kind
    ///
    /// - `~` = universal (works on both client and server)
    /// - `!` = client only (requires client API: window, document, etc.)
    /// - `#` = server private (reserved for future Server Components)
    #[inline]
    pub const fn prefix(&self) -> &'static str {
        match self {
            // Client-only (requires client API)
            Self::ClientOnly | Self::JsGlobalBrowser => "!",
            // Server private (reserved for future Server Components)
            Self::JsGlobalNode | Self::JsGlobalDeno | Self::JsGlobalBun => "#",
            // Universal (works on both)
            _ => "~",
        }
    }

    /// Get the display name for this scope kind
    #[inline]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "callback",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "universal",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
        }
    }

    /// Format for VIR display (zero allocation)
    #[inline]
    pub const fn to_display(&self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "callback",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "universal",
            Self::JsGlobalBrowser => "client",
            Self::JsGlobalNode => "server",
            Self::JsGlobalDeno => "server",
            Self::JsGlobalBun => "server",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "extern",
            Self::Closure => "closure",
        }
    }

    /// Get reference prefix for parent scope references
    /// - `~` = universal (works on both client and server)
    /// - `!` = client only (requires client API)
    /// - `#` = server private (reserved for future Server Components)
    #[inline]
    pub const fn ref_prefix(&self) -> &'static str {
        match self {
            Self::ClientOnly | Self::JsGlobalBrowser => "!",
            Self::JsGlobalNode | Self::JsGlobalDeno | Self::JsGlobalBun => "#",
            _ => "~",
        }
    }
}

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
