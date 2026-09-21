use core::fmt;

/// Where one op's work runs once extraction has chosen a shape for it.
///
/// Every alternative is semantics-preserving where the verifier accepts it;
/// extraction only chooses among equivalent shapes by measured cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Placement {
    /// Canonical shape: the op runs where it stands, in its own effect scope
    /// when it is dynamic. Always available.
    Inline = 0,
    /// A structurally static subtree under a control region is materialized
    /// once and re-inserted, so its ops leave every update path.
    Hoist = 1,
    /// A scope-free event handler is created once and reused.
    Cache = 2,
    /// A leaf update joins the effect unit of its keyed predecessor when both
    /// read the identical direct reference.
    Group = 3,
}

impl Placement {
    /// Every placement in canonical print order.
    pub const ALL: [Self; 4] = [Self::Inline, Self::Hoist, Self::Cache, Self::Group];

    /// Stable Folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Hoist => "hoist",
            Self::Cache => "cache",
            Self::Group => "group",
        }
    }

    /// Parse the stable Folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"inline" => Some(Self::Inline),
            b"hoist" => Some(Self::Hoist),
            b"cache" => Some(Self::Cache),
            b"group" => Some(Self::Group),
            _ => None,
        }
    }

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A set of placement alternatives, printed in [`Placement::ALL`] order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PlacementSet(u8);

impl PlacementSet {
    /// No alternative at all. Never valid on a recorded op.
    pub const EMPTY: Self = Self(0);
    /// The canonical-only set.
    pub const INLINE: Self = Self(Placement::Inline.bit());

    const KNOWN: u8 = Placement::Inline.bit()
        | Placement::Hoist.bit()
        | Placement::Cache.bit()
        | Placement::Group.bit();

    /// This set plus `placement`.
    #[must_use]
    pub const fn with(self, placement: Placement) -> Self {
        Self(self.0 | placement.bit())
    }

    /// Whether `placement` is one of the alternatives.
    #[must_use]
    pub const fn contains(self, placement: Placement) -> bool {
        self.0 & placement.bit() != 0
    }

    /// Number of alternatives.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Whether the set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Raw bits in [`Placement`] discriminant order.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Rebuild a set from [`Self::bits`], rejecting unknown bits.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::KNOWN == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Alternatives in canonical order.
    pub fn iter(self) -> impl Iterator<Item = Placement> {
        Placement::ALL
            .into_iter()
            .filter(move |placement| self.contains(*placement))
    }
}

impl fmt::Display for PlacementSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("-");
        }
        for (index, placement) in self.iter().enumerate() {
            if index > 0 {
                f.write_str(",")?;
            }
            f.write_str(placement.as_str())?;
        }
        Ok(())
    }
}
