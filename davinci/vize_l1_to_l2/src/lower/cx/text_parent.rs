//! Text rules belong to the direct owner; pre whitespace also spans descendants.
use vize_l1::{Element, SurfaceChild};

use super::Cx;

/// Classify the owner's rule once; text passes need no ancestor/RCDATA join.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextWhitespace {
    Normal,
    Pre,
    Rcdata,
}

impl<'a> Cx<'a> {
    pub(crate) fn normalize_pre_newlines(&self) -> bool {
        self.text_whitespace == TextWhitespace::Pre
    }

    pub(crate) fn enter_text_parent(
        &mut self,
        element: &Element<'a>,
        tag: &str,
        html: bool,
    ) -> (TextWhitespace, Option<u32>) {
        let previous = (self.text_whitespace, self.ignore_newline_at);
        let rcdata = html && tag == "textarea";
        self.text_whitespace = if rcdata {
            TextWhitespace::Rcdata
        } else if self.condense_depth > 0 {
            TextWhitespace::Pre
        } else {
            TextWhitespace::Normal
        };
        self.ignore_newline_at = if rcdata || (html && tag == "pre") {
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
