//! Text rules belong to the direct owner; pre whitespace also spans descendants.
use vize_l1::{Element, SurfaceChild};

use super::Cx;

impl<'a> Cx<'a> {
    pub(crate) fn normalize_pre_newlines(&self) -> bool {
        self.condense_depth > 0 && !self.rcdata_text
    }

    pub(crate) fn enter_text_parent(
        &mut self,
        element: &Element<'a>,
        tag: &str,
        html: bool,
    ) -> (bool, Option<u32>) {
        let previous = (self.rcdata_text, self.ignore_newline_at);
        self.rcdata_text = html && tag == "textarea";
        self.ignore_newline_at = if html && matches!(tag, "pre" | "textarea") {
            element
                .children
                .iter()
                .find(|child| match child {
                    SurfaceChild::Comment(token) => {
                        self.preserve_comments()
                            || super::super::leaf::keeps_directive_comment(token.text)
                    }
                    _ => true,
                })
                .and_then(|child| match child {
                    SurfaceChild::Text(token) => Some(self.offset(token.text)),
                    _ => None,
                })
        } else {
            None
        };
        previous
    }
}
