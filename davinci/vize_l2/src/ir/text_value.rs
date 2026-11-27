//! Normally owned original preparation, joined only to its real Text mint.

use super::{L2Ir, RejectedL2Ir};
use crate::{lang::js::NativeTemplateIssueKind, op::TextOp};
use core::ptr::NonNull;
use vize_l0::id::NodeId;
use vize_l1::markup::{TextValue, TextValueFailure};

/// Bounded source policy, independent of target or whitespace condensation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTextValuePolicyError {
    RootExtent,
    Leading,
    Empty,
    RawWhitespace,
    DecodedWhitespace,
    Control,
    ScriptOrStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeL2IrTextValueState {
    Pending,
    Attached(NodeId),
    Refused(NativeTemplateIssueKind),
}

/// Whole results remain normally owned on failure and caught unwind.
/// This row alone grants no File completion or native output profile.
/// ```compile_fail
/// use vize_l2::ir::NativeL2IrTextValue;
/// fn clone<T: Clone>() {}
/// clone::<NativeL2IrTextValue<'static>>();
/// ```
/// ```compile_fail
/// use vize_l2::ir::{NativeL2IrTextValue, NativeL2IrTextValueState};
/// let _ = NativeL2IrTextValue {
///     observation: panic!(), state: NativeL2IrTextValueState::Pending, text: None,
/// };
/// ```
pub struct NativeL2IrTextValue<'a> {
    pub(crate) observation: Result<TextValue<'a>, TextValueFailure<'a>>,
    pub(crate) state: NativeL2IrTextValueState,
    pub(crate) text: Option<NonNull<TextOp<'a>>>,
}
impl<'a> NativeL2IrTextValue<'a> {
    #[must_use]
    pub fn observation(&self) -> Option<&TextValue<'a>> {
        self.observation.as_ref().ok()
    }
    #[must_use]
    pub fn failure(&self) -> Option<&TextValueFailure<'a>> {
        self.observation.as_ref().err()
    }
    #[must_use]
    pub const fn state(&self) -> NativeL2IrTextValueState {
        self.state
    }
}

/// Same actual File, NodeId and arena Text allocation, with whole preparation.
/// Equal bytes, a numeric node or a copied Text cannot construct this view.
/// ```compile_fail
/// use vize_l2::{ir::{L2Ir, L2IrTextValue}, op::TextOp};
/// use vize_l0::id::NodeId;
/// fn forge<'f, 'a>(l2ir: &'f L2Ir<'a>, text: &'f TextOp<'a>) {
///     let _ = L2IrTextValue { l2ir, text, node: NodeId::FIRST };
/// }
/// ```
#[derive(Clone, Copy)]
pub struct L2IrTextValue<'f, 'a> {
    l2ir: &'f L2Ir<'a>,
    text: &'f TextOp<'a>,
    node: NodeId,
}
impl<'a> L2Ir<'a> {
    #[must_use]
    pub fn native_text_values(&self) -> &[NativeL2IrTextValue<'a>] {
        &self.facts.native_text_values
    }
    /// Attached diagnostic prefixes remain readable even in incomplete Files.
    /// Product completion must independently require the actual whole owner.
    #[must_use]
    pub fn native_text_value_for<'f>(
        &'f self,
        node: NodeId,
        text: &'f TextOp<'a>,
    ) -> Option<L2IrTextValue<'f, 'a>> {
        let [record] = self.facts.native_text_values.as_slice() else {
            return None;
        };
        let observation = record.observation()?;
        let source = observation.source();
        if record.state != NativeL2IrTextValueState::Attached(node)
            || record.text != Some(NonNull::from(text))
            || !self.artifact().contains_node(node)
            || !core::ptr::eq(source.authored_root(), self.artifact().source())
            || source.span() != text.span
            || !core::ptr::eq(source.text(), text.content)
        {
            return None;
        }
        Some(L2IrTextValue {
            l2ir: self,
            text,
            node,
        })
    }
}
impl<'a> RejectedL2Ir<'a> {
    #[must_use]
    pub fn native_text_values(&self) -> &[NativeL2IrTextValue<'a>] {
        &self.facts.native_text_values
    }
}
impl<'f, 'a> L2IrTextValue<'f, 'a> {
    #[must_use]
    pub const fn l2ir(self) -> &'f L2Ir<'a> {
        self.l2ir
    }
    #[must_use]
    pub const fn text(self) -> &'f TextOp<'a> {
        self.text
    }
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }
    #[must_use]
    pub fn observation(self) -> Option<&'f TextValue<'a>> {
        self.l2ir.native_text_values().first()?.observation()
    }
}
