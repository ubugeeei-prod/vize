//! Scope chain management for Vue templates and scripts.
//!
//! - [`Scope`] - A single scope in the scope chain
//! - [`ScopeChain`] - Manages the hierarchical scope chain
//!
//! - [`builder`]: Methods for entering/creating scopes
//! - [`resolution`]: Binding lookup, mutation tracking, and depth computation

mod access;
mod builder;
mod resolution;
mod template_offsets;
mod visibility;
use core::fmt;
use index_vec::IndexVec;
use std::borrow::Cow;

use vize_carton::{
    CompactString, FxHashMap, FxHashSet, SmallVec, String, ToCompactString, smallvec,
};
use vize_relief::BindingType;

use super::globals::{BindingMap, StandardGlobals, UNIVERSAL};
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
    /// Bindings declared in this scope. A standard global scope borrows its
    /// shared table until the first write.
    bindings: Cow<'static, BindingMap>,
    /// Scope-specific data
    data: Cow<'static, ScopeData>,
    /// Source span
    pub span: Span,
}

struct SortedBindings<'a>(&'a BindingMap);

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
            bindings: Cow::Owned(BindingMap::default()),
            data: Cow::Owned(ScopeData::None),
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
            bindings: Cow::Owned(BindingMap::default()),
            data: Cow::Owned(ScopeData::None),
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
            bindings: Cow::Owned(BindingMap::default()),
            data: Cow::Owned(ScopeData::None),
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
            bindings: Cow::Owned(BindingMap::default()),
            data: Cow::Owned(ScopeData::None),
            span: Span::new(start, end),
        }
    }

    /// A standard global scope borrowing its shared table.
    #[inline]
    pub(crate) fn standard(
        id: ScopeId,
        parent: Option<ScopeId>,
        globals: &'static StandardGlobals,
    ) -> Self {
        Self {
            id,
            parents: parent.map(|p| smallvec![p]).unwrap_or_default(),
            kind: globals.kind,
            bindings: Cow::Borrowed(&globals.bindings),
            data: Cow::Borrowed(&globals.data),
            span: Span::default(),
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
        self.data = Cow::Owned(data);
    }

    /// Get scope-specific data
    #[inline]
    pub fn data(&self) -> &ScopeData {
        &self.data
    }

    /// Add a binding to this scope
    #[inline]
    pub fn add_binding(&mut self, name: CompactString, binding: ScopeBinding) {
        self.bindings.to_mut().insert(name, binding);
    }

    /// Get a binding by name (only in this scope, not parents)
    #[inline]
    pub fn get_binding(&self, name: &str) -> Option<&ScopeBinding> {
        self.bindings.get(name)
    }

    /// Get a mutable binding by name
    #[inline]
    pub fn get_binding_mut(&mut self, name: &str) -> Option<&mut ScopeBinding> {
        match &mut self.bindings {
            Cow::Owned(bindings) => bindings.get_mut(name),
            // A miss leaves the shared table borrowed.
            Cow::Borrowed(bindings) if !bindings.contains_key(name) => None,
            bindings => bindings.to_mut().get_mut(name),
        }
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
        match (&self.kind, self.data()) {
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
    pub(crate) scopes: IndexVec<ScopeId, Scope>,
    pub(crate) current: ScopeId,
    /// v-slot scopes whose directive argument was dynamic (`v-slot:[name]`).
    dynamic_v_slot_scopes: FxHashSet<ScopeId>,
    directive_expression_offsets: FxHashMap<ScopeId, u32>,
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

impl ScopeChain {
    /// Create a new scope chain with JS universal globals as root
    /// ECMAScript standard built-ins only (ECMA-262)
    #[inline]
    pub fn new() -> Self {
        Self::with_root(IndexVec::from_vec(vec![Self::root()]))
    }

    /// Create with pre-allocated capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut scopes = IndexVec::with_capacity(capacity);
        scopes.push(Self::root());
        Self::with_root(scopes)
    }

    fn root() -> Scope {
        Scope::standard(ScopeId::ROOT, None, &UNIVERSAL)
    }

    fn with_root(scopes: IndexVec<ScopeId, Scope>) -> Self {
        Self {
            scopes,
            current: ScopeId::ROOT,
            dynamic_v_slot_scopes: FxHashSet::default(),
            directive_expression_offsets: FxHashMap::default(),
        }
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
            for binding in scope.bindings.to_mut().values_mut() {
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

    /// Get mutable scope by ID
    #[inline]
    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id)
    }

    /// Set the current scope directly (used for switching between sibling scopes)
    ///
    /// # Panics
    /// Panics before changing the current scope if the handle is out of range.
    #[inline]
    pub fn set_current(&mut self, id: ScopeId) {
        assert!(
            self.scopes.get(id).is_some(),
            "scope handle is out of range"
        );
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
mod standard_globals_tests;
#[cfg(test)]
mod tests;
