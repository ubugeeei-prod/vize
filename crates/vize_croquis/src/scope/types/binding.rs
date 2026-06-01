//! Binding tracking types for scope analysis.
//!
//! Contains the types used to track individual bindings within scopes:
//! - `BindingFlags` - Bitflags for tracking usage and mutation
//! - `ScopeBinding` - A binding within a scope with its type, location, and flags
//! - `Span` - Source location span

use vize_carton::bitflags;
use vize_relief::BindingType;

/// Source span
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline(always)]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn shift(&mut self, delta: u32) {
        self.start = self.start.saturating_add(delta);
        self.end = self.end.saturating_add(delta);
    }

    /// Returns true when `offset` falls within `[start, end]`, inclusive.
    ///
    /// Cursor positions live between bytes, so a cursor sitting at exactly
    /// `end` is still considered "inside" the span. Default spans (start=end=0)
    /// only contain offset 0.
    #[inline]
    pub const fn contains(&self, offset: u32) -> bool {
        self.start <= offset && offset <= self.end
    }

    /// Span length in bytes.
    #[inline]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// True when the span has zero length (default or unset spans).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

bitflags! {
    /// Binding flags for tracking usage and mutation
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct BindingFlags: u8 {
        /// Binding has been referenced
        const USED = 1 << 0;
        /// Binding has been mutated
        const MUTATED = 1 << 1;
        /// Binding is a rest parameter
        const REST = 1 << 2;
        /// Binding has a default value
        const HAS_DEFAULT = 1 << 3;
    }
}

/// A binding within a scope
#[derive(Debug, Clone, Copy)]
pub struct ScopeBinding {
    /// The type of binding
    pub binding_type: BindingType,
    /// Source location of the declaration (offset in source)
    pub declaration_offset: u32,
    /// Binding flags
    flags: BindingFlags,
}

impl ScopeBinding {
    /// Create a new scope binding
    #[inline]
    pub const fn new(binding_type: BindingType, declaration_offset: u32) -> Self {
        Self {
            binding_type,
            declaration_offset,
            flags: BindingFlags::empty(),
        }
    }

    /// Check if binding is used
    #[inline]
    pub const fn is_used(&self) -> bool {
        self.flags.contains(BindingFlags::USED)
    }

    /// Check if binding is mutated
    #[inline]
    pub const fn is_mutated(&self) -> bool {
        self.flags.contains(BindingFlags::MUTATED)
    }

    /// Mark as used
    #[inline]
    pub fn mark_used(&mut self) {
        self.flags.insert(BindingFlags::USED);
    }

    /// Mark as mutated
    #[inline]
    pub fn mark_mutated(&mut self) {
        self.flags.insert(BindingFlags::MUTATED);
    }

    #[inline]
    pub fn shift_declaration_offset(&mut self, delta: u32) {
        self.declaration_offset = self.declaration_offset.saturating_add(delta);
    }
}
