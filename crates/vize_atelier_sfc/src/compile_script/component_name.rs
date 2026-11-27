//! The `__name` option, spelled as Vue spells it: a single-quoted string.

use vize_carton::string_builder::TextBuf;

/// Append `__name: '<name>',` for the component named after its file.
///
/// A file stem is arbitrary text, so anything that would end or bend the
/// literal is escaped; an ordinary stem is copied byte for byte.
pub(crate) fn push_component_name_option(output: &mut TextBuf, name: &str) {
    output.push_str("  __name: '");
    let mut copied = 0;
    for (index, character) in name.char_indices() {
        let escaped: Option<&str> = match character {
            '\'' => Some("\\'"),
            '\\' => Some("\\\\"),
            '\n' => Some("\\n"),
            '\r' => Some("\\r"),
            '\u{2028}' => Some("\\u2028"),
            '\u{2029}' => Some("\\u2029"),
            _ if character.is_control() => None,
            _ => continue,
        };
        output.push_str(name.get(copied..index).unwrap_or_default());
        match escaped {
            Some(escaped) => output.push_str(escaped),
            None => push_unicode_escape(output, character),
        }
        copied = index + character.len_utf8();
    }
    output.push_str(name.get(copied..).unwrap_or_default());
    output.push_str("',\n");
}

fn push_unicode_escape(output: &mut TextBuf, character: char) {
    output.push_str("\\u");
    let value = u32::from(character);
    for shift in [12, 8, 4, 0] {
        let digit = char::from_digit((value >> shift) & 0xf, 16).unwrap_or('0');
        output.push(digit);
    }
}

#[cfg(test)]
mod tests {
    use super::{TextBuf, push_component_name_option};

    fn option(name: &str) -> vize_carton::String {
        let mut output = TextBuf::with_capacity(0);
        push_component_name_option(&mut output, name);
        output.into_compact()
    }

    #[test]
    fn ordinary_names_are_copied() {
        assert_eq!(option("MyButton"), "  __name: 'MyButton',\n");
        assert_eq!(option("[id].page"), "  __name: '[id].page',\n");
        assert_eq!(option("ボタン"), "  __name: 'ボタン',\n");
    }

    #[test]
    fn a_name_cannot_end_the_literal() {
        assert_eq!(option("it's+b"), "  __name: 'it\\'s+b',\n");
        assert_eq!(option("a\\"), "  __name: 'a\\\\',\n");
        assert_eq!(
            option("a\nb\u{2028}c\u{1b}d"),
            "  __name: 'a\\nb\\u2028c\\u001bd',\n"
        );
    }
}
