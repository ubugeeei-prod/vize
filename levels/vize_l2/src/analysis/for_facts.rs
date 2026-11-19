//! Canonical product schema for L2 `ui.for` binding positions.
//!
//! Lowerings retain their own grammar-specific name discovery. Consumers share
//! these owned facts without depending on a particular input's lowering crate.

use vize_l0::String;

use crate::scope::ScopeTag;

/// One `ui.for`'s consumed scope view, positions in grammar order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForParts {
    /// The introduction-site tag the lowering minted.
    pub tag: ScopeTag,
    /// The value position.
    pub value: ForName,
    /// The second position (object key).
    pub key: ForName,
    /// The third position (index).
    pub index: ForName,
}

/// One binding position's consumed name status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForName {
    /// The position's one enumerated simple-identifier name.
    Named(String),
    /// The position is authored but is not a simple identifier: a
    /// destructuring pattern, a foreign dialect, or the classified escape.
    Pending,
    /// The position was not authored.
    Absent,
}

impl ForName {
    /// The provenance spelling: the name, `?` for pending, `-` for absent.
    #[inline]
    pub fn spell(&self) -> &str {
        match self {
            ForName::Named(name) => name.as_str(),
            ForName::Pending => "?",
            ForName::Absent => "-",
        }
    }
}

// Facts cross compile boundaries with their artifact.
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ForName>();
    assert_owned::<ForParts>();
};

// Retain the original 64-bit footprints; wasm32 has its own pointer width.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ForName>() == 24);
    assert!(core::mem::size_of::<ForParts>() == 80);
};
