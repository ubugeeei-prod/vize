//! Re-quote CSS strings the way Prettier does.
//!
//! A string takes the preferred quote unless its content holds more of that
//! quote than of the other one. Escapes stay as written, except an escaped
//! old delimiter, which no longer needs its backslash. Comments and
//! unterminated strings are copied unchanged. With `line_comments` (Sass and
//! Less), every `//` starts a comment; a `//` inside an unquoted `url()` then
//! only leaves the rest of that line as written.

use vize_l0::String;

pub(super) fn normalize(css: &str, prefer_single: bool, line_comments: bool) -> String {
    let (preferred, alternate) = if prefer_single {
        ('\'', '"')
    } else {
        ('"', '\'')
    };
    let bytes = css.as_bytes();
    let mut output = String::with_capacity(css.len());
    let mut copied = 0;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index = css
                .get(index + 2..)
                .and_then(|rest| rest.find("*/"))
                .map_or(css.len(), |end| index + 2 + end + 2);
            continue;
        }
        if line_comments && byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            index = css
                .get(index..)
                .and_then(|rest| rest.find('\n'))
                .map_or(css.len(), |end| index + end);
            continue;
        }
        if byte != b'"' && byte != b'\'' {
            index += 1;
            continue;
        }
        let Some(end) = string_end(bytes, index) else {
            break;
        };
        let old = char::from(byte);
        let content = css.get(index + 1..end).unwrap_or_default();
        let enclosing = if content.matches(preferred).count() > content.matches(alternate).count() {
            alternate
        } else {
            preferred
        };
        if enclosing != old {
            output.push_str(css.get(copied..index).unwrap_or_default());
            write_requoted(&mut output, content, old, enclosing);
            copied = end + 1;
        }
        index = end + 1;
    }
    output.push_str(css.get(copied..).unwrap_or_default());
    output
}

/// Index of the closing quote, or `None` when the string is unterminated.
fn string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let quote = *bytes.get(start)?;
    let mut index = start + 1;
    loop {
        match *bytes.get(index)? {
            b'\\' => index += 2,
            b'\n' | b'\r' => return None,
            byte if byte == quote => return Some(index),
            _ => index += 1,
        }
    }
}

fn write_requoted(output: &mut String, content: &str, old: char, enclosing: char) {
    output.push(enclosing);
    let mut chars = content.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some(escaped) if escaped == old => output.push(escaped),
                Some(escaped) => {
                    output.push('\\');
                    output.push(escaped);
                }
                None => output.push('\\'),
            }
        } else {
            if ch == enclosing {
                output.push('\\');
            }
            output.push(ch);
        }
    }
    output.push(enclosing);
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn prefers_the_configured_quote() {
        let css = "@import 'a.css';\na[x='y'] {\n  background: url('b.png');\n  content: '\\a';\n}";
        assert_eq!(
            normalize(css, false, false).as_str(),
            "@import \"a.css\";\na[x=\"y\"] {\n  background: url(\"b.png\");\n  content: \"\\a\";\n}"
        );
        assert_eq!(
            normalize("a { content: \"b\"; }", true, false).as_str(),
            "a { content: 'b'; }"
        );
    }

    #[test]
    fn keeps_strings_that_would_need_more_escapes() {
        let css = "a { font-family: 'A \"B\"', 'C'; }";
        assert_eq!(
            normalize(css, false, false).as_str(),
            "a { font-family: 'A \"B\"', \"C\"; }"
        );
    }

    #[test]
    fn drops_the_escape_of_the_old_delimiter() {
        assert_eq!(
            normalize("a { content: 'it\\'s'; }", false, false).as_str(),
            "a { content: \"it's\"; }"
        );
    }

    #[test]
    fn leaves_comments_and_unterminated_strings() {
        let css = "/* 'c' */\na { content: 'b }";
        assert_eq!(normalize(css, false, false).as_str(), css);
    }

    #[test]
    fn skips_line_comments_when_asked() {
        let scss = "// it's\n@use 'a';\n.a { background: url(http://x/'y'); }";
        assert_eq!(
            normalize(scss, false, true).as_str(),
            "// it's\n@use \"a\";\n.a { background: url(http://x/'y'); }"
        );
    }
}
