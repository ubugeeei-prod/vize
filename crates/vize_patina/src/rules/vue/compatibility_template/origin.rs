//! Authored-range recovery for decoded template expressions.

use crate::ir::ByteRange;
use oxc_span::Span;
use vize_l0::String;
use vize_l1::markup::entity::{EntityContext, decode_one};

pub(super) struct Origin {
    start: u32,
    offsets: Option<Vec<u32>>,
}

impl Origin {
    pub(super) fn new(
        source: &str,
        content: &str,
        span: vize_l0::Span,
        context: EntityContext,
    ) -> Option<Self> {
        let authored = source.get(span.start as usize..span.end as usize)?;
        if authored == content {
            return Some(Self {
                start: span.start,
                offsets: None,
            });
        }
        let mut decoded = String::default();
        let mut offsets = vec![0];
        let mut at = 0;
        while let Some(rest) = authored.get(at..)
            && let Some(first) = rest.chars().next()
        {
            let length = if let Some((value, length)) = decode_one(rest.as_bytes(), context) {
                value.for_each(|character| {
                    decoded.push(character);
                    offsets.extend(std::iter::repeat_n(at as u32, character.len_utf8()));
                });
                offsets.pop();
                length
            } else {
                decoded.push(first);
                offsets.extend(std::iter::repeat_n(at as u32, first.len_utf8() - 1));
                first.len_utf8()
            };
            at += length;
            offsets.push(at as u32);
        }
        (decoded == content).then_some(Self {
            start: span.start,
            offsets: Some(offsets),
        })
    }

    pub(super) fn range(&self, span: Span) -> Option<ByteRange> {
        let offset = |position: u32| match &self.offsets {
            Some(offsets) => offsets.get(position as usize).copied(),
            None => Some(position),
        };
        Some(ByteRange {
            start: self.start.checked_add(offset(span.start)?)?,
            end: self.start.checked_add(offset(span.end)?)?,
        })
    }
}
