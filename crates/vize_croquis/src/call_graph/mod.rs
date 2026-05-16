//! Function call graph for tracking Vue API calls and composables.
//!
//! Tracks whether Vue APIs (ref, reactive, provide, inject, etc.) are called
//! within the setup context, either directly or through composable functions.
//!
//! ## Key Features
//!
//! - Tracks function definitions and their containing scopes
//! - Tracks Vue API calls (ref, reactive, computed, provide, inject, watch, etc.)
//! - Tracks composable function calls (use* pattern)
//! - Validates that Vue APIs are called within appropriate contexts
//!
//! ## Performance
//!
//! - Uses FxHashMap for O(1) lookups
//! - SmallVec for typical small collections
//! - Minimal allocations during analysis
//!
//! Split into:
//! - Core types and data structures (this file)
//! - [`builder`]: Graph construction and Vue API categorization
//! - [`analysis`]: Setup context propagation, queries, and markdown output

mod analysis;
mod builder;

use vize_carton::{CompactString, FxHashMap, FxHashSet, SmallVec};

use crate::scope::ScopeId;

// Re-export free functions from builder
pub use builder::{is_composable_name, is_vue_api};

/// Unique identifier for a function in the call graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FunctionId(u32);

impl FunctionId {
    /// Create a new function ID.
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value.
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Category of Vue API function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VueApiCategory {
    /// Reactivity primitives: ref, reactive, computed, etc.
    Reactivity,
    /// Lifecycle hooks: onMounted, onUnmounted, etc.
    Lifecycle,
    /// Dependency injection: provide, inject
    DependencyInjection,
    /// Watchers: watch, watchEffect, watchPostEffect, watchSyncEffect
    Watcher,
    /// Template refs: useTemplateRef
    TemplateRef,
    /// Other Vue APIs: nextTick, defineComponent, etc.
    Other,
}

/// A Vue API call detected in the code.
#[derive(Debug, Clone)]
pub struct VueApiCall {
    /// Name of the API function (e.g., "ref", "provide", "onMounted").
    pub name: CompactString,
    /// Category of the API.
    pub category: VueApiCategory,
    /// Scope where this call occurs.
    pub scope_id: ScopeId,
    /// Containing function (if inside a function).
    pub containing_function: Option<FunctionId>,
    /// Whether this call is inside the setup context (directly or transitively).
    pub in_setup_context: bool,
    /// Source offset.
    pub start: u32,
    pub end: u32,
}

/// A function definition in the code.
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function ID.
    pub id: FunctionId,
    /// Function name (None for anonymous functions).
    pub name: Option<CompactString>,
    /// Scope where this function is defined.
    pub scope_id: ScopeId,
    /// Parent function (if nested).
    pub parent_function: Option<FunctionId>,
    /// Whether this is an arrow function.
    pub is_arrow: bool,
    /// Whether this function is called within setup context.
    pub called_in_setup: bool,
    /// Whether this function uses Vue APIs.
    pub uses_vue_apis: bool,
    /// Whether this is a composable (use* pattern and uses Vue APIs).
    pub is_composable: bool,
    /// Source offset.
    pub start: u32,
    pub end: u32,
}

/// A composable function call (use* pattern).
#[derive(Debug, Clone)]
pub struct ComposableCallInfo {
    /// Name of the composable function.
    pub name: CompactString,
    /// Import source (if imported).
    pub source: Option<CompactString>,
    /// Scope where this call occurs.
    pub scope_id: ScopeId,
    /// Containing function (if inside a function).
    pub containing_function: Option<FunctionId>,
    /// Whether this composable is called in setup context.
    pub in_setup_context: bool,
    /// Local binding name (if assigned).
    pub local_binding: Option<CompactString>,
    /// Vue APIs used by this composable (if known from analysis).
    pub vue_apis_used: SmallVec<[CompactString; 4]>,
    /// Source offset.
    pub start: u32,
    pub end: u32,
}

/// Call edge between two functions.
#[derive(Debug, Clone)]
pub struct CallEdge {
    /// Caller function.
    pub caller: FunctionId,
    /// Callee function.
    pub callee: FunctionId,
    /// Call site offset.
    pub call_site: u32,
}

/// Kind of setup context entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SetupContextKind {
    /// Direct setup function body (`<script setup>` or `setup()` function).
    SetupBody,
    /// Inside a composable called from setup.
    Composable,
    /// Inside a callback passed to a composable (e.g., computed callback).
    ComposableCallback,
    /// Not in setup context.
    None,
}

/// Tracks function calls and Vue API usage.
#[derive(Debug, Default)]
pub struct CallGraph {
    /// All function definitions.
    pub(crate) functions: Vec<FunctionDef>,
    /// Vue API calls.
    pub(crate) vue_api_calls: Vec<VueApiCall>,
    /// Composable calls.
    pub(crate) composable_calls: Vec<ComposableCallInfo>,
    /// Call edges between functions.
    pub(crate) call_edges: Vec<CallEdge>,
    /// Map from function name to function IDs (for resolution).
    pub(crate) function_by_name: FxHashMap<CompactString, SmallVec<[FunctionId; 2]>>,
    /// Functions that are part of setup context (directly or transitively).
    pub(crate) setup_context_functions: FxHashSet<FunctionId>,
    /// The setup function ID (if found).
    pub(crate) setup_function: Option<FunctionId>,
    /// Next function ID.
    pub(crate) next_id: u32,
}
