//! Whole original handler facts joined at the actual canonical On event.

use vize_l2::{file::FileHandler, op::OnOp, resolution::HandlerResolution};

/// A private same-File, same-allocation join, never a numeric lookup receipt.
/// This retains complete handler facts; it does not classify Vue handler
/// reference versus inline syntax or authorize runtime spelling.
///
/// ```compile_fail
/// use vize_l2::{file::FileHandler, op::OnOp};
/// use vize_l3::decision::dom::DomFileHandler;
/// fn forge<'f,'a>(handler: FileHandler<'f,'a>, on: &'f OnOp<'a>) {
///     let _ = DomFileHandler { handler, on };
/// }
/// ```
pub struct DomFileHandler<'owner, 'arena> {
    pub(in crate::decision::dom) handler: FileHandler<'owner, 'arena>,
    pub(in crate::decision::dom) on: &'owner OnOp<'arena>,
    pub(in crate::decision::dom) resolution: &'owner HandlerResolution<'arena>,
}

impl core::fmt::Debug for DomFileHandler<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DomFileHandler")
            .field("node", &self.handler.id().node())
            .field("scope", &self.handler.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> DomFileHandler<'owner, 'arena> {
    #[must_use]
    pub const fn handler(&self) -> FileHandler<'owner, 'arena> {
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
