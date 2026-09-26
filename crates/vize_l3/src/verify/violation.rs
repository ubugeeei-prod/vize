//! Violation codes and records shared by every verifier check.

use core::fmt;

use vize_l0::{Span, String};

/// Which invariant a [`Violation`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationCode {
    DuplicateId,
    RootRegion,
    OpRegion,
    EdgeEndpoint,
    RegionResolution,
    RegionNesting,
    EffectScope,
    ScheduledOrder,
    Operand,
    Placement,
}

impl ViolationCode {
    /// Every invariant code, in declaration order; catalog coverage checks this list.
    pub const ALL: [Self; 10] = [
        Self::DuplicateId,
        Self::RootRegion,
        Self::OpRegion,
        Self::EdgeEndpoint,
        Self::RegionResolution,
        Self::RegionNesting,
        Self::EffectScope,
        Self::ScheduledOrder,
        Self::Operand,
        Self::Placement,
    ];

    /// Stable rendering code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateId => "L3V001",
            Self::RootRegion => "L3V002",
            Self::OpRegion => "L3V003",
            Self::EdgeEndpoint => "L3V004",
            Self::RegionResolution => "L3V005",
            Self::RegionNesting => "L3V006",
            Self::EffectScope => "L3V007",
            Self::ScheduledOrder => "L3V008",
            Self::Operand => "L3V009",
            Self::Placement => "L3V010",
        }
    }
}

/// One rejected invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub code: ViolationCode,
    pub span: Span,
    pub message: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} @{}:{} {}",
            self.code.as_str(),
            self.span.start,
            self.span.end,
            self.message
        )
    }
}

const _: () = assert!(size_of::<ViolationCode>() == 1);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Violation>() == 40);
