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

/// Decode each authored HTML reference exactly once. This is the consumer's
/// attribute context, not a DATA classification from an amp's neighboring bytes.
fn decode_attribute(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = rest.find('&') {
        output.push_str(rest.get(..index).unwrap_or_default());
        let tail = rest.get(index..).unwrap_or_default();
        if let Some((entity, width)) = decode_one(tail.as_bytes(), EntityContext::Attribute) {
            entity.for_each(|character| output.push(character));
            rest = tail.get(width..).unwrap_or_default();
        } else {
            output.push('&');
            rest = tail.get(1..).unwrap_or_default();
        }
    }
    output.push_str(rest);
    output
}

fn reference_positions(value: &str) -> SmallVec<[usize; 2]> {
    value
        .match_indices('&')
        .filter_map(|(index, _)| {
            decode_one(value.get(index..)?.as_bytes(), EntityContext::Attribute).map(|_| index)
        })
        .collect()
}

/// Preserve already-correct emission byte for byte. Only a recognized reference
/// in ACTUAL printed JavaScript can change the next HTML consumer. Comparing
/// complete canonical JS after each HTML decode distinguishes existing encoded
/// source from newly exposed DATA, including JS escape normalization.
///
/// On a real change, print the actual original consumer program, then protect
/// reference candidates in that printed output. No source offsets are guessed
/// across the JavaScript printer; operators such as `value & copy` still format.
pub(super) fn protect_formatted_reference_data(
    formatted: crate::script::FormattedExpression,
    authored: &str,
    options: &crate::options::FormatOptions,
    attribute_depth: usize,
    name: &str,
) -> Option<crate::script::FormattedExpression> {
    if reference_positions(&formatted.code).is_empty() {
        return Some(formatted);
    }
    let original = decode_attribute(authored.trim());
    let consumed = decode_attribute(&formatted.code);
    let original_canonical = crate::script::format_js_expression_in_attribute(&original, options)?;
    if crate::script::format_js_expression_in_attribute(&consumed, options)
        .is_some_and(|candidate| candidate == original_canonical)
    {
        return Some(formatted);
    }
    let mut repaired = crate::script::format_js_expression_in_attribute_with_layout(
        &original,
        options,
        attribute_depth,
        name,
        authored,
        authored.trim(),
    )?;
    if repaired.retained_bare_sequence {
        // A changed parse shape cannot borrow the old bare branch's offsets.
        return None;
    }
    let positions = reference_positions(&repaired.code);
    repaired.code = super::super::literal_lines::encode_reference_data(repaired.code, &positions);
    Some(repaired)
}

#[cfg(test)]
mod tests {
    use super::protect_formatted_reference_data;
    use crate::{FormatOptions, script};

    #[test]
    fn actual_printed_refs_preserve_data_without_suppressing_operator_formatting() {
        let options = FormatOptions::default();
        for (authored, expected) in [
            ("value &amp;copy", "value & copy"),
            ("'&amp;quot;'", "'&amp;quot;'"),
            ("'&amp;apos;'", "'&amp;apos;'"),
            // The actual pinned printer keeps this harmless JS escape. It
            // does not form an HTML reference and needs no invented rewrite.
            (r"'&amp;qu\ot;'", r"'&qu\ot;'"),
            (r"'&qu\ot;'", r"'&qu\ot;'"),
            ("'&#38;quot;'", "'&#38;quot;'"),
            ("'&lt;'", "'&lt;'"),
            ("left &amp;&amp; right", "left && right"),
            ("'&amp;quot;'   +   'tail'", "'&amp;quot;' + 'tail'"),
        ] {
            let decoded = super::decode_expression_attribute_entities(authored);
            let expression = decoded
                .as_ref()
                .map_or(authored, |value| value.code.as_str());
            let formatted = script::format_js_expression_in_attribute_with_layout(
                expression, &options, 0, ":title", authored, authored,
            );
            let result = formatted.and_then(|formatted| {
                protect_formatted_reference_data(formatted, authored, &options, 0, ":title")
            });
            assert_eq!(
                result.as_ref().map(|value| value.code.as_str()),
                Some(expected),
                "{authored}"
            );
        }
    }

    #[test]
    fn an_unparseable_true_consumer_cannot_borrow_the_partial_decode_result() {
        let authored = "'&NewLine;'";
        let options = FormatOptions::default();
        let formatted = script::format_js_expression_in_attribute_with_layout(
            authored, &options, 0, ":title", authored, authored,
        );
        assert!(formatted.is_some());
        assert!(
            formatted
                .and_then(|formatted| protect_formatted_reference_data(
                    formatted, authored, &options, 0, ":title",
                ))
                .is_none()
        );
    }
}
