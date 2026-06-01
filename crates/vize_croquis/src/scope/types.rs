//! Type definitions for scope analysis.
//!
//! This module contains all the type definitions used in scope analysis:
//! - `ScopeId` - Unique identifier for scopes
//! - `ScopeKind` - Kind of scope (Module, Function, VFor, etc.)
//! - Scope data structures for different scope types
//! - `ScopeBinding` - Binding information within a scope

mod binding;
mod scope_data;

pub use binding::{BindingFlags, ScopeBinding, Span};
pub use scope_data::{
    BlockKind, BlockScopeData, CallbackScopeData, ClientOnlyScopeData, ClosureScopeData,
    EventHandlerScopeData, ExternalModuleScopeData, JsGlobalScopeData, JsRuntime,
    NonScriptSetupScopeData, ScopeData, ScriptSetupScopeData, UniversalScopeData, VForScopeData,
    VSlotScopeData, VueGlobalScopeData,
};

use vize_carton::{CompactString, SmallVec};

/// Maximum parameters typically seen in v-for/v-slot/callbacks
/// Stack-allocated up to this count, heap-allocated beyond
pub const PARAM_INLINE_CAP: usize = 4;

/// Type alias for parameter name lists (stack-allocated for small counts)
pub type ParamNames = SmallVec<[CompactString; PARAM_INLINE_CAP]>;

/// Parent scope references (typically 1-2 parents)
pub type ParentScopes = SmallVec<[ScopeId; 2]>;

/// Unique identifier for a scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ScopeId(u32);

impl ScopeId {
    /// The root scope (SFC level)
    pub const ROOT: Self = Self(0);

    /// Create a new scope ID
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScopeKind {
    /// SFC (Single File Component) level scope
    /// This is the root scope that contains script setup/non-script-setup scopes
    Module = 0,
    /// Function scope
    Function = 1,
    /// Block scope (if, for, etc.)
    Block = 2,
    /// v-for scope (template)
    VFor = 3,
    /// v-slot scope (template)
    VSlot = 4,
    /// Event handler scope (@click, etc.)
    EventHandler = 5,
    /// Callback/arrow function scope in expressions
    Callback = 6,
    /// Script setup scope (`<script setup>`)
    ScriptSetup = 7,
    /// Non-script setup scope (Options API, regular `<script>`)
    NonScriptSetup = 8,
    /// Universal scope (SSR - runs on both server and client)
    Universal = 9,
    /// Client-only scope (onMounted, onBeforeUnmount, etc.)
    ClientOnly = 10,
    /// Universal JavaScript global scope (console, Math, Object, Array, etc.)
    /// Works in all runtimes
    JsGlobalUniversal = 11,
    /// Browser-only JavaScript global scope (window, document, navigator, localStorage, etc.)
    /// WARNING: Not available in SSR server context
    JsGlobalBrowser = 12,
    /// Node.js-only JavaScript global scope (process, Buffer, __dirname, require, etc.)
    /// WARNING: Not available in browser context
    JsGlobalNode = 13,
    /// Deno-only JavaScript global scope (Deno namespace)
    JsGlobalDeno = 14,
    /// Bun-only JavaScript global scope (Bun namespace)
    JsGlobalBun = 15,
    /// Vue global scope ($refs, $emit, $slots, $attrs, etc.)
    VueGlobal = 16,
    /// External module scope (imported modules)
    ExternalModule = 17,
    /// Closure scope (function declaration, function expression, arrow function)
    /// Has access to arguments, this, and local variables
    Closure = 18,
}

impl ScopeKind {
    /// Get the display prefix for this scope kind
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
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "cb",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "univ",
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
            Self::Module => "mod",
            Self::Function => "fn",
            Self::Block => "block",
            Self::VFor => "v-for",
            Self::VSlot => "v-slot",
            Self::EventHandler => "event",
            Self::Callback => "cb",
            Self::ScriptSetup => "setup",
            Self::NonScriptSetup => "plain",
            Self::Universal => "universal",
            Self::ClientOnly => "client",
            Self::JsGlobalUniversal => "univ",
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

#[cfg(test)]
mod tests {
    use super::{BlockKind, JsRuntime, ScopeBinding, ScopeData, ScopeId, ScopeKind, Span};
    use insta::assert_snapshot;
    use vize_relief::BindingType;

    #[test]
    fn test_scope_id_constants() {
        assert_eq!(ScopeId::ROOT.as_u32(), 0);
        assert_eq!(ScopeId::new(42).as_u32(), 42);
    }

