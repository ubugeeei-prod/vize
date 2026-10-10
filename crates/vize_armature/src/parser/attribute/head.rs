//! Full attribute-head completion.

use super::Parser;

impl<'a> Parser<'a> {
    /// Process the end of a full attribute or directive head.
    pub(in crate::parser) fn on_attrib_name_end_impl(&mut self, end: usize) {
        if let Some(ref mut attr) = self.current_attr {
            attr.name_end = end;
        }

        if let Some(ref mut dir) = self.current_dir {
            dir.name_end = end;
        }
    }
}
