//! Authored directive spelling read off L1: the dialect's own attribute-name
//! decomposition supplies argument and modifier coordinates, and values are
//! judged after HTML entity decoding, as the compiler reads them.

use super::l2_range;
use crate::ir::ByteRange;
use memchr::memchr;
use vize_l1::markup::Attribute;
use vize_l1::markup::directive::{ArgSyntax, DirectiveName, DirectiveSyntax, VueDirectives};
use vize_l1::markup::entity::{EntityContext, decode_one};

/// Decompose `attr`'s name, whose first byte sits at absolute `offset`.
fn decompose(attr: &Attribute<'_>, offset: u32) -> Option<DirectiveName> {
    VueDirectives
        .decompose(attr.name.text, offset)
        .ok()
        .flatten()
}

/// The argument's range: the static name or the dynamic bracket contents.
pub(super) fn argument_range(attr: &Attribute<'_>, offset: u32) -> Option<ByteRange> {
    match decompose(attr, offset)?.arg? {
        ArgSyntax::Static(span) | ArgSyntax::Dynamic(span) => Some(l2_range(span)),
    }
}

/// Visit the spelled, non-empty modifier segments with their ranges.
pub(super) fn walk_modifier_ranges<'a>(
    attr: &Attribute<'a>,
    offset: u32,
    visitor: &mut impl FnMut(&'a str, ByteRange),
) {
    let Some(name) = decompose(attr, offset) else {
        return;
    };
    let run_start = name.modifiers.start;
    let local = run_start.saturating_sub(offset) as usize
        ..name.modifiers.end.saturating_sub(offset) as usize;
    let Some(run) = attr.name.text.get(local) else {
        return;
    };
    let mut cursor = run_start;
    for part in run.split('.') {
        let end = cursor + part.len() as u32;
        if !part.is_empty() {
            visitor(part, ByteRange::new(cursor, end));
        }
        cursor = end + 1;
    }
}

/// Whether the authored value holds anything but whitespace once decoded.
pub(super) fn has_value(attr: &Attribute<'_>) -> bool {
    attr.value
        .as_ref()
        .is_some_and(|value| !is_blank(value.content.text))
}

fn is_blank(mut raw: &str) -> bool {
    while let Some(at) = memchr(b'&', raw.as_bytes()) {
        let (literal, candidate) = raw.split_at(at);
        if !literal.trim().is_empty() {
            return false;
        }
        let Some((entity, consumed)) = decode_one(candidate.as_bytes(), EntityContext::Attribute)
        else {
            return false;
        };
        let mut blank = true;
        entity.for_each(|character| blank &= character.is_whitespace());
        if !blank {
            return false;
        }
        raw = candidate.get(consumed..).unwrap_or_default();
    }
    raw.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::is_blank;

    #[test]
    fn blankness_reads_decoded_entities() {
        for (raw, blank) in [
            ("", true),
            ("  \t", true),
            ("&#32;", true),
            ("&nbsp; ", true),
            ("&amp;", false),
            ("& ", false),
            (" a ", false),
            ("&#32;x", false),
        ] {
            assert_eq!(is_blank(raw), blank, "{raw:?}");
        }
    }
}
