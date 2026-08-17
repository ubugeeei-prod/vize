//! Authored attribute-value spans for `vue/html-quotes`.
//!
//! `eslint-plugin-vue` reports this rule on the attribute *value node*, whose
//! range covers the delimiters (`'foo'`, not `foo`) and, for an unquoted value,
//! the bare text. It reads that value straight from the source text rather than
//! from any parsed expression, so the same span is produced for a plain
//! attribute and for a directive.
//!
//! Relief exposes only the inner text of a plain attribute value and a parsed
//! expression for a directive, neither of which addresses the delimiters, so the
//! span is recovered here by scanning the attribute's own source range.

use vize_relief::SourceLocation;

/// An attribute value as authored, addressed in the linted source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ValueRange {
    /// Offset of the first byte of the value, delimiter included.
    pub(super) start: u32,
    /// Offset one past the last byte of the value, delimiter included.
    pub(super) end: u32,
    /// The opening delimiter, or `None` when the value is unquoted.
    pub(super) quote: Option<u8>,
}

impl ValueRange {
    /// The value text without its delimiters.
    pub(super) fn inner(&self) -> (u32, u32) {
        match self.quote {
            Some(_) => (self.start + 1, self.end - 1),
            None => (self.start, self.end),
        }
    }
}

/// Locate the authored value of the attribute occupying `loc`.
///
/// Returns `None` for a valueless attribute (`disabled`, `:foo`), which upstream
/// skips through its `VAttribute[value!=null]` selector.
pub(super) fn value_range(source: &str, loc: &SourceLocation) -> Option<ValueRange> {
    let start = usize::try_from(loc.start.offset).ok()?;
    let end = usize::try_from(loc.end.offset).ok()?;
    if start >= end || end > source.len() {
        return None;
    }
    let bytes = source.as_bytes();
    // The first `=` inside the attribute separates the name — including a
    // directive's argument and modifiers, none of which may contain one — from
    // the value, so any later `=` belongs to the value itself.
    let separator = bytes
        .get(start..end)?
        .iter()
        .position(|byte| *byte == b'=')?
        + start;
    let mut cursor = separator + 1;
    while cursor < end && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if cursor >= end {
        return None;
    }
    let quote = bytes[cursor];
    if quote != b'"' && quote != b'\'' {
        return Some(ValueRange {
            start: u32::try_from(cursor).ok()?,
            end: loc.end.offset,
            quote: None,
        });
    }
    let closing = bytes
        .get(cursor + 1..end)?
        .iter()
        .position(|byte| *byte == quote)?
        + cursor
        + 1;
    Some(ValueRange {
        start: u32::try_from(cursor).ok()?,
        end: u32::try_from(closing + 1).ok()?,
        quote: Some(quote),
    })
}

#[cfg(test)]
mod tests {
    use super::{ValueRange, value_range};
    use vize_relief::{Position, SourceLocation};

    fn whole(source: &str) -> SourceLocation {
        let end = u32::try_from(source.len()).expect("probe source fits in u32");
        SourceLocation {
            start: Position {
                offset: 0,
                line: 1,
                column: 1,
            },
            end: Position {
                offset: end,
                line: 1,
                column: end + 1,
            },
            source: Default::default(),
        }
    }

    fn range_of(source: &str) -> Option<ValueRange> {
        value_range(source, &whole(source))
    }

    #[test]
    fn single_quoted_value_covers_both_delimiters() {
        let source = "class='foo'";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 6,
                end: 11,
                quote: Some(b'\''),
            }
        );
        assert_eq!(&source[range.start as usize..range.end as usize], "'foo'");
        assert_eq!(range.inner(), (7, 10));
    }

    #[test]
    fn double_quoted_value_keeps_an_embedded_single_quote() {
        let source = "title=\"don't\"";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 6,
                end: 13,
                quote: Some(b'"'),
            }
        );
    }

    #[test]
    fn empty_quoted_value_is_the_two_delimiters() {
        let source = "title=''";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 6,
                end: 8,
                quote: Some(b'\''),
            }
        );
        assert_eq!(range.inner(), (7, 7));
    }

    #[test]
    fn unquoted_value_reports_no_delimiter() {
        let source = "class=bare";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 6,
                end: 10,
                quote: None,
            }
        );
        assert_eq!(range.inner(), (6, 10));
    }

    #[test]
    fn whitespace_around_the_separator_is_skipped() {
        let source = "class = 'foo'";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 8,
                end: 13,
                quote: Some(b'\''),
            }
        );
    }

    #[test]
    fn a_directive_value_is_found_past_argument_and_modifiers() {
        let source = "v-on:click.prevent='go()'";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 19,
                end: 25,
                quote: Some(b'\''),
            }
        );
    }

    #[test]
    fn an_equals_sign_inside_the_value_is_not_the_separator() {
        let source = "@click='a = b'";
        let range = range_of(source).expect("value range");
        assert_eq!(
            range,
            ValueRange {
                start: 7,
                end: 14,
                quote: Some(b'\''),
            }
        );
    }

    #[test]
    fn a_valueless_attribute_has_no_range() {
        assert_eq!(range_of("disabled"), None);
        assert_eq!(range_of(":foo"), None);
    }

    #[test]
    fn an_unterminated_quote_has_no_range() {
        assert_eq!(range_of("class='foo"), None);
    }
}
