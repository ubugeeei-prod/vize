//! Scope data structures for different scope types.
//!
//! Each scope kind can carry additional data specific to its context:
//! - `VForScopeData` - v-for iteration variables and source
//! - `VSlotScopeData` - slot name and props
//! - `EventHandlerScopeData` - event name, parameters, handler expression
//! - `CallbackScopeData` - callback parameters
//! - `ScriptSetupScopeData` / `NonScriptSetupScopeData` - script block metadata
//! - `ClientOnlyScopeData` / `UniversalScopeData` - lifecycle / SSR context
//! - `JsGlobalScopeData` / `VueGlobalScopeData` - global bindings
//! - `ExternalModuleScopeData` - imported module info
//! - `ClosureScopeData` - function/arrow parameters and flags
//! - `BlockScopeData` - block statement kind

use vize_carton::CompactString;
use vize_relief::BindingType;

use super::{ParamNames, ScopeKind};

/// Runtime environment for JavaScript globals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JsRuntime {
    /// Universal - works in all runtimes (console, Math, Object, Array, JSON, etc.)
    Universal = 0,
    /// Browser - window, document, navigator, localStorage, etc.
    Browser = 1,
    /// Node.js - process, Buffer, __dirname, __filename, require, etc.
    Node = 2,
    /// Deno - Deno namespace
    Deno = 3,
    /// Bun - Bun namespace
    Bun = 4,
}

impl JsRuntime {
    /// Get the corresponding ScopeKind for this runtime
    #[inline]
    pub const fn to_scope_kind(self) -> ScopeKind {
        match self {
            JsRuntime::Universal => ScopeKind::JsGlobalUniversal,
            JsRuntime::Browser => ScopeKind::JsGlobalBrowser,
            JsRuntime::Node => ScopeKind::JsGlobalNode,
            JsRuntime::Deno => ScopeKind::JsGlobalDeno,
            JsRuntime::Bun => ScopeKind::JsGlobalBun,
        }
    }

    /// Get the corresponding BindingType for this runtime
    #[inline]
    pub const fn to_binding_type(self) -> BindingType {
        match self {
            JsRuntime::Universal => BindingType::JsGlobalUniversal,
            JsRuntime::Browser => BindingType::JsGlobalBrowser,
            JsRuntime::Node => BindingType::JsGlobalNode,
            JsRuntime::Deno => BindingType::JsGlobalDeno,
            JsRuntime::Bun => BindingType::JsGlobalBun,
        }
    }
}

/// Block kind for block scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockKind {
    Block,
    If,
    Else,
    For,
    ForIn,
    ForOf,
    While,
    DoWhile,
    Switch,
    Try,
    Catch,
    Finally,
    With,
}

impl BlockKind {
    /// Get the display name for this block kind
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::ForIn => "for-in",
            Self::ForOf => "for-of",
            Self::While => "while",
            Self::DoWhile => "do-while",
            Self::Switch => "switch",
            Self::Try => "try",
            Self::Catch => "catch",
            Self::Finally => "finally",
            Self::With => "with",
        }
    }
}

/// Data specific to v-for scope
#[derive(Debug, Clone)]
pub struct VForScopeData {
    /// The value alias (e.g., "item" in v-for="item in items")
    pub value_alias: CompactString,
    /// The key alias (e.g., "key" in v-for="(item, key) in items")
    pub key_alias: Option<CompactString>,
    /// The index alias (e.g., "index" in v-for="(item, index) in items")
    pub index_alias: Option<CompactString>,
    /// The source expression (e.g., "items")
    pub source: CompactString,
    /// The :key expression if present (e.g., "item.id")
    pub key_expression: Option<CompactString>,
}

/// Data specific to v-slot scope
#[derive(Debug, Clone)]
pub struct VSlotScopeData {
    /// Slot name
    pub name: CompactString,
    /// Props pattern (e.g., "{ item, index }" in v-slot="{ item, index }")
    pub props_pattern: Option<CompactString>,
    /// Extracted prop names (stack-allocated for typical cases)
    pub prop_names: ParamNames,
}

