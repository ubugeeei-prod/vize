//! Lexical boundaries for bracketed directive arguments, without reparsing JS.

use vize_relief::ErrorCode;
use vize_s0::SmallVec;

use super::{Callbacks, State, Tokenizer, types::is_end_of_tag_section};

impl<C: Callbacks> Tokenizer<'_, C> {
    pub(super) fn state_in_dir_dynamic_arg(&mut self, _c: u8) {
        let (end, closed) = scan_argument(self.input, self.index);
        self.index = end;
        if closed {
            self.callbacks.on_dir_arg(self.section_start, end);
            self.state = State::InDirArg;
            self.section_start = end + 1;
        } else if end == self.input.len() {
            // The outer loop advances once more, then the existing EOF recovery
            // emits both the incomplete argument and the unterminated-tag error.
            self.index = end.saturating_sub(1);
        } else {
            self.callbacks
                .on_error(ErrorCode::MissingDynamicDirectiveArgumentEnd, end);
            if self.section_start < end {
                self.callbacks.on_dir_arg(self.section_start, end);
            }
            self.callbacks.on_attrib_name_end(end);
            self.section_start = end;
            self.state = State::AfterAttrName;
            if let Some(&c) = self.input.get(end) {
                self.state_after_attr_name(c);
            }
        }
    }
}

fn is_boundary(byte: u8) -> bool {
    byte == b'=' || is_end_of_tag_section(byte)
}

fn scan_argument(input: &[u8], start: usize) -> (usize, bool) {
    let mut delimiters = SmallVec::<[u8; 8]>::new();
    delimiters.push(b']');
    let mut index = start;
    while let Some(&byte) = input.get(index) {
        // Directive heads remain HTML attribute names. Preserve their existing
        // boundaries even inside unfinished JS literals, without lookahead or
        // backtracking that could swallow subsequent attributes and tags.
        if is_boundary(byte) {
            return (index, false);
        }
        // The stack only empties when the argument's own `]` closes it,
        // which returns below.
        let Some(&delimiter) = delimiters.last() else {
            return (index, true);
        };
        if matches!(delimiter, b'\'' | b'"' | b'`') {
            match byte {
                b'\\' => {
                    if input.get(index + 1).is_some_and(|&byte| is_boundary(byte)) {
                        return (index + 1, false);
                    }
                    index += 1;
                }
                _ if byte == delimiter => {
                    delimiters.pop();
                }
                b'$' if delimiter == b'`' && input.get(index + 1) == Some(&b'{') => {
                    delimiters.push(b'}');
                    index += 1;
                }
                _ => {}
            }
        } else {
            match byte {
                b'\'' | b'"' | b'`' => delimiters.push(byte),
                b'[' => delimiters.push(b']'),
                b'(' => delimiters.push(b')'),
                b'{' => delimiters.push(b'}'),
                b']' | b')' | b'}' if delimiters.last() == Some(&byte) => {
                    delimiters.pop();
                    if delimiters.is_empty() {
                        return (index, true);
                    }
                }
                _ => {}
            }
        }
        index += 1;
    }
    (input.len(), false)
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
