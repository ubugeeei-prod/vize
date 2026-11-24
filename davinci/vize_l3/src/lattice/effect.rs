#[cfg(not(creusot))]
use core::fmt;

#[cfg(creusot)]
use creusot_std::prelude::{Clone, Invariant, bitwise_proof, ensures, logic};

use super::ReactivityClass;
#[cfg(creusot)]
use super::ReactivityClass::{PropsStable, Reactive, Static, Unstable};

#[cfg(not(creusot))]
const EFFECTS: [EffectKind; 8] = [
    EffectKind::Freeze,
    EffectKind::Capture,
    EffectKind::ReadProp,
    EffectKind::ReadReactive,
    EffectKind::MutateLocal,
    EffectKind::MutateGlobal,
    EffectKind::CallUnknown,
    EffectKind::Allocate,
];

/// React-Compiler-style effect vocabulary used by the naive P3-2 classifier.
///
/// Under `cfg(creusot)` the derives that read the discriminant through
/// intrinsics (`Debug`, `PartialEq`, `Eq`, `Hash`) are left out of the proof
/// crate, which compares kinds in logic only. The product build keeps the
/// original derive list.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, PartialEq, Eq, Hash))]
#[cfg_attr(creusot, derive(Clone, Copy))]
#[repr(u8)]
pub enum EffectKind {
    Freeze = 0,
    Capture = 1,
    ReadProp = 2,
    ReadReactive = 3,
    MutateLocal = 4,
    MutateGlobal = 5,
    CallUnknown = 6,
    Allocate = 7,
}

impl EffectKind {
    /// Stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Freeze => "freeze",
            Self::Capture => "capture",
            Self::ReadProp => "read-prop",
            Self::ReadReactive => "read-reactive",
            Self::MutateLocal => "mutate-local",
            Self::MutateGlobal => "mutate-global",
            Self::CallUnknown => "call-unknown",
            Self::Allocate => "allocate",
        }
    }

    /// Parse the stable folio spelling.
    #[cfg(not(creusot))]
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"freeze" => Some(Self::Freeze),
            b"capture" => Some(Self::Capture),
            b"read-prop" => Some(Self::ReadProp),
            b"read-reactive" => Some(Self::ReadReactive),
            b"mutate-local" => Some(Self::MutateLocal),
            b"mutate-global" => Some(Self::MutateGlobal),
            b"call-unknown" => Some(Self::CallUnknown),
            b"allocate" => Some(Self::Allocate),
            _ => None,
        }
    }

    /// The kind's bit in an [`EffectSet`]: its `repr(u8)` discriminant.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn offset(self) -> u16 {
        match self {
            Self::Freeze => 0u16,
            Self::Capture => 1u16,
            Self::ReadProp => 2u16,
            Self::ReadReactive => 3u16,
            Self::MutateLocal => 4u16,
            Self::MutateGlobal => 5u16,
            Self::CallUnknown => 6u16,
            Self::Allocate => 7u16,
        }
    }

    /// The P3-2 rule table (`docs/davinci/plan/phase-3.md`): a binding that
    /// carries this effect is at least this unstable.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn demand(self) -> ReactivityClass {
        match self {
            Self::Freeze | Self::ReadProp | Self::Allocate => PropsStable,
            Self::Capture | Self::ReadReactive | Self::MutateLocal => Reactive,
            Self::MutateGlobal | Self::CallUnknown => Unstable,
        }
    }

    #[cfg(not(creusot))]
    const fn bit(self) -> u16 {
        1 << (self as u8)
    }

    /// The proof crate's `bit`. Creusot 0.14 leaves an enum-to-discriminant
    /// cast unconstrained, so it spells the same bits (one per `repr(u8)`
    /// discriminant above) through a match. `tests/lattice.rs` pins those
    /// discriminants to the offsets.
    #[cfg(creusot)]
    #[ensures(result == 1u16 << self.offset())]
    #[bitwise_proof]
    const fn bit(self) -> u16 {
        match self {
            Self::Freeze => 1,
            Self::Capture => 2,
            Self::ReadProp => 4,
            Self::ReadReactive => 8,
            Self::MutateLocal => 16,
            Self::MutateGlobal => 32,
            Self::CallUnknown => 64,
            Self::Allocate => 128,
        }
    }
}

/// Compact effect set for one binding or expression summary.
///
/// Under `cfg(creusot)` the derives outside the lattice laws (`Debug`,
/// `Default`, `PartialEq`, `Eq`, `Hash`) are left out of the proof crate;
/// `tests/lattice.rs` checks them against membership. The product build keeps
/// the original derive list.
#[cfg_attr(not(creusot), derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash))]
#[cfg_attr(creusot, derive(Copy))]
pub struct EffectSet(u16);