/// Data specific to event handler scope
#[derive(Debug, Clone)]
pub struct EventHandlerScopeData {
    /// Event name (e.g., "click")
    pub event_name: CompactString,
    /// Whether this handler has implicit $event
    pub has_implicit_event: bool,
    /// Explicit parameter names (stack-allocated for typical cases)
    pub param_names: ParamNames,
    /// The handler expression (e.g., "handleClick" or "handleClick($event)")
    pub handler_expression: Option<CompactString>,
    /// Target component name (for component custom events, e.g., "TodoItem")
    /// None for DOM element events
    pub target_component: Option<CompactString>,
}

/// Data specific to callback scope
#[derive(Debug, Clone)]
pub struct CallbackScopeData {
    /// Parameter names (stack-allocated for typical cases)
    pub param_names: ParamNames,
    /// Context description (for debugging)
    pub context: CompactString,
}

/// Data specific to script setup scope
#[derive(Debug, Clone)]
pub struct ScriptSetupScopeData {
    /// Whether this is TypeScript
    pub is_ts: bool,
    /// Whether async setup
    pub is_async: bool,
    /// Generic type parameter from `<script setup generic="T">`
    pub generic: Option<CompactString>,
}

/// Data specific to non-script-setup scope (Options API, regular script)
#[derive(Debug, Clone)]
pub struct NonScriptSetupScopeData {
    /// Whether this is TypeScript
    pub is_ts: bool,
    /// Whether using defineComponent
    pub has_define_component: bool,
}

/// Data specific to client-only scope (onMounted, onBeforeUnmount, etc.)
#[derive(Debug, Clone)]
pub struct ClientOnlyScopeData {
    /// The lifecycle hook name (e.g., "onMounted", "onBeforeUnmount")
    pub hook_name: CompactString,
}

/// Data specific to universal scope (SSR - runs on both server and client)
#[derive(Debug, Clone)]
pub struct UniversalScopeData {
    /// Context description
    pub context: CompactString,
}

/// Data specific to JavaScript global scope
#[derive(Debug, Clone)]
pub struct JsGlobalScopeData {
    /// Runtime environment
    pub runtime: JsRuntime,
    /// Known JS globals for this runtime
    pub globals: ParamNames,
}

/// Data specific to Vue global scope
#[derive(Debug, Clone)]
pub struct VueGlobalScopeData {
    /// Known Vue globals ($refs, $emit, $slots, $attrs, $el, etc.)
    pub globals: ParamNames,
}

/// Data specific to external module scope
#[derive(Debug, Clone)]
pub struct ExternalModuleScopeData {
    /// Module source path
    pub source: CompactString,
    /// Whether this is a type-only import
    pub is_type_only: bool,
}

/// Data specific to closure scope (function declaration, function expression, arrow function)
#[derive(Debug, Clone)]
pub struct ClosureScopeData {
    /// Function name (if named function)
    pub name: Option<CompactString>,
    /// Parameter names
    pub param_names: ParamNames,
    /// Whether this is an arrow function (no `arguments`, no `this` binding)
    pub is_arrow: bool,
    /// Whether this is async
    pub is_async: bool,
    /// Whether this is a generator
    pub is_generator: bool,
}

/// Data specific to block scope (if, for, switch, etc.)
#[derive(Debug, Clone, Copy)]
pub struct BlockScopeData {
    /// Block kind
    pub kind: BlockKind,
}

/// Scope-specific data
#[derive(Debug, Clone)]
pub enum ScopeData {
    /// No additional data
    None,
    /// v-for specific data
    VFor(VForScopeData),
    /// v-slot specific data
    VSlot(VSlotScopeData),
    /// Event handler specific data
    EventHandler(EventHandlerScopeData),
    /// Callback specific data
    Callback(CallbackScopeData),
    /// Script setup specific data
    ScriptSetup(ScriptSetupScopeData),
    /// Non-script-setup specific data
    NonScriptSetup(NonScriptSetupScopeData),
    /// Client-only specific data
    ClientOnly(ClientOnlyScopeData),
    /// Universal scope specific data
    Universal(UniversalScopeData),
    /// JavaScript global specific data (with runtime info)
    JsGlobal(JsGlobalScopeData),
    /// Vue global specific data
    VueGlobal(VueGlobalScopeData),
    /// External module specific data
    ExternalModule(ExternalModuleScopeData),
    /// Closure specific data
    Closure(ClosureScopeData),
    /// Block specific data
    Block(BlockScopeData),
}

impl Default for ScopeData {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}
