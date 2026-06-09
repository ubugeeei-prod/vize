//! Scope chain management for Vue templates and scripts.
//!
//! This module provides the core scope management functionality:
//! - [`Scope`] - A single scope in the scope chain
//! - [`ScopeChain`] - Manages the hierarchical scope chain
//!
//! Split into:
//! - Core types, `Scope`, and `ScopeChain` struct with basic accessors (this file)
//! - [`builder`]: Methods for entering/creating scopes
//! - [`resolution`]: Binding lookup, mutation tracking, and depth computation

mod builder;
mod resolution;

use core::fmt;

use vize_carton::{
    CompactString, FxHashMap, FxHashSet, SmallVec, String, ToCompactString, smallvec,
};
use vize_relief::BindingType;

use super::types::{
    BlockScopeData, CallbackScopeData, ClientOnlyScopeData, ClosureScopeData,
    EventHandlerScopeData, ExternalModuleScopeData, JsGlobalScopeData, NonScriptSetupScopeData,
    ParentScopes, ScopeBinding, ScopeData, ScopeId, ScopeKind, ScriptSetupScopeData, Span,
    UniversalScopeData, VForScopeData, VSlotScopeData, VueGlobalScopeData,
};

/// A single scope in the scope chain
pub struct Scope {
    /// Unique identifier
    pub id: ScopeId,
    /// Parent scopes (empty for root, can have multiple for template scopes)
    /// First parent is the lexical parent, additional parents are accessible scopes (e.g., Vue globals)
    pub parents: ParentScopes,
    /// Kind of scope
    pub kind: ScopeKind,
    /// Bindings declared in this scope
    bindings: FxHashMap<CompactString, ScopeBinding>,
    /// Scope-specific data
    data: ScopeData,
    /// Source span
    pub span: Span,
}

struct SortedBindings<'a>(&'a FxHashMap<CompactString, ScopeBinding>);

impl fmt::Debug for SortedBindings<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut entries: SmallVec<[(&CompactString, &ScopeBinding); 64]> = self.0.iter().collect();
        entries.sort_unstable_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));

        f.debug_map()
            .entries(
                entries
                    .iter()
                    .map(|(name, binding)| (name.as_str(), *binding)),
            )
            .finish()
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field("id", &self.id)
            .field("parents", &self.parents)
            .field("kind", &self.kind)
            .field("bindings", &SortedBindings(&self.bindings))
            .field("data", &self.data)
            .field("span", &self.span)
            .finish()
    }
}

