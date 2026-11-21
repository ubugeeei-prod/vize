//! Reserve an enclosing link in preorder, then close it after writing its children.
//! Offsets are already in the final document. Growing the row buffer preserves
//! the index, including when another enclosing link opens in the meantime.
use super::{LinkSink, Recorded, Writer};
use vize_l0::Span;

pub(crate) struct OpenLink(usize);

impl Writer<Recorded> {
    pub(crate) fn open_link(&mut self) -> OpenLink {
        let index = self.links.links.len();
        let at = self.cursor();
        self.links.record(Span::new(at, at), Span::new(0, 0), None);
        OpenLink(index)
    }

    /// An unfinished document never leaves its projection request.
    pub(crate) fn close_link(&mut self, open: OpenLink, authored: Span) -> Option<()> {
        let end = u32::try_from(self.len()).ok()?;
        let row = self.links.links.get_mut(open.0)?;
        row.generated.end = end;
        row.authored = authored;
        Some(())
    }
}
