//! Whole original handler facts joined at the actual canonical On event.

use vize_l2::{file::L2IrHandler, op::OnOp, resolution::HandlerResolution};

/// A private same-File, same-allocation join, never a numeric lookup receipt.
/// This retains complete handler facts; it does not classify Vue handler
/// reference versus inline syntax or authorize runtime spelling.
///
/// ```compile_fail
/// use vize_l2::{file::L2IrHandler, op::OnOp};
/// use vize_l3::decision::dom::DomL2IrHandler;
/// fn forge<'f,'a>(handler: L2IrHandler<'f,'a>, on: &'f OnOp<'a>) {
///     let _ = DomL2IrHandler { handler, on };
/// }
/// ```
pub struct DomL2IrHandler<'owner, 'arena> {
    pub(in crate::decision::dom) handler: L2IrHandler<'owner, 'arena>,
    pub(in crate::decision::dom) on: &'owner OnOp<'arena>,
    pub(in crate::decision::dom) resolution: &'owner HandlerResolution<'arena>,
}

impl core::fmt::Debug for DomL2IrHandler<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DomL2IrHandler")
            .field("node", &self.handler.id().node())
            .field("scope", &self.handler.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> DomL2IrHandler<'owner, 'arena> {
    #[must_use]
    pub const fn handler(&self) -> L2IrHandler<'owner, 'arena> {
        self.handler
    }
    #[must_use]
    pub const fn on(&self) -> &'owner OnOp<'arena> {
        self.on
    }
    #[must_use]
    pub const fn resolution(&self) -> &'owner HandlerResolution<'arena> {
        self.resolution
    }
    /// Copied and equal-ID foreign On values cannot replace this actual event.
    #[must_use]
    pub fn accepts_on(&self, on: &OnOp<'arena>) -> bool {
        core::ptr::eq(self.on, on) && self.handler.accepts_on(on)
    }
}
