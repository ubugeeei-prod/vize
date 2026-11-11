//! HTML numeric character reference errors.
//!
//! <https://html.spec.whatwg.org/multipage/parsing.html#numeric-character-reference-end-state>

use vize_l0::ErrorCode;

/// A numeric reference at the start of `candidate` (which starts with `&#`):
/// its length and the errors the HTML tokenizer reports for it.
pub(super) fn numeric_reference(candidate: &[u8]) -> (usize, [Option<ErrorCode>; 2]) {
    let (digits_start, radix) = match candidate.get(2) {
        Some(b'x' | b'X') => (3, 16),
        _ => (2, 10),
    };
    let digits = candidate
        .get(digits_start..)
        .unwrap_or_default()
        .iter()
        .take_while(|byte| char::from(**byte).is_digit(radix))
        .count();
    if digits == 0 {
        return (
            digits_start,
            [
                Some(ErrorCode::AbsenceOfDigitsInNumericCharacterReference),
                None,
            ],
        );
    }
    let end = digits_start + digits;
    let value = candidate
        .get(digits_start..end)
        .unwrap_or_default()
        .iter()
        .filter_map(|byte| char::from(*byte).to_digit(radix))
        .fold(0u32, |value, digit| {
            value
                .saturating_mul(radix)
                .saturating_add(digit)
                .min(0x11_0000)
        });
    let terminated = candidate.get(end) == Some(&b';');
    let missing_semicolon =
        (!terminated).then_some(ErrorCode::MissingSemicolonAfterCharacterReference);
    (
        end + usize::from(terminated),
        [value_error(value), missing_semicolon],
    )
}

fn value_error(value: u32) -> Option<ErrorCode> {
    match value {
        0 => Some(ErrorCode::NullCharacterReference),
        0x11_0000.. => Some(ErrorCode::CharacterReferenceOutsideUnicodeRange),
        0xD800..=0xDFFF => Some(ErrorCode::SurrogateCharacterReference),
        0xFDD0..=0xFDEF => Some(ErrorCode::NoncharacterCharacterReference),
        _ if value & 0xFFFE == 0xFFFE => Some(ErrorCode::NoncharacterCharacterReference),
        // A control other than ASCII whitespace, or a carriage return.
        0x0D | 0x01..=0x08 | 0x0B | 0x0E..=0x1F | 0x7F..=0x9F => {
            Some(ErrorCode::ControlCharacterReference)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::numeric_reference;
    use vize_l0::ErrorCode;

    #[test]
    fn reports_the_html_numeric_reference_errors() {
        for (source, consumed, expected) in [
            (
                "&#;",
                2,
                Some(ErrorCode::AbsenceOfDigitsInNumericCharacterReference),
            ),
            (
                "&#x;",
                3,
                Some(ErrorCode::AbsenceOfDigitsInNumericCharacterReference),
            ),
            (
                "&#1234567;",
                10,
                Some(ErrorCode::CharacterReferenceOutsideUnicodeRange),
            ),
            ("&#0;", 4, Some(ErrorCode::NullCharacterReference)),
            ("&#0003;", 7, Some(ErrorCode::ControlCharacterReference)),
            ("&#13;", 5, Some(ErrorCode::ControlCharacterReference)),
            ("&#xD800;", 8, Some(ErrorCode::SurrogateCharacterReference)),
            (
                "&#xFFFE;",
                8,
                Some(ErrorCode::NoncharacterCharacterReference),
            ),
            (
                "&#x10FFFF;",
                10,
                Some(ErrorCode::NoncharacterCharacterReference),
            ),
            (
                "&#xFDD0;",
                8,
                Some(ErrorCode::NoncharacterCharacterReference),
            ),
            ("&#9;", 4, None),
            ("&#65;", 5, None),
            ("&#x1F600;", 9, None),
        ] {
            let (length, [value, semicolon]) = numeric_reference(source.as_bytes());
            assert_eq!(
                (length, value, semicolon),
                (consumed, expected, None),
                "{source}"
            );
        }
        let (length, [value, semicolon]) = numeric_reference(b"&#65 b");
        assert_eq!(length, 4);
        assert_eq!(value, None);
        assert_eq!(
            semicolon,
            Some(ErrorCode::MissingSemicolonAfterCharacterReference)
        );
    }
}
