//! Violation codes and records shared by every verifier check.

use core::fmt;

use vize_s0::{Span, String};

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
    /// Stable rendering code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateId => "S3V001",
            Self::RootRegion => "S3V002",
            Self::OpRegion => "S3V003",
            Self::EdgeEndpoint => "S3V004",
            Self::RegionResolution => "S3V005",
            Self::RegionNesting => "S3V006",
            Self::EffectScope => "S3V007",
            Self::ScheduledOrder => "S3V008",
            Self::Operand => "S3V009",
            Self::Placement => "S3V010",
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
