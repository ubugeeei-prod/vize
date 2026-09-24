//! Restore leading zeroes omitted by lightningcss's non-minified printer.
//!
//! Only CSS number tokens are changed. Strings, comments and URL bodies can
//! contain text such as `.5` that must remain byte-for-byte unchanged.

use vize_s0::{String, ToCompactString};

pub(super) fn add_leading_zero_to_fractional_numbers(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output: Option<String> = None;
    let mut copied_through = 0;
    let mut index = 0;

    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'"' | b'\'' => index = skip_string(bytes, index + 1, byte),
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = memchr::memmem::find(bytes.get(index + 2..).unwrap_or_default(), b"*/")
                    .map_or(bytes.len(), |end| index + 2 + end + 2);
            }
            b'u' | b'U' => {
                if let Some(body_start) = url_body_start(bytes, index) {
                    index = skip_url(bytes, body_start);
                } else {
                    index += 1;
                }
            }
            b'.' if bytes.get(index + 1).is_some_and(u8::is_ascii_digit)
                && is_number_start(bytes, index) =>
            {
                let Some(segment) = source.get(copied_through..index) else {
                    return source.to_compact_string();
                };
                let output = output.get_or_insert_with(|| String::with_capacity(source.len() + 4));
                output.push_str(segment);
                output.push('0');
                copied_through = index;
                index += 1;
            }
            _ => index += 1,
        }
    }

    let Some(mut output) = output else {
        return source.to_compact_string();
    };
    let Some(remainder) = source.get(copied_through..) else {
        return source.to_compact_string();
    };
    output.push_str(remainder);
    output
}

fn is_number_start(bytes: &[u8], dot: usize) -> bool {
    let Some(&previous) = dot.checked_sub(1).and_then(|index| bytes.get(index)) else {
        return true;
    };
    if matches!(previous, b'+' | b'-') {
        return dot <= 1
            || dot
                .checked_sub(2)
                .and_then(|index| bytes.get(index))
                .copied()
                .is_some_and(is_number_boundary);
    }
    is_number_boundary(previous)
}

fn is_number_boundary(byte: u8) -> bool {
    byte.is_ascii()
        && !byte.is_ascii_alphanumeric()
        && !matches!(byte, b'_' | b'-' | b'.' | b'\\' | b'#')
}

fn skip_string(bytes: &[u8], mut index: usize, quote: u8) -> usize {
    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' {
            index += 2;
        } else if byte == quote {
            return index + 1;
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn url_body_start(bytes: &[u8], start: usize) -> Option<usize> {
    if start > 0
        && !start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .copied()
            .is_some_and(is_number_boundary)
    {
        return None;
    }
    if !bytes.get(start..start + 3)?.eq_ignore_ascii_case(b"url") {
        return None;
    }
    let mut index = start + 3;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    (bytes.get(index) == Some(&b'(')).then_some(index + 1)
}

fn skip_url(bytes: &[u8], mut index: usize) -> usize {
    let mut depth = 1;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'"' | b'\'' => index = skip_string(bytes, index + 1, byte),
            b'\\' => index += 2,
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth -= 1;
                index += 1;
                if depth == 0 {
                    return index;
                }
            }
            _ => index += 1,
        }
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::add_leading_zero_to_fractional_numbers;

    #[test]
    fn only_css_number_tokens_get_a_leading_zero() {
        let css = r#".foo { opacity: .5; margin: -.25em .75px; transition: opacity .2s cubic-bezier(.4, 0, .2, 1); content: ".5"; background: url(https://example.test/.5/icon.svg); } /* .5 */"#;
        let expected = r#".foo { opacity: 0.5; margin: -0.25em 0.75px; transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1); content: ".5"; background: url(https://example.test/.5/icon.svg); } /* .5 */"#;
        assert_eq!(
            add_leading_zero_to_fractional_numbers(css).as_str(),
            expected
        );
    }

    #[test]
    fn does_not_change_identifiers_or_existing_numbers() {
        let css = r#".foo\.5 { --data: .5; --élément.5: x; width: 0.5px; content: '.5'; }"#;
        let expected = r#".foo\.5 { --data: 0.5; --élément.5: x; width: 0.5px; content: '.5'; }"#;
        assert_eq!(
            add_leading_zero_to_fractional_numbers(css).as_str(),
            expected
        );
    }

    #[test]
    fn number_at_start_of_source_is_rewritten() {
        assert_eq!(
            add_leading_zero_to_fractional_numbers(".5em").as_str(),
            "0.5em"
        );
    }

    #[test]
    fn fractional_number_after_unicode_text_is_rewritten() {
        assert_eq!(
            add_leading_zero_to_fractional_numbers(".foo { --élément: .5; }").as_str(),
            ".foo { --élément: 0.5; }"
        );
    }
}
