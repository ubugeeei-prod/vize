/// The named phase an L3 artifact currently satisfies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// L2 has lowered into flat L3 ops; partition and schedule are not fixed.
    Built,
    /// Static/dynamic partition facts have been attached.
    Partitioned,
    /// State edges are in execution order.
    Scheduled,
}

impl Phase {
    /// Stable dump spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Built => "built",
            Self::Partitioned => "partitioned",
            Self::Scheduled => "scheduled",
        }
    }

    /// Parse the stable dump spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"built" => Some(Self::Built),
            b"partitioned" => Some(Self::Partitioned),
            b"scheduled" => Some(Self::Scheduled),
            _ => None,
        }
    }
}
