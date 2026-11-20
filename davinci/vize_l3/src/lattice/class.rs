#[cfg(creusot)]
use creusot_std::prelude::{Clone, DeepModel, Int, PartialEq, ensures, logic};

/// Value axis of the L3 reactivity lattice.
///
/// Under `cfg(creusot)` the derives that are not part of the lattice laws
/// (`Debug`, `Hash` and the declaration-order `PartialOrd`/`Ord`) are left
/// out of the proof crate; `tests/lattice.rs` checks the order against
/// `join` exhaustively. The product build keeps the original derive list, so
/// it expands to exactly the code it did before the proofs.
#[cfg_attr(
    not(creusot),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)
)]
#[cfg_attr(creusot, derive(Clone, Copy, PartialEq, Eq))]
#[repr(u8)]
pub enum ReactivityClass {
    /// The binding is independent of runtime inputs.
    Static = 0,
    /// The binding depends only on stable component props.
    PropsStable = 1,
    /// The binding reads local reactive state, capture, provide, or inject.
    Reactive = 2,
    /// The binding is effectful or escapes beyond the tracked component.
    Unstable = 3,
}

impl ReactivityClass {
    /// Stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::PropsStable => "props-stable",
            Self::Reactive => "reactive",
            Self::Unstable => "unstable",
        }
    }

    /// Parse the stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"static" => Some(Self::Static),
            b"props-stable" => Some(Self::PropsStable),
            b"reactive" => Some(Self::Reactive),
            b"unstable" => Some(Self::Unstable),
            _ => None,
        }
    }

    /// The position on the chain, the order every lattice law states.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn position(self) -> Int {
        match self {
            Self::Static => 0,
            Self::PropsStable => 1,
            Self::Reactive => 2,
            Self::Unstable => 3,
        }
    }

    /// The join as the laws state it: the higher position.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn join_logic(self, other: Self) -> Self {
        if self.position() >= other.position() {
            self
        } else {
            other
        }
    }

    /// Least upper bound in the stable-to-unstable order.
    #[cfg_attr(creusot, ensures(result == self.join_logic(other)))]
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }

    #[cfg(not(creusot))]
    const fn rank(self) -> u8 {
        self as u8
    }

    /// The proof crate's `rank`. Creusot 0.14 leaves an enum-to-discriminant
    /// cast unconstrained, so it reads the same numbers (the explicit
    /// `repr(u8)` discriminants above) through a match. `tests/lattice.rs`
    /// pins those discriminants to the chain positions.
    #[cfg(creusot)]
    #[ensures(result@ == self.position())]
    const fn rank(self) -> u8 {
        match self {
            Self::Static => 0,
            Self::PropsStable => 1,
            Self::Reactive => 2,
            Self::Unstable => 3,
        }
    }
}

/// Equality and order compare positions on the chain.
#[cfg(creusot)]
impl DeepModel for ReactivityClass {
    type DeepModelTy = Int;

    #[logic(open)]
    fn deep_model(self) -> Int {
        self.position()
    }
}
