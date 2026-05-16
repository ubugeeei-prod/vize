//! Types representing lint context state.
//!
//! Provides `DisabledRange`, `SsrMode`, and `ElementContext` which are used
//! to track rule suppression, SSR linting mode, and element traversal state.

use vize_carton::CompactString;

/// Represents a disabled range for a specific rule or all rules.
#[derive(Debug, Clone)]
pub struct DisabledRange {
    /// Start line (1-indexed).
    pub start_line: u32,
    /// End line (1-indexed, inclusive). None means until end of file.
    pub end_line: Option<u32>,
}

/// SSR mode for linting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsrMode {
    /// Disabled - no SSR-specific rules.
    Disabled,
    /// Enabled - warn about SSR-unfriendly code (default).
    #[default]
    Enabled,
}

/// Context for tracking element state during traversal.
///
/// Uses `CompactString` for tag to avoid lifetime complications while
/// maintaining efficiency for small strings (inline storage up to 24 bytes).
#[derive(Debug, Clone)]
pub struct ElementContext {
    /// Tag name (CompactString for efficiency).
    pub tag: CompactString,
    /// Whether element has v-for directive.
    pub has_v_for: bool,
    /// Whether element has v-if directive.
    pub has_v_if: bool,
    /// Variables defined by v-for on this element.
    pub v_for_vars: Vec<CompactString>,
}

impl ElementContext {
    /// Create a new element context.
    #[inline]
    pub fn new(tag: impl Into<CompactString>) -> Self {
        Self {
            tag: tag.into(),
            has_v_for: false,
            has_v_if: false,
            v_for_vars: Vec::new(),
        }
    }

    /// Create with v-for info.
    #[inline]
    pub fn with_v_for(tag: impl Into<CompactString>, vars: Vec<CompactString>) -> Self {
        Self {
            tag: tag.into(),
            has_v_for: true,
            has_v_if: false,
            v_for_vars: vars,
        }
    }
}
