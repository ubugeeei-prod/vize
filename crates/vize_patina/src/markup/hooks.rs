//! [`MarkupHooks`]: the hooks a [`super::MarkupRule`] subscribes to.
//!
//! The fused dispatcher ([`super::MarkupRuleSet`]) calls a rule only at the
//! hooks it subscribes to, so a lint pass pays one call per interested rule
//! per node rather than one per registered rule per hook — and skips a whole
//! binding / directive walk when no rule listens there.

use core::ops::BitOr;

/// A set of [`super::MarkupRule`] hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkupHooks(u16);

impl MarkupHooks {
    /// `enter_document`.
    pub const DOCUMENT: Self = Self(1);
    /// `enter_element`.
    pub const ELEMENT: Self = Self(1 << 1);
    /// `exit_element`.
    pub const EXIT_ELEMENT: Self = Self(1 << 2);
    /// `enter_binding`.
    pub const BINDING: Self = Self(1 << 3);
    /// `enter_directive`.
    pub const DIRECTIVE: Self = Self(1 << 4);
    /// `enter_conditional`.
    pub const CONDITIONAL: Self = Self(1 << 5);
    /// `enter_list`.
    pub const LIST: Self = Self(1 << 6);
    /// `enter_text`.
    pub const TEXT: Self = Self(1 << 7);
    /// `enter_interpolation`.
    pub const INTERPOLATION: Self = Self(1 << 8);
    /// Every hook: the default subscription, always correct.
    pub const ALL: Self = Self((1 << HOOK_COUNT) - 1);

    /// Whether every hook in `other` is in this set.
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// The single hooks, in dispatch-table order.
    pub(super) const EACH: [Self; HOOK_COUNT] = [
        Self::DOCUMENT,
        Self::ELEMENT,
        Self::EXIT_ELEMENT,
        Self::BINDING,
        Self::DIRECTIVE,
        Self::CONDITIONAL,
        Self::LIST,
        Self::TEXT,
        Self::INTERPOLATION,
    ];

    /// This single hook's dispatch-table index.
    #[inline]
    pub(super) const fn index(self) -> usize {
        self.0.trailing_zeros() as usize
    }
}

/// How many hooks a [`super::MarkupRule`] has.
pub(super) const HOOK_COUNT: usize = 9;

impl BitOr for MarkupHooks {
    type Output = Self;

    #[inline]
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
