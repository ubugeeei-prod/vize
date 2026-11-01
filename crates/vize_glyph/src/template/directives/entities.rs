//! Authored directive HTML decoding and decoded amp DATA provenance.

use vize_l0::{SmallVec, String};
use vize_l1::markup::entity::{EntityContext, decode_one};

pub(super) struct DecodedExpression {
    pub(super) code: String,
    pub(super) amp_positions: SmallVec<[usize; 2]>,
}

pub(super) fn decode_expression_attribute_entities(value: &str) -> Option<DecodedExpression> {
    if !value.contains('&') {
        return None;
    }

    let mut decoded = String::with_capacity(value.len());
    let mut amp_positions = SmallVec::new();
    let mut changed = false;
    let mut rest = value;
    while !rest.is_empty() {
        if let Some(tail) = rest.strip_prefix("&quot;") {
            decoded.push('"');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest
            .strip_prefix("&#34;")
            .or_else(|| rest.strip_prefix("&#x22;"))
            .or_else(|| rest.strip_prefix("&#X22;"))
        {
            decoded.push('"');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest.strip_prefix("&apos;") {
            decoded.push('\'');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest
            .strip_prefix("&#39;")
            .or_else(|| rest.strip_prefix("&#x27;"))
            .or_else(|| rest.strip_prefix("&#X27;"))
        {
            decoded.push('\'');
            rest = tail;
            changed = true;
        } else if let Some(tail) = rest.strip_prefix("&amp;") {
            amp_positions.push(decoded.len());
            decoded.push('&');
            rest = tail;
            changed = true;
        } else {
            let Some(ch) = rest.chars().next() else {
                break;
            };
            decoded.push(ch);
            rest = rest.get(ch.len_utf8()..).unwrap_or_default();
        }
    }

    changed.then_some(DecodedExpression {
        code: decoded,
        amp_positions,
    })
}

impl DecodedExpression {
    /// The decoded amp belongs to string/comment DATA only if another HTML
    /// parse would consume it as a reference. Ordinary operators and unknown
    /// references do not need this fallback. The formatter has no mapping from
    /// these original byte offsets to arbitrary reprinted JavaScript offsets.
    pub(super) fn reference_data_would_decode(&self) -> bool {
        self.amp_positions.iter().any(|&index| {
            self.code.as_bytes().get(index) == Some(&b'&')
                && decode_one(
                    self.code.get(index..).unwrap_or_default().as_bytes(),
                    EntityContext::Attribute,
                )
                .is_some()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::decode_expression_attribute_entities;

    #[test]
    fn only_decoded_amp_data_that_forms_an_html_reference_needs_authored_bytes() {
        for source in [
            "'&amp;quot;'",
            "read('&amp;#34;')",
            "read('&lt; &amp;quot;')",
        ] {
            assert!(
                decode_expression_attribute_entities(source)
                    .is_some_and(|decoded| decoded.reference_data_would_decode())
            );
        }
        for source in [
            "left &amp;&amp; right",
            "'a&amp;b'",
            "'&amp;unknown;'",
            "'&#38;quot;'",
            "prefix + &quot;ab&quot;",
        ] {
            assert!(
                !decode_expression_attribute_entities(source)
                    .is_some_and(|decoded| decoded.reference_data_would_decode())
            );
        }
    }
}
