//! Whole original For custody joined during the existing canonical Enter event.

use super::vue::VueRenderRead;
use vize_l2::{
    file::{BindingRef, FileForHead},
    op::OriginalForOp,
    resolution::ForResolution,
};

/// An authentic same-File/allocation receipt with bounded original setup reads,
/// without loop runtime eligibility. All original observations remain normally owned
/// by the borrowed File; no AST, parameter or source copy is stored here.
///
/// Caller-written combinations cannot mint this receipt:
/// ```compile_fail
/// use vize_l2::{file::{BindingRef, FileForHead}, op::OriginalForOp, resolution::ForResolution};
/// use vize_l3::decision::dom::DomFileForHead;
/// fn forge<'f, 'a>(head: FileForHead<'f, 'a>, original: &'f OriginalForOp<'a>,
///     resolution: &'f ForResolution<'a>, collection: BindingRef<'f, 'a>) {
///     let _ = DomFileForHead { head, original, resolution, collection, read: None };
/// }
/// ```
/// ```compile_fail
/// use vize_l3::decision::dom::DomFileForHead;
/// fn duplicate(row: DomFileForHead<'_, '_>) { let _ = row.clone(); }
/// ```
/// The actual owner must remain live:
/// ```compile_fail
/// use vize_l2::{file::FileArtifact, op::Op};
/// use vize_l3::decision::build_dom_file_decisions;
/// fn discard(file: FileArtifact<'_>) {
///     let analysis = build_dom_file_decisions(&file).unwrap();
///     let Op::OriginalFor(original) = &file.artifact().root().ops[0] else { return };
///     let row = analysis.dom().unwrap().file_for_head(original.id().node()).unwrap();
///     drop(file);
///     let _ = row.resolution();
/// }
/// ```
pub struct DomFileForHead<'owner, 'arena> {
    pub(in crate::decision::dom) head: FileForHead<'owner, 'arena>,
    pub(in crate::decision::dom) original: &'owner OriginalForOp<'arena>,
    pub(in crate::decision::dom) resolution: &'owner ForResolution<'arena>,
    pub(in crate::decision::dom) collection: BindingRef<'owner, 'arena>,
    pub(in crate::decision::dom) read: Option<VueRenderRead<'owner, 'arena>>,
}

impl core::fmt::Debug for DomFileForHead<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DomFileForHead")
            .field("node", &self.head.id().node())
            .field("scope", &self.head.scope())
            .finish_non_exhaustive()
    }
}

impl<'owner, 'arena> DomFileForHead<'owner, 'arena> {
    #[must_use]
    pub const fn head(&self) -> FileForHead<'owner, 'arena> {
        self.head
    }
    #[must_use]
    pub const fn original(&self) -> &'owner OriginalForOp<'arena> {
        self.original
    }
    #[must_use]
    pub const fn resolution(&self) -> &'owner ForResolution<'arena> {
        self.resolution
    }
    /// The enclosing collection binding may be a genuine parent template alias.
    /// Its identity supplies no Vue runtime spelling or loop target policy.
    #[must_use]
    pub const fn collection(&self) -> BindingRef<'owner, 'arena> {
        self.collection
    }
    /// The actual setup collection occurrence and its sealed native access kind.
    /// Parent-template aliases retain custody but have no setup read policy.
    /// A positive read alone grants no loop target or executable output.
    #[must_use]
    pub fn collection_read(&self) -> Option<&VueRenderRead<'owner, 'arena>> {
        self.read.as_ref()
    }
    #[must_use]
    pub fn accepts_original(&self, original: &OriginalForOp<'arena>) -> bool {
        core::ptr::eq(self.original, original) && self.head.accepts(original)
    }
}
