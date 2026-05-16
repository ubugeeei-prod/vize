//! Scope analysis for Vue templates and scripts.
//!
//! Provides a hierarchical scope chain that tracks variable visibility
//! across different contexts (module, function, block, v-for, v-slot).
//!
//! ## Performance Optimizations
//!
//! - Uses `CompactString` instead of `String` for identifier names (SSO for short strings)
//! - Uses `SmallVec` for parameter lists (stack-allocated for small counts)
//! - Bitflags for binding properties to reduce memory and improve cache locality
//! - `#[inline]` hints for hot path functions
//!
//! ## Module Structure
//!
//! - [`types`] - Type definitions (ScopeId, ScopeKind, ScopeData, ScopeBinding, etc.)
//! - [`chain`] - Scope and ScopeChain implementations

mod chain;
mod types;

// Re-export all public types
pub use chain::{Scope, ScopeChain};
pub use types::{
    BindingFlags, BlockKind, BlockScopeData, CallbackScopeData, ClientOnlyScopeData,
    ClosureScopeData, EventHandlerScopeData, ExternalModuleScopeData, JsGlobalScopeData, JsRuntime,
    NonScriptSetupScopeData, PARAM_INLINE_CAP, ParamNames, ParentScopes, ScopeBinding, ScopeData,
    ScopeId, ScopeKind, ScriptSetupScopeData, Span, UniversalScopeData, VForScopeData,
    VSlotScopeData, VueGlobalScopeData,
};
