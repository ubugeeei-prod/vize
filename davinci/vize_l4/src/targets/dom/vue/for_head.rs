//! No ExprRef is fabricated for the owning original collection observation.

use super::VueRows;
use crate::{
    targets::dom::{DomError, DomErrorKind},
    write::{LinkSink, Writer},
};
use vize_l0::id::NodeId;
use vize_l2::op::OriginalForOp;
use vize_l3::decision::dom::vue::VueReadKind;

pub(super) fn write<L: LinkSink>(
    rows: &VueRows<'_, '_, '_, '_, '_, '_>,
    writer: &mut Writer<L>,
    node: NodeId,
    original: &OriginalForOp<'_>,
) -> Result<(), DomError> {
    let reject = |kind| DomError {
        node: Some(node),
        span: original.span,
        kind,
    };
    let VueRows::Selected(analysis) = rows else {
        return Err(reject(DomErrorKind::RuntimeAccessUnavailable));
    };
    let row = analysis
        .dom()
        .and_then(|dom| dom.file_for_head(node))
        .filter(|row| row.accepts_original(original))
        .ok_or_else(|| reject(DomErrorKind::FileOwnerMismatch))?;
    let read = row
        .collection_read()
        .ok_or_else(|| reject(DomErrorKind::RuntimeAccessUnavailable))?;
    if read.kind != VueReadKind::SetupLet
        || !core::ptr::eq(read.occurrence, row.resolution().collection_occurrence())
        || !core::ptr::eq(read.binding.file(), analysis.file())
        || analysis.setup().binding(read.binding).is_err()
    {
        return Err(reject(DomErrorKind::RuntimeAccessUnavailable));
    }
    writer.push("$setup.");
    writer.push_named(
        read.occurrence.name,
        row.resolution().collection_authored_span(),
        read.occurrence.name,
    );
    Ok(())
}
