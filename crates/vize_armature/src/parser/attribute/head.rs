//! Completed authored attribute heads, including opaque v-pre spellings.

use super::super::Parser;
use vize_l1::markup::directive::frozen_attribute_name;
use vize_relief::{
    SourceLocation,
    errors::{CompilerError, ErrorCode},
};

impl<'a> Parser<'a> {
    pub(in crate::parser) fn report_head_custody_error(
        &mut self,
        loc: &SourceLocation,
        extra: bool,
    ) {
        let message = if extra {
            "Completed directive head custody has extra entries while freezing v-pre."
        } else {
            "Completed directive head custody is missing while freezing v-pre."
        };
        self.errors.push(CompilerError::with_message(
            ErrorCode::MissingDirectiveName,
            message,
            Some(loc.clone()),
        ));
    }

    /// Process the end of a full attribute or directive head.
    pub(in crate::parser) fn on_attrib_name_end_impl(&mut self, end: usize) {
        if self.in_v_pre
            && let Some(attr) = self.current_attr.as_ref()
        {
            let authored = self.get_source_retained(attr.name_start, end);
            let name = frozen_attribute_name(self.allocator, authored);
            if let Some(attr) = self.current_attr.as_mut() {
                attr.name = name;
            }
        }
        if let Some(ref mut attr) = self.current_attr {
            attr.name_end = end;
        }

        if let Some(ref mut dir) = self.current_dir {
            dir.name_end = end;
        }
    }
}
