//! Project decoded Vue expression bytes back into authored template source.

use vize_l0::String;
use vize_l1::markup::entity::{EntityContext, decode_one};

/// Authored byte boundaries for one decoded expression window.
struct DecodedSource {
    offsets: Vec<u32>,
}

impl DecodedSource {
    /// Decode one complete expression using the HTML context that produced it.
    fn new(source: &str, context: EntityContext) -> Self {
        let mut decoded = String::default();
        let mut offsets = vec![0];
        let mut at = 0;
        while let Some(rest) = source.get(at..)
            && let Some(first) = rest.chars().next()
        {
            let len = if let Some((entity, len)) = decode_one(rest.as_bytes(), context) {
                entity.for_each(|ch| {
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
        Self { offsets }
    }

    /// Resolve a decoded UTF-8 boundary to its authored byte boundary.
    fn offset(&self, decoded: u32) -> u32 {
        self.offsets
            .get(decoded as usize)
            .copied()
            .or_else(|| self.offsets.last().copied())
            .unwrap_or_default()
    }
}

/// Cached decoded-to-authored byte projection for one expression fragment.
pub struct AuthoredSourceOffsets {
    decoded: Option<DecodedSource>,
    source_len: u32,
}

impl AuthoredSourceOffsets {
    /// Build an offset projection for an authored directive value.
    pub fn attribute(source: &str) -> Self {
        Self::new(source, EntityContext::Attribute)
    }

    /// Build an offset projection for authored template text.
    pub fn text(source: &str) -> Self {
        Self::new(source, EntityContext::Text)
    }

    /// Map one decoded UTF-8 boundary into the authored source fragment.
    pub fn offset(&self, decoded: u32) -> u32 {
        self.decoded
            .as_ref()
            .map_or(decoded.min(self.source_len), |source| {
                source.offset(decoded)
            })
    }

    fn new(source: &str, context: EntityContext) -> Self {
        Self {
            decoded: source
                .contains('&')
                .then(|| DecodedSource::new(source, context)),
            source_len: source.len() as u32,
        }
    }
}

/// Map a decoded directive-value byte offset into authored attribute source.
pub fn attribute_source_offset(source: &str, decoded: u32) -> u32 {
    AuthoredSourceOffsets::attribute(source).offset(decoded)
}

/// Map a decoded interpolation byte offset into authored template text.
pub fn text_source_offset(source: &str, decoded: u32) -> u32 {
    AuthoredSourceOffsets::text(source).offset(decoded)
}

#[cfg(test)]
mod tests {
    use super::{AuthoredSourceOffsets, attribute_source_offset, text_source_offset};

    #[test]
    fn entity_offsets_are_authored_utf8_bytes_in_each_html_context() {
        let source = "🌸 0 &lt; open";
        let decoded = "🌸 0 < open";
        let open = decoded.find("open").unwrap() as u32;
        let authored = source.find("open").unwrap() as u32;
        assert_eq!(text_source_offset(source, open), authored);
        assert_eq!(attribute_source_offset(source, open), authored);
        let offsets = AuthoredSourceOffsets::text(source);
        assert_eq!(offsets.offset(open), authored);
        assert_eq!(offsets.offset(decoded.len() as u32), source.len() as u32);
    }
}
