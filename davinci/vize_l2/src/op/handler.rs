//! Event positions distinguish expressions from original File-owned bodies.

use crate::expr::{ExprRef, ForeignExpr, JsExpr, OpaqueExpr, VueFilterExpr};
use vize_l0::id::NodeId;

/// An actual attached event node, meaningful only beside its owning File.
/// Numeric IDs cannot establish body ownership or native admission.
/// ```compile_fail
/// use vize_l2::op::HandlerId;
/// use vize_l0::id::NodeId;
/// let _ = HandlerId(NodeId::FIRST);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandlerId(pub(crate) NodeId);

impl HandlerId {
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.0
    }
}

/// The original expression family, or a genuine whole-handler File entry.
/// Keeping the expression variants direct preserves the two-word footprint.
#[derive(Debug, Clone, Copy)]
pub enum OnHandlerRef<'a> {
    Js(&'a JsExpr<'a>),
    Foreign(&'a ForeignExpr<'a>),
    Filter(&'a VueFilterExpr<'a>),
    Opaque(&'a OpaqueExpr<'a>),
    Body(HandlerId),
}

impl<'a> OnHandlerRef<'a> {
    /// The existing diagnostic expression family. A body requires its File.
    #[must_use]
    pub const fn expression(self) -> Option<ExprRef<'a>> {
        match self {
            Self::Js(value) => Some(ExprRef::Js(value)),
            Self::Foreign(value) => Some(ExprRef::Foreign(value)),
            Self::Filter(value) => Some(ExprRef::Filter(value)),
            Self::Opaque(value) => Some(ExprRef::Opaque(value)),
            Self::Body(_) => None,
        }
    }
    #[must_use]
    pub const fn body(self) -> Option<HandlerId> {
        match self {
            Self::Body(id) => Some(id),
            _ => None,
        }
    }
}

impl<'a> From<ExprRef<'a>> for OnHandlerRef<'a> {
    fn from(value: ExprRef<'a>) -> Self {
        match value {
            ExprRef::Js(value) => Self::Js(value),
            ExprRef::Foreign(value) => Self::Foreign(value),
            ExprRef::Filter(value) => Self::Filter(value),
            ExprRef::Opaque(value) => Self::Opaque(value),
        }
    }
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<OnHandlerRef<'_>>() == 16);
    assert!(!core::mem::needs_drop::<OnHandlerRef<'_>>());
};
