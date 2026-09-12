use vize_s0::{Allocator, Span, Vec};
use vize_s3::op::OpId;

/// Static or dynamic partition assigned to one canonical S3 op.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PartitionKind {
    /// The op can live in the static partition unless a containing control
    /// context makes it dynamic.
    Static = 0,
    /// The op participates in runtime updates or is controlled by one.
    Dynamic = 1,
}

impl PartitionKind {
    /// Stable spelling used by records and future folio output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dynamic => "dynamic",
        }
    }

    /// Combine two partition requirements.
    #[must_use]
    pub const fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Dynamic, _) | (_, Self::Dynamic) => Self::Dynamic,
            (Self::Static, Self::Static) => Self::Static,
        }
    }

    #[must_use]
    pub const fn is_dynamic(self) -> bool {
        matches!(self, Self::Dynamic)
    }
}

/// One exported partition fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionFact {
    pub op: OpId,
    pub kind: PartitionKind,
    pub span: Span,
}

/// Partition facts computed during S2→S3 lowering.
#[derive(Debug)]
pub struct PartitionFacts<'a> {
    pub ops: Vec<'a, PartitionFact>,
}

impl<'a> PartitionFacts<'a> {
    /// Empty fact group.
    #[must_use]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            ops: Vec::new_in(&allocator),
        }
    }

    /// Add one op fact.
    pub fn push(&mut self, fact: PartitionFact) {
        self.ops.push(fact);
    }

    /// Find the partition fact for `op`.
    #[must_use]
    pub fn get(&self, op: OpId) -> Option<&PartitionFact> {
        self.ops.iter().find(|fact| fact.op == op)
    }
}

const _: () = assert!(!core::mem::needs_drop::<PartitionFact>());
const _: () = assert!(core::mem::size_of::<PartitionKind>() == 1);

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<PartitionFact>() == 16);