impl Scope {
    /// Create a new scope with single parent
    #[inline]
    pub fn new(id: ScopeId, parent: Option<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            id,
            parents: parent.map(|p| smallvec![p]).unwrap_or_default(),
            kind,
            bindings: FxHashMap::default(),
            data: ScopeData::None,
            span: Span::default(),
        }
    }

    /// Create a new scope with multiple parents
    #[inline]
    pub fn with_parents(id: ScopeId, parents: ParentScopes, kind: ScopeKind) -> Self {
        Self {
            id,
            parents,
            kind,
            bindings: FxHashMap::default(),
            data: ScopeData::None,
            span: Span::default(),
        }
    }

    /// Create a new scope with span
    #[inline]
    pub fn with_span(
        id: ScopeId,
        parent: Option<ScopeId>,
        kind: ScopeKind,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            id,
            parents: parent.map(|p| smallvec![p]).unwrap_or_default(),
            kind,
            bindings: FxHashMap::default(),
            data: ScopeData::None,
            span: Span::new(start, end),
        }
    }

    /// Create a new scope with span and multiple parents
    #[inline]
    pub fn with_span_parents(
        id: ScopeId,
        parents: ParentScopes,
        kind: ScopeKind,
        start: u32,
        end: u32,
    ) -> Self {
        Self {
            id,
            parents,
            kind,
            bindings: FxHashMap::default(),
            data: ScopeData::None,
            span: Span::new(start, end),
        }
    }

    /// Get the primary (lexical) parent
    #[inline]
    pub fn parent(&self) -> Option<ScopeId> {
        self.parents.first().copied()
    }

    /// Add an additional parent scope
    #[inline]
    pub fn add_parent(&mut self, parent: ScopeId) {
        if !self.parents.contains(&parent) {
            self.parents.push(parent);
        }
    }

    /// Set scope-specific data
    #[inline]
    pub fn set_data(&mut self, data: ScopeData) {
        self.data = data;
    }

    /// Get scope-specific data
    #[inline]
    pub fn data(&self) -> &ScopeData {
        &self.data
    }

    /// Add a binding to this scope
    #[inline]
    pub fn add_binding(&mut self, name: CompactString, binding: ScopeBinding) {
        self.bindings.insert(name, binding);
    }

    /// Get a binding by name (only in this scope, not parents)
    #[inline]
    pub fn get_binding(&self, name: &str) -> Option<&ScopeBinding> {
        self.bindings.get(name)
    }

    /// Get a mutable binding by name
    #[inline]
    pub fn get_binding_mut(&mut self, name: &str) -> Option<&mut ScopeBinding> {
        self.bindings.get_mut(name)
    }

    /// Check if this scope has a binding
    #[inline]
    pub fn has_binding(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Iterate over all bindings in this scope
    #[inline]
    pub fn bindings(&self) -> impl Iterator<Item = (&str, &ScopeBinding)> {
        self.bindings.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Number of bindings in this scope
    #[inline]
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Get display name for this scope (includes hook name for ClientOnly scopes)
    pub fn display_name(&self) -> String {
        match (&self.kind, &self.data) {
            (ScopeKind::ClientOnly, ScopeData::ClientOnly(data)) => {
                // Use hook name without "on" prefix: onMounted -> mounted
                data.hook_name
                    .strip_prefix("on")
                    .map(|s| String::from(s.to_ascii_lowercase().as_str()))
                    .unwrap_or_else(|| data.hook_name.clone())
            }
            _ => self.kind.to_display().to_compact_string(),
        }
    }
}

/// Manages the scope chain during analysis
pub struct ScopeChain {
    /// All scopes (indexed by ScopeId)
    pub(crate) scopes: Vec<Scope>,
    /// Current scope ID
    pub(crate) current: ScopeId,
    /// v-slot scopes whose directive argument was dynamic (`v-slot:[name]`).
    dynamic_v_slot_scopes: FxHashSet<ScopeId>,
}

impl fmt::Debug for ScopeChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopeChain")
            .field("scopes", &self.scopes)
            .field("current", &self.current)
            .finish()
    }
}

impl Default for ScopeChain {
    fn default() -> Self {
        Self::new()
    }
}

/// ECMAScript standard built-in globals (ECMA-262)
const JS_UNIVERSAL_GLOBALS: &[&str] = &[
    "AggregateError",
    "arguments", // Function scope closure
    "Array",
    "ArrayBuffer",
    "AsyncFunction",
    "AsyncGenerator",
    "AsyncGeneratorFunction",
    "AsyncIterator",
    "Atomics",
    "BigInt",
    "BigInt64Array",
    "BigUint64Array",
    "Boolean",
    "console", // Non-standard but universally available
    "DataView",
    "Date",
    "decodeURI",
    "decodeURIComponent",
    "encodeURI",
    "encodeURIComponent",
    "Error",
    "eval",
    "EvalError",
    "Float32Array",
    "Float64Array",
    "Function",
    "Generator",
    "GeneratorFunction",
    "globalThis",
    "Infinity",
    "Int16Array",
    "Int32Array",
    "Int8Array",
    "Intl",
    "isFinite",
    "isNaN",
    "Iterator",
    "JSON",
    "Map",
    "Math",
    "NaN",
    "Number",
    "Object",
    "parseFloat",
    "parseInt",
    "Promise",
    "Proxy",
    "RangeError",
    "ReferenceError",
    "Reflect",
    "RegExp",
    "Set",
    "SharedArrayBuffer",
    "String",
    "Symbol",
    "SyntaxError",
    "this", // Function scope closure
    "TypeError",
    "Uint16Array",
    "Uint32Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "undefined",
    "URIError",
    "WeakMap",
    "WeakSet",
];

