//! Restore authored grouping after the existing CSS parse and print.
//! Only matching complete token streams can place an in-rule separator.

use super::comment_scan::{consume_css_escape, find_comment_end};
use crate::options::FormatOptions;
use vize_l0::String;

#[cfg(test)]
fn preserve(source: &str, formatted: String, options: &FormatOptions) -> String {
    preserve_rule_layout(source, formatted, options, None, source)
}

pub(super) fn preserve_rule_layout(
    source: &str,
    formatted: String,
    options: &FormatOptions,
    layout: Option<&super::rule_layout::RuleLayout>,
    protected: &str,
) -> String {
    if layout.is_none()
        && (formatted.as_str().trim() == source
            || !source.lines().any(|line| line.trim().is_empty()))
    {
        return formatted;
    }
    let mut original = Tokens::new(source);
    let mut printed = Tokens::new(&formatted);
    let mut adjustments = Vec::new();
    let mut pending = Vec::new();
    let mut values = super::values::DeclarationValues::default();
    let mut previous_boundary = false;
    let mut previous_close = false;
    let mut previous_close_line = false;
    let mut previous_declaration = false;
    let mut previous_comma = false;
    let mut brace = 0;
    let mut preludes = layout
        .into_iter()
        .flat_map(|layout| &layout.preludes)
        .peekable();

    loop {
        match (original.next(), printed.next()) {
            (None, None) => break,
            (Some(authored), Some(target)) if authored.text == target.text => {
                let rule_brace = authored.text == "{"
                    && preludes.peek().is_some_and(|prelude| {
                        prelude.brace == brace
                            && authored
                                .start
                                .checked_sub(prelude.range.len())
                                .and_then(|start| source.get(start..authored.start))
                                .zip(protected.get(prelude.range.clone()))
                                .is_some_and(|(original, parsed)| original == parsed)
                    });
                values.observe(source, &authored, &target, rule_brace, &mut adjustments);
                if previous_boundary
                    && authored.depth > 0
                    && authored.parens == 0
                    && authored.brackets == 0
                    && authored.text != "}"
                    && has_blank_line(authored.gap)
                    && target.gap.contains(['\r', '\n'])
                {
                    adjustments.push(Adjustment::new(&authored, &target, 2, false));
                }
                if previous_comma
                    || previous_close
                    || (previous_declaration && authored.gap.contains(['\r', '\n']))
                {
                    let newlines = if previous_close && has_blank_line(authored.gap) {
                        2
                    } else {
                        1
                    };
                    let mut adjustment =
                        Adjustment::new(&authored, &target, newlines, previous_comma);
                    adjustment.authored_rule_gap = authored.gap.contains(['\r', '\n'])
                        && (previous_close_line || previous_declaration);
                    pending.push(adjustment);
                }
                if authored.parens == 0 && authored.brackets == 0 {
                    if authored.text == "{" {
                        while preludes
                            .peek()
                            .is_some_and(|prelude| prelude.brace <= brace)
                        {
                            let Some(prelude) = preludes.next() else {
                                break;
                            };
                            let Some(start) = authored.start.checked_sub(prelude.range.len())
                            else {
                                continue;
                            };
                            // Color markers can move offsets but cannot establish ownership:
                            // the complete original prelude must equal the parsed prelude.
                            if prelude.brace == brace
                                && source
                                    .get(start..authored.start)
                                    .zip(protected.get(prelude.range.clone()))
                                    .is_some_and(|(original, parsed)| original == parsed)
                            {
                                adjustments.extend(
                                    pending
                                        .iter()
                                        .filter(|adjustment: &&Adjustment| {
                                            if adjustment.selector {
                                                prelude.selector_list
                                                    && adjustment.source_start > start
                                            } else {
                                                adjustment.source_start == start
                                                    && (adjustment.authored_rule_gap
                                                        || prelude.selector_list)
                                            }
                                        })
                                        .cloned(),
                                );
                            }
                        }
                        brace += 1;
                    }
                    if matches!(authored.text, ";" | "{" | "}") {
                        pending.clear();
                    }
                }
                if !authored.text.starts_with("/*") {
                    previous_boundary = matches!(authored.text, ";" | "}");
                    previous_close =
                        authored.text == "}" && authored.parens == 0 && authored.brackets == 0;
                    // Compact blocks retain gaps unless an actual selector list owns them.
                    previous_close_line = previous_close
                        && source
                            .get(..authored.start)
                            .and_then(|prefix| prefix.rsplit(['\r', '\n']).next())
                            .is_some_and(|line| line.trim().is_empty());
                    previous_declaration =
                        authored.text == ";" && authored.parens == 0 && authored.brackets == 0;
                    previous_comma =
                        authored.text == "," && authored.parens == 0 && authored.brackets == 0;
                }
            }
            // No edits are accepted until the entire authored stream matches.
            _ => return formatted,
        }
    }
    adjustments.sort_by_key(|adjustment| adjustment.gap_start);
    adjustments.dedup_by_key(|adjustment| adjustment.gap_start);
    let mut output = String::default();
    let mut cursor = 0;
    for adjustment in adjustments {
        let indentation = if adjustment.selector || adjustment.value {
            options
                .indent_string()
                .repeat(adjustment.depth + usize::from(adjustment.value))
        } else {
            formatted
                .get(adjustment.gap_start..adjustment.start)
                .unwrap_or_default()
                .rsplit(['\r', '\n'])
                .next()
                .unwrap_or_default()
                .into()
        };
        let mut replacement = String::default();
        for _ in 0..adjustment.newlines {
            replacement.push_str(options.newline_string());
        }
        replacement.push_str(&indentation);
        if formatted.get(adjustment.gap_start..adjustment.start) != Some(replacement.as_str()) {
            output.push_str(
                formatted
                    .get(cursor..adjustment.gap_start)
                    .unwrap_or_default(),
            );
            output.push_str(&replacement);
            cursor = adjustment.start;
        }
    }
    if cursor == 0 {
        formatted
    } else {
        output.push_str(formatted.get(cursor..).unwrap_or_default());
        output
    }
}

#[derive(Clone)]
pub(super) struct Adjustment {
    source_start: usize,
    gap_start: usize,
    start: usize,
    depth: usize,
    newlines: usize,
    selector: bool,
    authored_rule_gap: bool,
    pub(super) value: bool,
}

impl Adjustment {
    pub(super) fn new(
        source: &Token<'_>,
        target: &Token<'_>,
        newlines: usize,
        selector: bool,
    ) -> Self {
        Self {
            source_start: source.start,
            gap_start: target.gap_start,
            start: target.start,
            depth: source.depth,
            newlines,
            selector,
            authored_rule_gap: false,
            value: false,
        }
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

pub(super) struct Token<'a> {
    pub(super) text: &'a str,
    pub(super) gap: &'a str,
    pub(super) gap_start: usize,
    pub(super) start: usize,
    pub(super) depth: usize,
    pub(super) parens: usize,
    pub(super) brackets: usize,
}

pub(super) struct Tokens<'a> {
    source: &'a str,
    index: usize,
    depth: usize,
    parens: usize,
    brackets: usize,
}

impl<'a> Tokens<'a> {
    pub(super) fn new(source: &'a str) -> Self {
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
