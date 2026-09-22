//! Byte-level identifier checks shared by the operand schemas. Admission runs
//! them on every operand, so they avoid `char` searchers and hashed sets; the
//! tests pin them to `str::trim`, oxc's reserved keywords and the carton
//! void-tag table.

/// `str::trim` without the Unicode scan when both ends are plainly not
/// whitespace (every admitted identifier byte is ASCII).
pub(super) fn trimmed(text: &str) -> &str {
    let plain = |byte: Option<&u8>| byte.is_some_and(|b| b.is_ascii() && !is_space(*b));
    let bytes = text.as_bytes();
    if plain(bytes.first()) && plain(bytes.last()) {
        text
    } else {
        text.trim()
    }
}

/// The ASCII bytes `char::is_whitespace` accepts (vertical tab included).
const fn is_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

pub(super) fn identifier_segment(part: &[u8]) -> bool {
    part.first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_' || *b == b'$')
        && part
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'$')
}

/// The root of a dotted identifier path (`a`, `a.b`), once trimmed.
pub(super) fn path_root(text: &str) -> Option<&str> {
    let text = trimmed(text);
    let bytes = text.as_bytes();
    if !bytes.split(|b| *b == b'.').all(identifier_segment) {
        return None;
    }
    let end = bytes.iter().position(|b| *b == b'.').unwrap_or(bytes.len());
    Some(&text[..end])
}

/// oxc's `RESERVED_KEYWORDS`, spelled out so admission avoids its SipHash
/// lookup; `reserved_keywords_match_oxc` keeps the two sets equal.
pub(super) fn reserved_keyword(word: &str) -> bool {
    matches!(
        word,
        "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

/// A name hashed and compared ASCII-case-insensitively, the way the legacy
/// parser reports repeated attributes; a set of these keeps the repeat check
/// linear on wide elements.
#[derive(Clone, Copy)]
pub(super) struct Folded<'a>(pub(super) &'a str);

impl PartialEq for Folded<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for Folded<'_> {}

impl core::hash::Hash for Folded<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for byte in self.0.bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
        state.write_usize(self.0.len());
    }
}

/// Void elements among the tags admission accepts (`operands::element`).
pub(in crate::s3) fn admitted_void(tag: &str) -> bool {
    matches!(tag, "input" | "img" | "br" | "hr")
}

pub(in crate::s3) fn reference(value: &str) -> bool {
    // A deliberately narrower grammar than JavaScript. The S3 producer has
    // already classified it as JS; no reparsing or opaque reinterpretation.
    path_root(value).is_some_and(|root| root != "$event" && !reserved_keyword(root))
}

#[cfg(test)]
mod tests {
    use super::{admitted_void, path_root, reserved_keyword, trimmed};

    #[test]
    fn reserved_keywords_match_oxc() {
        let oxc = &oxc_syntax::keyword::RESERVED_KEYWORDS;
        assert_eq!(oxc.len(), 46);
        for word in oxc.iter() {
            assert!(reserved_keyword(word), "{word}");
        }
        for word in ["undefined", "NaN", "row", "Let", "lets", "", "of", "async"] {
            assert_eq!(reserved_keyword(word), oxc.contains(word), "{word}");
        }
    }

    #[test]
    fn fast_paths_agree_with_the_standard_library() {
        for text in [
            "a",
            " a",
            "a ",
            "\u{b}a",
            "a\u{a0}",
            "\u{2028}a.b",
            "",
            " ",
            "a.b",
            "é",
        ] {
            assert_eq!(trimmed(text), text.trim(), "{text:?}");
        }
        assert_eq!(path_root(" row.id "), Some("row"));
        assert_eq!(path_root("$event.target"), Some("$event"));
        for text in ["a.", ".a", "a..b", "a b", "a-b", "1a", ""] {
            assert_eq!(path_root(text), None, "{text:?}");
        }
        for tag in [
            "div", "span", "main", "section", "article", "header", "footer", "nav", "aside",
            "button", "strong", "em", "b", "i", "small", "label", "input", "img", "br", "hr", "ul",
            "ol", "li", "template", "p", "a", "form", "h1", "h2", "h3", "h4", "h5", "h6",
        ] {
            assert_eq!(admitted_void(tag), vize_carton::is_void_tag(tag), "{tag}");
        }
    }
}
