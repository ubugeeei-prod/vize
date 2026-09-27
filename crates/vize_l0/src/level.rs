//! The level registry: L0–L4 and the conversion crates between them.
//!
//! Levels are type boundaries, not runtime boundaries. The registry is const
//! data for tooling (`vize dump --level`, dependency gates); nothing converts
//! through it at run time.

/// One Davinci level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// L0: source, arena, spans, ids and the shared managers.
    L0,
    /// L1: what the text is — lossless surface trees.
    L1,
    /// L2: what the text means — the semantic IR.
    L2,
    /// L3: what to do — reactivity and backend decisions.
    L3,
    /// L4: emission.
    L4,
}

impl Level {
    /// Every level, in order.
    pub const ALL: [Level; 5] = [Level::L0, Level::L1, Level::L2, Level::L3, Level::L4];

    /// The short id: `l0` … `l4`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Level::L0 => "l0",
            Level::L1 => "l1",
            Level::L2 => "l2",
            Level::L3 => "l3",
            Level::L4 => "l4",
        }
    }

    /// The level crate name, e.g. `vize_l2`.
    #[must_use]
    pub const fn crate_name(self) -> &'static str {
        match self {
            Level::L0 => "vize_l0",
            Level::L1 => "vize_l1",
            Level::L2 => "vize_l2",
            Level::L3 => "vize_l3",
            Level::L4 => "vize_l4",
        }
    }

    /// Parse a short id (`l0` … `l4`).
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.id() == id)
    }
}

/// A conversion crate between two adjacent levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conversion {
    /// Input level.
    pub from: Level,
    /// Output level.
    pub to: Level,
    /// The crate name, e.g. `vize_l1_to_l2`.
    pub crate_name: &'static str,
}

/// Every conversion crate. L0→L1 and L3→L4 live inside their target levels.
pub const CONVERSIONS: &[Conversion] = &[
    Conversion {
        from: Level::L1,
        to: Level::L2,
        crate_name: "vize_l1_to_l2",
    },
    Conversion {
        from: Level::L2,
        to: Level::L3,
        crate_name: "vize_l2_to_l3",
    },
];

#[cfg(test)]
mod tests {
    use super::{CONVERSIONS, Level};

    #[test]
    fn ids_round_trip() {
        let expected = [
            (Level::L0, "l0", "vize_l0"),
            (Level::L1, "l1", "vize_l1"),
            (Level::L2, "l2", "vize_l2"),
            (Level::L3, "l3", "vize_l3"),
            (Level::L4, "l4", "vize_l4"),
        ];
        for (level, id, crate_name) in expected {
            assert_eq!(level.id(), id);
            assert_eq!(level.crate_name(), crate_name);
            assert_eq!(Level::from_id(id), Some(level));
        }
        assert_eq!(Level::from_id("s2"), None);
    }

    #[test]
    fn conversions_join_adjacent_levels() {
        for conversion in CONVERSIONS {
            let to = conversion
                .crate_name
                .strip_prefix("vize_")
                .and_then(|rest| rest.strip_prefix(conversion.from.id()))
                .and_then(|rest| rest.strip_prefix("_to_"));
            assert_eq!(to, Some(conversion.to.id()));
        }
    }
}
