/// The named phase an Impeto artifact currently satisfies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// S2 has lowered into flat S3 ops; partition and schedule are not fixed.
    Built,
    /// Static/dynamic partition facts have been attached.
    Partitioned,
    /// State edges are in execution order.
    Scheduled,
}

impl Phase {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Built => "built",
            Self::Partitioned => "partitioned",
            Self::Scheduled => "scheduled",
        }
    }

    /// Parse the stable folio spelling.
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
