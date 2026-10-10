//! Vue's static class spelling from the retained, already decoded attribute.

use alloc::{borrow::Cow, string::String};
use vize_l2::op::Attribute;

pub(super) fn value<'a>(attribute: &Attribute<'a>) -> Cow<'a, str> {
    normalize(attribute.name, attribute.value.unwrap_or_default())
}

pub(super) fn normalize<'a>(name: &str, value: &'a str) -> Cow<'a, str> {
    if name != "class" {
        return Cow::Borrowed(value);
    }
    // Vue condenses only its five HTML whitespace characters, then applies
    // ECMAScript trim. Rust's Unicode whitespace includes U+0085 and excludes
    // U+FEFF, so str::trim/is_whitespace would change authored class names.
    let value = value.trim_matches(trim_space);
    let mut previous_space = false;
    let unchanged = value.bytes().all(|byte| {
        let space = html_space(byte);
        let unchanged = !space || (byte == b' ' && !previous_space);
        previous_space = space;
        unchanged
    });
    if unchanged {
        return Cow::Borrowed(value);
    }
    let mut normalized = String::with_capacity(value.len());
    let mut previous_space = false;
    for character in value.chars() {
        let space = character.is_ascii() && html_space(character as u8);
        if !space {
            normalized.push(character);
        } else if !previous_space {
            normalized.push(' ');
        }
        previous_space = space;
    }
    Cow::Owned(normalized)
}

fn html_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\x0c' | b'\r')
}

fn trim_space(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_classes_and_ordinary_values_keep_borrowed_storage() {
        assert!(matches!(
            normalize("class", "ready active"),
            Cow::Borrowed(_)
        ));
        assert!(matches!(
            normalize("title", " a\t b "),
            Cow::Borrowed(" a\t b ")
        ));
        assert!(matches!(
            normalize("class", "\u{feff}ready\u{feff}"),
            Cow::Borrowed("ready")
        ));
    }

    #[test]
    fn html_condensation_and_ecmascript_trim_preserve_other_class_characters() {
        assert_eq!(normalize("class", " a\t b\nc\rd\u{c}e "), "a b c d e");
        assert_eq!(normalize("class", "\u{a0}a\u{85}b\u{feff}"), "a\u{85}b");
        assert_eq!(
            normalize("class", "a\u{a0}b\u{feff}c\u{200b}d"),
            "a\u{a0}b\u{feff}c\u{200b}d"
        );
        assert_eq!(normalize("class", "a\u{b}b"), "a\u{b}b");
    }
}
