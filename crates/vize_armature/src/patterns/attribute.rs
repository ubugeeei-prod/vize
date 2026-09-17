//! Rebase decoded pattern spans through the same entity decoder as HTML attributes.

use super::{MatchArm, MatchPattern, PatternKind, PatternSyntaxError, parse_match_pattern};
use crate::tokenizer::entity_decode::try_decode_entity;
use htmlize::Context;
use oxc_span::Span;
use vize_s0::String;

struct Attribute {
    text: String,
    offsets: Vec<u32>,
}

impl Attribute {
    fn new(source: &str) -> Self {
        let mut text = String::default();
        let mut offsets = vec![0];
        let mut at = 0;
        while at < source.len() {
            let (ch, len) = try_decode_entity(&source.as_bytes()[at..], Context::Attribute)
                .unwrap_or_else(|| {
                    let ch = source[at..].chars().next().expect("character boundary");
                    (ch, ch.len_utf8())
                });
            text.push(ch);
            offsets.extend(std::iter::repeat_n(at as u32, ch.len_utf8() - 1));
            at += len;
            offsets.push(at as u32);
        }
        Self { text, offsets }
    }

    fn offset(&self, offset: u32) -> u32 {
        self.offsets
            .get(offset as usize)
            .copied()
            .unwrap_or_else(|| *self.offsets.last().expect("initial offset"))
    }

    fn span(&self, span: &mut Span) {
        span.start = self.offset(span.start);
        span.end = self.offset(span.end);
    }

    fn pattern(&self, pattern: &mut MatchPattern) {
        self.span(&mut pattern.span);
        match &mut pattern.kind {
            PatternKind::Literal(value) | PatternKind::Value(value) => self.span(&mut value.span),
            PatternKind::Binding(binding) => self.span(&mut binding.span),
            PatternKind::As { pattern, binding } => {
                self.pattern(pattern);
                self.span(&mut binding.span);
            }
            PatternKind::Or(patterns) => {
                for pattern in patterns {
                    self.pattern(pattern);
                }
            }
            PatternKind::Object { properties, rest } => {
                for property in properties {
                    self.span(&mut property.span);
                    self.span(&mut property.key.span);
                    self.pattern(&mut property.pattern);
                }
                self.rest(rest);
            }
            PatternKind::Array { elements, rest } => {
                for element in elements {
                    self.pattern(element);
                }
                self.rest(rest);
            }
            PatternKind::Wildcard => {}
        }
    }

    fn rest(&self, rest: &mut Option<super::PatternRest>) {
        if let Some(rest) = rest {
            self.span(&mut rest.span);
            if let Some(binding) = &mut rest.binding {
                self.span(&mut binding.span);
            }
        }
    }
}

/// Parse an authored attribute value, retaining decoded expression text while
/// mapping every syntax/declaration span to its original UTF-8 byte range.
pub fn parse_match_attribute(source: &str) -> Result<MatchArm, PatternSyntaxError> {
    if !source.contains('&') {
        return parse_match_pattern(source);
    }
    let attribute = Attribute::new(source);
    let mut arm = parse_match_pattern(attribute.text.as_str()).map_err(|mut error| {
        error.offset = attribute.offset(error.offset);
        error
    })?;
    attribute.pattern(&mut arm.pattern);
    for binding in &mut arm.bindings {
        attribute.span(&mut binding.span);
    }
    if let Some(guard) = &mut arm.guard {
        attribute.span(&mut guard.span);
    }
    Ok(arm)
}

/// Map a decoded expression byte offset back into an authored attribute value.
pub fn attribute_source_offset(source: &str, decoded: u32) -> u32 {
    if source.contains('&') {
        Attribute::new(source).offset(decoded)
    } else {
        decoded.min(source.len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entities_preserve_binding_guard_and_error_offsets() {
        let source = "{ kind: &quot;ok&quot;, const rows } as whole if (rows.length &gt; missing)";
        let arm = parse_match_attribute(source).unwrap();
        for binding in &arm.bindings {
            assert_eq!(
                &source[binding.span.start as usize..binding.span.end as usize],
                binding.name.as_str()
            );
        }
        let guard = arm.guard.unwrap();
        assert_eq!(guard.text, "rows.length > missing");
        assert_eq!(
            &source[guard.span.start as usize..guard.span.end as usize],
            "rows.length &gt; missing"
        );
        let bad = "{ kind: &quot;ok&quot;, const rows, const rows }";
        let error = parse_match_attribute(bad).unwrap_err();
        assert!(error.offset as usize >= bad.rfind("rows").unwrap());
    }

    #[test]
    fn offsets_are_bytes_not_unicode_scalar_or_utf16_counts() {
        let source = "&#x1f680; + &#x1f680; + &gt;";
        assert_eq!(attribute_source_offset(source, 7), 12);
        assert_eq!(attribute_source_offset(source, 14), 24);
        assert_eq!(attribute_source_offset(source, 15), source.len() as u32);
        let source = "{ &quot;x&quot;: const \u{00e9} }";
        let arm = parse_match_attribute(source).unwrap();
        let binding = &arm.bindings[0];
        assert_eq!(
            &source[binding.span.start as usize..binding.span.end as usize],
            "\u{00e9}"
        );
    }
}
