//! Map decoded JavaScript token ranges back to the complete authored token.

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
        span: Span,
        context: EntityContext,
    ) -> Option<Self> {
        let authored = source.get(span.start as usize..span.end as usize)?;
        if authored == content {
            return Some(Self {
                start: span.start,
                offsets: None,
            });
        }
        // Only decoded expressions allocate a map. Verify the entire decoded
        // input before trusting its offsets; a generated expression has no
        // authority to report against an unrelated authored span.
        let mut decoded = String::default();
        let mut offsets = vec![0];
        let mut at = 0;
        while let Some(rest) = authored.get(at..)
            && let Some(first) = rest.chars().next()
        {
            let len = if let Some((value, len)) = decode_one(rest.as_bytes(), context) {
                value.for_each(|ch| {
                    decoded.push(ch);
                    offsets.extend(std::iter::repeat_n(at as u32, ch.len_utf8()));
                });
                offsets.pop();
                len
            } else {
                decoded.push(first);
                offsets.extend(std::iter::repeat_n(at as u32, first.len_utf8() - 1));
                first.len_utf8()
            };
            at += len;
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
