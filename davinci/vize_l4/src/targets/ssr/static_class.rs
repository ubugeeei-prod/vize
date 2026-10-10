//! Vue's static class spelling from the retained, already decoded attribute.

use vize_l0::String;
use vize_l2::op::Attribute;

pub(super) enum Value<'a> {
    Borrowed(&'a str),
    Normalized(String),
}

impl Value<'_> {
    pub(super) fn as_str(&self) -> &str {
        match self {
            Self::Borrowed(value) => value,
            Self::Normalized(value) => value.as_str(),
        }
    }
}

pub(super) fn value<'a>(attribute: &Attribute<'a>) -> Value<'a> {
    normalize(attribute.name, attribute.value.unwrap_or_default())
}

pub(super) fn normalize<'a>(name: &str, value: &'a str) -> Value<'a> {
    if name != "class" {
        return Value::Borrowed(value);
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
        return Value::Borrowed(value);
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
    Value::Normalized(normalized)
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
    use super::{Value, normalize};

    #[test]
    fn canonical_classes_and_ordinary_values_keep_borrowed_storage() {
        assert!(matches!(
            normalize("class", "ready active"),
            Value::Borrowed(_)
        ));
        assert!(matches!(
            normalize("title", " a\t b "),
            Value::Borrowed(" a\t b ")
        ));
        assert!(matches!(
            normalize("class", "\u{feff}ready\u{feff}"),
            Value::Borrowed("ready")
        ));
    }

    #[test]
    fn html_condensation_and_ecmascript_trim_preserve_other_class_characters() {
        assert_eq!(
            normalize("class", " a\t b\nc\rd\u{c}e ").as_str(),
            "a b c d e"
        );
        assert_eq!(
            normalize("class", "\u{a0}a\u{85}b\u{feff}").as_str(),
            "a\u{85}b"
        );
        assert_eq!(
            normalize("class", "a\u{a0}b\u{feff}c\u{200b}d").as_str(),
            "a\u{a0}b\u{feff}c\u{200b}d"
        );
        assert_eq!(normalize("class", "a\u{b}b").as_str(), "a\u{b}b");
    }
}
