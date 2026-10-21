//! Canonical product schema for L2 static-subtree analysis.
//!
//! These facts describe canonical L2 owners and are independent of the
//! backend decision that consumes them. The transitional Vue lowering adapter
//! still produces the table while its binding-surface policy is separated from
//! the neutral tree walk; consumers can depend on this L2 schema directly.

use vize_l0::pass::{Fusability, PassDesc, PassKind, Preserved};

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "hoist-static";

/// Static-subtree analysis is optional, fusable and preserves the artifact.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);

/// The per-node constant level inherited from the legacy static-type lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticLevel {
    /// Dynamic surface or dynamic descendants; never hoistable whole.
    NotStatic,
    /// Static but for interpolation children; never whole-hoistable.
    HasDynamicText,
    /// The whole subtree is static where backend position policy permits it.
    FullyStatic,
}

/// Static-analysis facts for one canonical L2 element or component owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticFacts {
    /// The owner's static level.
    pub level: StaticLevel,
    /// Whether the non-empty props surface is fully hoistable.
    pub props_hoistable: bool,
    /// Whether the owner has non-empty, entirely static nested children.
    pub nested_static: bool,
    /// Whether every descendant is a native element or text-family node.
    pub native_descendants: bool,
    /// Whether the owner stands in a foreign markup namespace.
    pub foreign: bool,
}

// Facts cross compile boundaries with their artifact.
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<StaticFacts>();
};

/// Five bytes on every width: one tag byte plus four bools.
const _: () = assert!(core::mem::size_of::<StaticFacts>() == 5);