/// The proof crate's `Clone`: a copy. Creusot's `Clone` derive does not
/// resolve on a tuple struct of a machine integer.
#[cfg(creusot)]
impl Clone for EffectSet {
    #[ensures(result == *self)]
    fn clone(&self) -> Self {
        *self
    }
}

/// A set holds kinds of the vocabulary only: no bit above the eight offsets.
#[cfg(creusot)]
impl Invariant for EffectSet {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        self.0 < 256u16
    }
}

impl EffectSet {
    /// Membership, the only thing the lattice laws observe of a set: the
    /// bit at the kind's offset is set. The body is visible in this module
    /// only, where the set operations are proved in Creusot's bitwise mode
    /// (`bitwise_proof`: `u16` operations are Why3 bitvectors); the laws read
    /// none of it.
    #[cfg(creusot)]
    #[logic(open(self))]
    pub fn has(self, kind: EffectKind) -> bool {
        self.0 & (1u16 << kind.offset()) != 0u16
    }

    /// `kind`'s demand when the set has it, the unit `static` otherwise.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn demand_of(self, kind: EffectKind) -> ReactivityClass {
        if self.has(kind) {
            kind.demand()
        } else {
            Static
        }
    }

    /// The effect floor as the rule table states it: the join of the demand
    /// of every member, over the whole vocabulary.
    #[cfg(creusot)]
    #[logic(open)]
    pub fn floor(self) -> ReactivityClass {
        self.demand_of(EffectKind::Freeze)
            .join_logic(self.demand_of(EffectKind::Capture))
            .join_logic(self.demand_of(EffectKind::ReadProp))
            .join_logic(self.demand_of(EffectKind::ReadReactive))
            .join_logic(self.demand_of(EffectKind::MutateLocal))
            .join_logic(self.demand_of(EffectKind::MutateGlobal))
            .join_logic(self.demand_of(EffectKind::CallUnknown))
            .join_logic(self.demand_of(EffectKind::Allocate))
    }

    /// Empty summary.
    #[cfg_attr(creusot, ensures(forall<k: EffectKind> !result.has(k)), bitwise_proof)]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Summary containing one effect.
    #[cfg_attr(creusot, ensures(forall<k: EffectKind> result.has(k) == (k == kind)), bitwise_proof)]
    #[must_use]
    pub const fn one(kind: EffectKind) -> Self {
        Self(kind.bit())
    }

    /// Add one effect.
    #[cfg_attr(
        creusot,
        ensures(forall<k: EffectKind> result.has(k) == (self.has(k) || k == kind)),
        bitwise_proof
    )]
    #[must_use]
    pub const fn with(self, kind: EffectKind) -> Self {
        Self(self.0 | kind.bit())
    }

    /// Merge two summaries.
    #[cfg_attr(
        creusot,
        ensures(forall<k: EffectKind> result.has(k) == (self.has(k) || other.has(k))),
        bitwise_proof
    )]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether the effect is present.
    #[cfg_attr(creusot, ensures(result == self.has(kind)))]
    #[must_use]
    pub const fn contains(self, kind: EffectKind) -> bool {
        self.0 & kind.bit() != 0
    }

    /// Whether no effect is present.
    #[cfg_attr(creusot, ensures(result == forall<k: EffectKind> !self.has(k)), bitwise_proof)]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The lattice floor implied by effects alone.
    #[cfg_attr(creusot, ensures(result == self.floor()))]
    #[must_use]
    pub const fn class_floor(self) -> ReactivityClass {
        if self.contains(EffectKind::MutateGlobal) || self.contains(EffectKind::CallUnknown) {
            ReactivityClass::Unstable
        } else if self.contains(EffectKind::ReadReactive)
            || self.contains(EffectKind::MutateLocal)
            || self.contains(EffectKind::Capture)
        {
            ReactivityClass::Reactive
        } else if self.contains(EffectKind::ReadProp)
            || self.contains(EffectKind::Freeze)
            || self.contains(EffectKind::Allocate)
        {
            ReactivityClass::PropsStable
        } else {
            ReactivityClass::Static
        }
    }

    #[cfg(not(creusot))]
    pub(super) fn print<W: fmt::Write>(self, w: &mut W) -> fmt::Result {
        if self.is_empty() {
            return w.write_str("-");
        }
        let mut first = true;
        for effect in EFFECTS {
            if !self.contains(effect) {
                continue;
            }
            if !first {
                w.write_str(",")?;
            }
            w.write_str(effect.as_str())?;
            first = false;
        }
        Ok(())
    }

    #[cfg(not(creusot))]
    pub(super) fn parse(text: &str) -> Option<Self> {
        if text == "-" {
            return Some(Self::empty());
        }
        let mut set = Self::empty();
        for raw in text.split(',') {
            set = set.with(EffectKind::from_str(raw)?);
        }
        Some(set)
    }
}
