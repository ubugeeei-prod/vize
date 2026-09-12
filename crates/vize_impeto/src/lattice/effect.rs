use core::fmt;

use super::ReactivityClass;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    const fn bit(self) -> u16 {
        1 << (self as u8)
    }
}

/// Compact effect set for one binding or expression summary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct EffectSet(u16);

impl EffectSet {
    /// Empty summary.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Summary containing one effect.
    #[must_use]
    pub const fn one(kind: EffectKind) -> Self {
        Self(kind.bit())
    }

    /// Add one effect.
    #[must_use]
    pub const fn with(self, kind: EffectKind) -> Self {
        Self(self.0 | kind.bit())
    }

    /// Merge two summaries.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether the effect is present.
    #[must_use]
    pub const fn contains(self, kind: EffectKind) -> bool {
        self.0 & kind.bit() != 0
    }

    /// Whether no effect is present.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The lattice floor implied by effects alone.
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