impl ScopeChain {
    /// Create a new scope chain with JS universal globals as root
    /// ECMAScript standard built-ins only (ECMA-262)
    #[inline]
    pub fn new() -> Self {
        let mut root = Scope::new(ScopeId::ROOT, None, ScopeKind::JsGlobalUniversal);
        for name in JS_UNIVERSAL_GLOBALS {
            root.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::JsGlobalUniversal, 0),
            );
        }
        Self {
            scopes: vec![root],
            current: ScopeId::ROOT,
            dynamic_v_slot_scopes: FxHashSet::default(),
        }
    }

    /// Create with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut root = Scope::new(ScopeId::ROOT, None, ScopeKind::JsGlobalUniversal);
        for name in JS_UNIVERSAL_GLOBALS {
            root.add_binding(
                CompactString::new(name),
                ScopeBinding::new(BindingType::JsGlobalUniversal, 0),
            );
        }
        let mut scopes = Vec::with_capacity(capacity);
        scopes.push(root);
        Self {
            scopes,
            current: ScopeId::ROOT,
            dynamic_v_slot_scopes: FxHashSet::default(),
        }
    }

    /// Get the current scope
    #[inline]
    pub fn current_scope(&self) -> &Scope {
        // SAFETY: `current` is initialized to `ROOT`, and every scope transition
        // writes an id returned by `push_scope`/`enter_scope`, both of which append
        // to `self.scopes` before exposing the id. Exiting a scope moves to a
        // stored parent id, never an arbitrary index. This unchecked access is on
        // every identifier lookup path, so we keep the invariant centralized here.
        unsafe { self.scopes.get_unchecked(self.current.as_u32() as usize) }
    }

    /// Get the current scope mutably
    #[inline]
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        let idx = self.current.as_u32() as usize;
        // SAFETY: same invariant as `current_scope`: `idx` comes from a
        // ScopeId minted by this chain and therefore addresses an existing scope.
        // The `&mut self` receiver guarantees no competing borrow of the scope
        // vector while returning this mutable reference.
        unsafe { self.scopes.get_unchecked_mut(idx) }
    }

    /// Get a scope by ID
    #[inline]
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.as_u32() as usize)
    }

    /// Get a scope by ID (unchecked)
    ///
    /// # Safety
    /// Caller must ensure `id` was produced by this `ScopeChain` and the chain has
    /// not been rebuilt since. The method exists for analyzer hot paths where the
    /// caller already proved the id through registry traversal.
    #[inline]
    pub unsafe fn get_scope_unchecked(&self, id: ScopeId) -> &Scope {
        // SAFETY: upheld by the caller contract above.
        unsafe { self.scopes.get_unchecked(id.as_u32() as usize) }
    }

    /// Current scope ID
    #[inline]
    pub const fn current_id(&self) -> ScopeId {
        self.current
    }

    /// Whether a v-slot scope name came from a static directive argument.
    #[inline]
    pub fn is_v_slot_name_static(&self, id: ScopeId) -> bool {
        !self.dynamic_v_slot_scopes.contains(&id)
    }

    /// Number of scopes
    #[inline]
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Check if empty (only root scope)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scopes.len() == 1
    }

    /// Iterate over all scopes
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter()
    }

    /// Shift script-local offsets after embedding a script analysis into a
    /// larger synthetic script. Template scopes should be added after this.
    pub fn shift_script_offsets(&mut self, delta: u32) {
        for scope in &mut self.scopes {
            if matches!(
                scope.kind,
                ScopeKind::JsGlobalUniversal
                    | ScopeKind::JsGlobalBrowser
                    | ScopeKind::JsGlobalNode
                    | ScopeKind::JsGlobalDeno
                    | ScopeKind::JsGlobalBun
                    | ScopeKind::VueGlobal
            ) {
                continue;
            }

            scope.span.shift(delta);
            for binding in scope.bindings.values_mut() {
                binding.shift_declaration_offset(delta);
            }
        }
    }

    /// Find a scope by kind (returns the first match)
    #[inline]
    pub fn find_scope_by_kind(&self, kind: ScopeKind) -> Option<ScopeId> {
        self.scopes.iter().find(|s| s.kind == kind).map(|s| s.id)
    }

    /// Find the deepest scope whose span contains `offset`.
    ///
    /// Scopes are entered in depth-first order during analysis, so the deepest
    /// containing scope is the one with the highest id among those that
    /// contain the offset. Global scopes (universal, browser, vue) use the
    /// default span and are never matched here — they are reached only by
    /// walking the parent chain of the matched scope.
    pub fn scope_at_offset(&self, offset: u32) -> Option<ScopeId> {
        let mut best: Option<(ScopeId, u32)> = None;
        for scope in &self.scopes {
            if matches!(
                scope.kind,
                ScopeKind::JsGlobalUniversal
                    | ScopeKind::JsGlobalBrowser
                    | ScopeKind::JsGlobalNode
                    | ScopeKind::JsGlobalDeno
                    | ScopeKind::JsGlobalBun
                    | ScopeKind::VueGlobal
            ) {
                continue;
            }
            if scope.span.contains(offset) {
                // Smaller spans win (deepest); ties broken by higher id which
                // is created later in DFS traversal.
                let len = scope.span.len();
                let should_replace = match best {
                    None => true,
                    Some((_, current_len)) if len < current_len => true,
                    Some((current_id, current_len)) if len == current_len => {
                        scope.id.as_u32() > current_id.as_u32()
                    }
                    _ => false,
                };
                if should_replace {
                    best = Some((scope.id, len));
                }
            }
        }
        best.map(|(id, _)| id)
    }

    /// Collect every binding visible at `offset`, walking from the deepest
    /// containing scope outward through its parents.
    ///
    /// Inner-scope bindings shadow outer ones — the first occurrence of each
    /// name wins. Returns `(name, binding, scope_kind)` triples. The order is
    /// inner-most first, which the LSP uses to prioritize closer scopes in
    /// completion sort order.
    pub fn bindings_visible_at(&self, offset: u32) -> Vec<(&str, ScopeBinding, ScopeKind)> {
        let Some(start_id) = self.scope_at_offset(offset) else {
            return Vec::new();
        };

        let mut seen: FxHashSet<&str> = FxHashSet::default();
        let mut out: Vec<(&str, ScopeBinding, ScopeKind)> = Vec::new();
        let mut stack: Vec<ScopeId> = Vec::new();
        let mut to_visit: Vec<ScopeId> = vec![start_id];

        while let Some(id) = to_visit.pop() {
            // Avoid revisiting the same scope through multiple parent paths.
            if stack.contains(&id) {
                continue;
            }
            stack.push(id);

            let Some(scope) = self.get_scope(id) else {
                continue;
            };

            for (name, binding) in scope.bindings() {
                if seen.insert(name) {
                    out.push((name, *binding, scope.kind));
                }
            }

            for parent in scope.parents.iter().copied() {
                to_visit.push(parent);
            }
        }

        out
    }

    /// Get mutable scope by ID
    #[inline]
    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id.as_u32() as usize)
    }

    /// Set the current scope directly (used for switching between sibling scopes)
    #[inline]
    pub fn set_current(&mut self, id: ScopeId) {
        self.current = id;
    }

    /// Build parents list including Vue global for template scopes
    pub(crate) fn build_template_parents(&self) -> ParentScopes {
        let mut parents: ParentScopes = smallvec![self.current];
        if let Some(vue_id) = self.find_scope_by_kind(ScopeKind::VueGlobal)
            && !parents.contains(&vue_id)
        {
            parents.push(vue_id);
        }
        parents
    }
}

#[cfg(test)]
#[path = "chain/chain_tests.rs"]
mod tests;
