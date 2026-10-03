//! Original token custody shared by native template document consumers.

use vize_l0::Vec;
use vize_l1::Token;

use super::{Doc, TemplateRefusal};

pub(in crate::native_doc) struct Cursor<'a> {
    pub(in crate::native_doc) source: &'a str,
    pub(in crate::native_doc) offset: usize,
}

impl Cursor<'_> {
    pub(in crate::native_doc) fn token(
        &mut self,
        token: &Token<'_>,
    ) -> Result<(), TemplateRefusal> {
        if token.is_missing() {
            return Err(TemplateRefusal::Recovered {
                offset: self.offset,
            });
        }
        for piece in [token.leading, token.text] {
            let end =
                self.offset
                    .checked_add(piece.len())
                    .ok_or(TemplateRefusal::SourceMismatch {
                        offset: self.offset,
                    })?;
            let expected =
                self.source
                    .get(self.offset..end)
                    .ok_or(TemplateRefusal::SourceMismatch {
                        offset: self.offset,
                    })?;
            if !piece.is_empty() && !core::ptr::eq(expected.as_ptr(), piece.as_ptr()) {
                return Err(TemplateRefusal::SourceMismatch {
                    offset: self.offset,
                });
            }
            self.offset = end;
        }
        Ok(())
    }

    pub(in crate::native_doc) fn trivia(
        &mut self,
        token: &Token<'_>,
    ) -> Result<(), TemplateRefusal> {
        if !token.leading.bytes().all(|byte| byte.is_ascii_whitespace()) {
            return Err(TemplateRefusal::Recovered {
                offset: self.offset,
            });
        }
        self.token(token)
    }
}

pub(in crate::native_doc) fn verbatim<'a>(parts: &mut Vec<'a, Doc<'a>>, token: &Token<'a>) {
    parts.push(Doc::text(token.leading));
    parts.push(Doc::text(token.text));
}