    #[test]
    #[allow(clippy::disallowed_macros)]
    fn test_scope_kind_display() {
        assert_snapshot!(
            "scope_kind_display",
            format!(
                "Module: {} (prefix: {})\n\
             Function: {} (prefix: {})\n\
             Block: {} (prefix: {})\n\
             VFor: {} (prefix: {})\n\
             VSlot: {} (prefix: {})\n\
             EventHandler: {} (prefix: {})\n\
             Callback: {} (prefix: {})\n\
             ScriptSetup: {} (prefix: {})\n\
             NonScriptSetup: {} (prefix: {})\n\
             Universal: {} (prefix: {})\n\
             ClientOnly: {} (prefix: {})\n\
             JsGlobalUniversal: {} (prefix: {})\n\
             JsGlobalBrowser: {} (prefix: {})\n\
             JsGlobalNode: {} (prefix: {})\n\
             VueGlobal: {} (prefix: {})\n\
             ExternalModule: {} (prefix: {})\n\
             Closure: {} (prefix: {})",
                ScopeKind::Module.to_display(),
                ScopeKind::Module.prefix(),
                ScopeKind::Function.to_display(),
                ScopeKind::Function.prefix(),
                ScopeKind::Block.to_display(),
                ScopeKind::Block.prefix(),
                ScopeKind::VFor.to_display(),
                ScopeKind::VFor.prefix(),
                ScopeKind::VSlot.to_display(),
                ScopeKind::VSlot.prefix(),
                ScopeKind::EventHandler.to_display(),
                ScopeKind::EventHandler.prefix(),
                ScopeKind::Callback.to_display(),
                ScopeKind::Callback.prefix(),
                ScopeKind::ScriptSetup.to_display(),
                ScopeKind::ScriptSetup.prefix(),
                ScopeKind::NonScriptSetup.to_display(),
                ScopeKind::NonScriptSetup.prefix(),
                ScopeKind::Universal.to_display(),
                ScopeKind::Universal.prefix(),
                ScopeKind::ClientOnly.to_display(),
                ScopeKind::ClientOnly.prefix(),
                ScopeKind::JsGlobalUniversal.to_display(),
                ScopeKind::JsGlobalUniversal.prefix(),
                ScopeKind::JsGlobalBrowser.to_display(),
                ScopeKind::JsGlobalBrowser.prefix(),
                ScopeKind::JsGlobalNode.to_display(),
                ScopeKind::JsGlobalNode.prefix(),
                ScopeKind::VueGlobal.to_display(),
                ScopeKind::VueGlobal.prefix(),
                ScopeKind::ExternalModule.to_display(),
                ScopeKind::ExternalModule.prefix(),
                ScopeKind::Closure.to_display(),
                ScopeKind::Closure.prefix(),
            )
        );
    }

    #[test]
    #[allow(clippy::disallowed_macros)]
    fn test_block_kind_display() {
        assert_snapshot!(
            "block_kind_display",
            format!(
                "Block: {}\n\
             If: {}\n\
             Else: {}\n\
             For: {}\n\
             ForIn: {}\n\
             ForOf: {}\n\
             While: {}\n\
             DoWhile: {}\n\
             Switch: {}\n\
             Try: {}\n\
             Catch: {}\n\
             Finally: {}\n\
             With: {}",
                BlockKind::Block.as_str(),
                BlockKind::If.as_str(),
                BlockKind::Else.as_str(),
                BlockKind::For.as_str(),
                BlockKind::ForIn.as_str(),
                BlockKind::ForOf.as_str(),
                BlockKind::While.as_str(),
                BlockKind::DoWhile.as_str(),
                BlockKind::Switch.as_str(),
                BlockKind::Try.as_str(),
                BlockKind::Catch.as_str(),
                BlockKind::Finally.as_str(),
                BlockKind::With.as_str(),
            )
        );
    }

    #[test]
    fn test_js_runtime_conversions() {
        assert_eq!(
            JsRuntime::Universal.to_scope_kind(),
            ScopeKind::JsGlobalUniversal
        );
        assert_eq!(
            JsRuntime::Browser.to_scope_kind(),
            ScopeKind::JsGlobalBrowser
        );
        assert_eq!(JsRuntime::Node.to_scope_kind(), ScopeKind::JsGlobalNode);
        assert_eq!(JsRuntime::Deno.to_scope_kind(), ScopeKind::JsGlobalDeno);
        assert_eq!(JsRuntime::Bun.to_scope_kind(), ScopeKind::JsGlobalBun);

        assert_eq!(
            JsRuntime::Universal.to_binding_type(),
            BindingType::JsGlobalUniversal
        );
        assert_eq!(
            JsRuntime::Browser.to_binding_type(),
            BindingType::JsGlobalBrowser
        );
        assert_eq!(JsRuntime::Node.to_binding_type(), BindingType::JsGlobalNode);
    }

    #[test]
    fn test_binding_flags() {
        let mut binding = ScopeBinding::new(BindingType::SetupRef, 0);
        assert!(!binding.is_used());
        assert!(!binding.is_mutated());

        binding.mark_used();
        assert!(binding.is_used());
        assert!(!binding.is_mutated());

        binding.mark_mutated();
        assert!(binding.is_used());
        assert!(binding.is_mutated());
    }

    #[test]
    fn test_span() {
        let span = Span::new(10, 50);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 50);

        let default_span = Span::default();
        assert_eq!(default_span.start, 0);
        assert_eq!(default_span.end, 0);
    }

    #[test]
    fn test_scope_data_default() {
        let data = ScopeData::default();
        assert!(matches!(data, ScopeData::None));
    }
}
