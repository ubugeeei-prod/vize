//! CSS analysis for Vue SFC styles.

use vize_carton::{CompactString, SmallVec};

/// Selector identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SelectorId(u32);

impl SelectorId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Kind of selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SelectorKind {
    /// Class selector (.foo)
    Class = 0,
    /// ID selector (#foo)
    Id = 1,
    /// Element selector (div)
    Element = 2,
    /// Attribute selector ([foo])
    Attribute = 3,
    /// Pseudo-class (:hover)
    PseudoClass = 4,
    /// Pseudo-element (::before)
    PseudoElement = 5,
    /// Combinator (>, +, ~, space)
    Combinator = 6,
}

/// Part of a selector
#[derive(Debug, Clone)]
pub struct SelectorPart {
    pub kind: SelectorKind,
    pub value: CompactString,
}

/// A CSS selector
#[derive(Debug, Clone)]
pub struct CssSelector {
    pub id: SelectorId,
    pub raw: CompactString,
    pub parts: SmallVec<[SelectorPart; 4]>,
    pub start: u32,
    pub end: u32,
}

/// CSS v-bind() usage
#[derive(Debug, Clone)]
pub struct CssVBind {
    pub property: CompactString,
    pub expression: CompactString,
    pub start: u32,
    pub end: u32,
}

/// CSS variable usage
#[derive(Debug, Clone)]
pub struct CssVariable {
    pub name: CompactString,
    pub value: Option<CompactString>,
    pub start: u32,
    pub end: u32,
}

/// Statistics about CSS analysis
#[derive(Debug, Clone, Default)]
pub struct CssStats {
    pub selector_count: u32,
    pub v_bind_count: u32,
    pub variable_count: u32,
    pub deep_selectors: u32,
    pub slotted_selectors: u32,
    pub global_selectors: u32,
}

/// Tracks CSS analysis during SFC compilation
#[derive(Debug, Default)]
pub struct CssTracker {
    selectors: Vec<CssSelector>,
    v_binds: Vec<CssVBind>,
    variables: Vec<CssVariable>,
    next_id: u32,
    deep_count: u32,
    slotted_count: u32,
    global_count: u32,
}

impl CssTracker {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a selector
    pub fn add_selector(
        &mut self,
        raw: CompactString,
        parts: SmallVec<[SelectorPart; 4]>,
        start: u32,
        end: u32,
    ) -> SelectorId {
        let id = SelectorId::new(self.next_id);
        self.next_id += 1;

        self.selectors.push(CssSelector {
            id,
            raw,
            parts,
            start,
            end,
        });

        id
    }

    /// Add a v-bind() usage
    pub fn add_v_bind(
        &mut self,
        property: CompactString,
        expression: CompactString,
        start: u32,
        end: u32,
    ) {
        self.v_binds.push(CssVBind {
            property,
            expression,
            start,
            end,
        });
    }

    /// Add a CSS variable
    pub fn add_variable(
        &mut self,
        name: CompactString,
        value: Option<CompactString>,
        start: u32,
        end: u32,
    ) {
        self.variables.push(CssVariable {
            name,
            value,
            start,
            end,
        });
    }

    /// Record :deep() usage
    #[inline]
    pub fn record_deep(&mut self) {
        self.deep_count += 1;
    }

    /// Record :slotted() usage
    #[inline]
    pub fn record_slotted(&mut self) {
        self.slotted_count += 1;
    }

    /// Record :global() usage
    #[inline]
    pub fn record_global(&mut self) {
        self.global_count += 1;
    }

    // Getters

    #[inline]
    pub fn selectors(&self) -> &[CssSelector] {
        &self.selectors
    }

    #[inline]
    pub fn v_binds(&self) -> &[CssVBind] {
        &self.v_binds
    }

    #[inline]
    pub fn variables(&self) -> &[CssVariable] {
        &self.variables
    }

    /// Get CSS statistics
    pub fn stats(&self) -> CssStats {
        CssStats {
            selector_count: self.selectors.len() as u32,
            v_bind_count: self.v_binds.len() as u32,
            variable_count: self.variables.len() as u32,
            deep_selectors: self.deep_count,
            slotted_selectors: self.slotted_count,
            global_selectors: self.global_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CssTracker, SelectorKind, SelectorPart};
    use vize_carton::CompactString;

    #[test]
    fn test_css_tracker() {
        let mut tracker = CssTracker::new();

        tracker.add_selector(
            CompactString::new(".btn"),
            vize_carton::smallvec![SelectorPart {
                kind: SelectorKind::Class,
                value: CompactString::new("btn"),
            }],
            0,
            10,
        );

        tracker.add_v_bind(
            CompactString::new("color"),
            CompactString::new("theme.color"),
            20,
            40,
        );

        let stats = tracker.stats();
        assert_eq!(stats.selector_count, 1);
        assert_eq!(stats.v_bind_count, 1);
    }
}
