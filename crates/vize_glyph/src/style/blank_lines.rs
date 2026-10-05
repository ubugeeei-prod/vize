//! Restore authored grouping after the existing CSS parse and print.
//! Only matching complete token streams can place an in-rule separator.

use super::comment_scan::{consume_css_escape, find_comment_end};
use crate::options::FormatOptions;
use vize_l0::String;

pub(super) fn preserve(source: &str, formatted: String, options: &FormatOptions) -> String {
    if formatted.as_str().trim() == source || !source.lines().any(|line| line.trim().is_empty()) {
        return formatted;
    }
    let mut original = Tokens::new(source);
    let mut printed = Tokens::new(&formatted);
    let mut output = String::default();
    let mut cursor = 0;
    let mut previous_boundary = false;

    loop {
        match (original.next(), printed.next()) {
            (None, None) => break,
            (Some(authored), Some(target)) if authored.text == target.text => {
                if previous_boundary
                    && authored.depth > 0
                    && authored.parens == 0
                    && authored.brackets == 0
                    && authored.text != "}"
                    && has_blank_line(authored.gap)
                    && target.gap.contains(['\r', '\n'])
                {
                    output.push_str(formatted.get(cursor..target.gap_start).unwrap_or_default());
                    output.push_str(options.newline_string());
                    output.push_str(options.newline_string());
                    output.push_str(target.gap.rsplit(['\r', '\n']).next().unwrap_or_default());
                    cursor = target.start;
                }
                if !authored.text.starts_with("/*") {
                    previous_boundary = matches!(authored.text, ";" | "}");
                }
            }
            // Do not invent an anchor when a printer changes an authored token.
            _ => return formatted,
        }
    }
    if cursor == 0 {
        formatted
    } else {
        output.push_str(formatted.get(cursor..).unwrap_or_default());
        output
    }
}

fn has_blank_line(gap: &str) -> bool {
    let mut breaks = 0;
    let mut previous_cr = false;
    for byte in gap.bytes() {
        if byte == b'\r' || (byte == b'\n' && !previous_cr) {
            breaks += 1;
            if breaks == 2 {
                return true;
            }
        }
        previous_cr = byte == b'\r';
    }
    false
}

struct Token<'a> {
    text: &'a str,
    gap: &'a str,
    gap_start: usize,
    start: usize,
    depth: usize,
    parens: usize,
    brackets: usize,
}

struct Tokens<'a> {
    source: &'a str,
    index: usize,
    depth: usize,
    parens: usize,
    brackets: usize,
}

impl<'a> Tokens<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            index: 0,
            depth: 0,
            parens: 0,
            brackets: 0,
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.source.as_bytes();
        loop {
            let gap_start = self.index;
            while bytes.get(self.index).is_some_and(u8::is_ascii_whitespace) {
                self.index += 1;
            }
            let start = self.index;
            let &byte = bytes.get(start)?;
            // The existing authored-token comparison permits a final semicolon.
            if byte == b';' && self.source.get(start + 1..)?.trim_start().starts_with('}') {
                self.index += 1;
                continue;
            }
            let state = (self.depth, self.parens, self.brackets);
            if bytes.get(start..start + 2) == Some(b"/*".as_slice()) {
                self.index = find_comment_end(bytes, start + 2);
            } else if let Some(end) = consume_css_escape(bytes, start) {
                self.index = end;
                while !self.source.is_char_boundary(self.index) {
                    self.index += 1;
                }
            } else if matches!(byte, b'\'' | b'"') {
                self.index += 1;
                while let Some(&next) = bytes.get(self.index) {
                    if let Some(end) = consume_css_escape(bytes, self.index) {
                        self.index = end;
                    } else {
                        self.index += 1;
                        if next == byte {
                            break;
                        }
                    }
                }
            } else {
                self.index += self.source.get(start..)?.chars().next()?.len_utf8();
                match byte {
                    b'{' if self.parens == 0 && self.brackets == 0 => self.depth += 1,
                    b'}' if self.parens == 0 && self.brackets == 0 => {
                        self.depth = self.depth.saturating_sub(1);
                    }
                    b'(' => self.parens += 1,
                    b')' => self.parens = self.parens.saturating_sub(1),
                    b'[' => self.brackets += 1,
                    b']' => self.brackets = self.brackets.saturating_sub(1),
                    _ => {}
                }
            }
            return Some(Token {
                text: self.source.get(start..self.index)?,
                gap: self.source.get(gap_start..start)?,
                gap_start,
                start,
                depth: state.0,
                parens: state.1,
                brackets: state.2,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::preserve;
    use crate::FormatOptions;
    use vize_l0::cstr;

    #[test]
    fn comment_punctuation_cannot_take_rule_group_ownership() {
        for comment in ["/* ( */", "/* } ; \" { [ */"] {
            let source = cstr!(".a {{\n  color: red;\n  {comment}\n\n  margin: 0;\n}}\n");
            let printed = cstr!(".a {{\n  color: red;\n  {comment}\n  margin: 0;\n}}\n");
            let output = preserve(&source, printed, &FormatOptions::default());
            assert_eq!(output, source);
            assert_eq!(
                preserve(&output, output.clone(), &FormatOptions::default()),
                output
            );
        }
    }
}
