//! Original iteration positions refer to their actual File-owned head.

use super::Region;
use vize_l0::{Span, id::NodeId};

/// A checked introducing iteration node beside its actual File owner.
/// Numeric IDs cannot establish original head or declaration custody.
/// ```compile_fail
/// use vize_l2::op::OriginalForId;
/// use vize_l0::id::NodeId;
/// let _ = OriginalForId(NodeId::FIRST);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalForId(pub(crate) NodeId);

impl OriginalForId {
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.0
    }
}

/// One original iteration owns its repeated region. Its actual collection and
/// parameter roots stay normally owned by the associated File record.
/// There is no synthetic expression in an original declaration position.
#[derive(Debug)]
pub struct OriginalForOp<'a> {
    pub(crate) id: OriginalForId,
    pub region: Region<'a>,
    pub span: Span,
}

impl OriginalForOp<'_> {
    #[must_use]
    pub const fn id(&self) -> OriginalForId {
        self.id
    }
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<OriginalForOp<'_>>() <= 96);
    assert!(!core::mem::needs_drop::<OriginalForOp<'_>>());
};
