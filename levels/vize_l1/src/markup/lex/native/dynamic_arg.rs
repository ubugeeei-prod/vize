//! Lexical boundaries for bracketed directive arguments, without reparsing JS.

use vize_l0::SmallVec;

use super::{Lexer, State, types::is_end_of_tag_section};
use crate::markup::profile::Profile;
use crate::markup::token::{LexErrorCode, Sink};

impl<P: Profile, S: Sink> Lexer<'_, P, S> {
    pub(super) fn state_in_dir_dynamic_arg(&mut self, _c: u8) {
        let (end, closed) = scan_argument(self.input, self.index);
        self.index = end;
        if closed {
            self.sink.on_dir_arg(self.section_start, end);
            self.state = State::InDirArg;
            self.section_start = end + 1;
        } else if end == self.input.len() {
            // The outer loop advances once more, then the existing EOF recovery
            // emits both the incomplete argument and the unterminated-tag error.
            self.index = end.saturating_sub(1);
        } else {
            self.sink
                .on_error(LexErrorCode::MissingDynamicDirectiveArgumentEnd, end);
            if self.section_start < end {
                self.sink.on_dir_arg(self.section_start, end);
            }
            self.sink.on_attrib_name_end(end);
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

pub(super) fn scan_argument(input: &[u8], start: usize) -> (usize, bool) {
    let mut delimiters = SmallVec::<[u8; 8]>::new();
    delimiters.push(b']');
    let mut index = start;
    while let Some(&byte) = input.get(index) {
        // The stack only empties when the argument's own `]` closes it,
        // which returns below.
        let Some(&delimiter) = delimiters.last() else {
            return (index, true);
        };
        // An adjacent arrow outside a literal belongs to a dynamic argument,
        // not an attribute assignment followed by a tag close. Only this exact
        // token crosses those two boundaries; no search for a later `]` and no
        // parser retry can swallow a following attribute or unfinished tag.
        if byte == b'='
            && input.get(index + 1) == Some(&b'>')
            && !matches!(delimiter, b'\'' | b'"' | b'`')
        {
            index += 2;
            continue;
        }
        // Preserve every other HTML head boundary, including unfinished quoted
        // literals. The retained JS producer independently validates the token.
        if is_boundary(byte) {
            return (index, false);
        }
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
mod tests {
    use super::super::{
        LexOptions, dynamic_argument_boundary,
        tests::{TokenEvent, lex_with},
    };
    use super::scan_argument;
    use crate::markup::token::LexErrorCode;
    use vize_l0::{SmallVec, cstr};

    #[test]
    fn emits_one_complete_argument_and_preserves_modifiers() {
        for argument in [
            "keys[index]",
            "keys[indices[index]]",
            "keys[']']",
            "keys['[']",
            r#"keys["\"]"]"#,
            "keys['a.b']",
            "`prefix${keys['[']}`",
            "`prefix${`nested${keys[']']}`}`",
            "`escaped\\`][${keys[index]}`",
            "(slot=>slot)(outside)",
            "(other=>slot)(outside)",
        ] {
            let source = cstr!("<Child v-model:[{argument}].trim=\"value\"/>");
            let tokenizer = lex_with(&source, LexOptions::default());
            assert!(tokenizer.errors.is_empty(), "{source}");
            let args: SmallVec<[&str; 2]> = tokenizer
                .events
                .iter()
                .filter_map(|event| match event {
                    TokenEvent::DirArg(start, end) => Some(&source[*start..*end]),
                    _ => None,
                })
                .collect();
            assert_eq!(args.as_slice(), [argument], "{source}");
            assert!(tokenizer.events.iter().any(|event| matches!(
                event, TokenEvent::DirModifier(start, end) if &source[*start..*end] == "trim"
            )));
        }
    }

    #[test]
    fn boundaries_and_unterminated_literals_recover_without_swallowing_markup() {
        // A suffix that never closes the tag also reports `EofInTag` at the end;
        // with no suffix at all the end of input is reached inside the argument.
        for (suffix, eof) in [
            ("=\"value\"", Some(Eof::AfterArgument)),
            (" other", Some(Eof::AfterArgument)),
            ("/>", None),
            (">", None),
            ("\nother", Some(Eof::AfterArgument)),
            ("", Some(Eof::InsideArgument)),
        ] {
            for argument in [
                "keys[index]",
                "keys['broken",
                "keys[\"broken",
                "`broken",
                "keys['escaped\\",
                "(slot=>slot)(outside)",
            ] {
                let source = cstr!("<Child :[{argument}{suffix}");
                let tokenizer = lex_with(&source, LexOptions::default());
                let boundary = "<Child :[".len() + argument.len();
                let unterminated = (LexErrorCode::MissingDynamicDirectiveArgumentEnd, boundary);
                let end = (LexErrorCode::EofInTag, source.len());
                let expected: &[(LexErrorCode, usize)] = match eof {
                    None => &[unterminated],
                    Some(Eof::AfterArgument) => &[unterminated, end],
                    Some(Eof::InsideArgument) => &[end, unterminated],
                };
                assert_eq!(tokenizer.errors.as_slice(), expected, "{source}");
                let args: SmallVec<[&str; 2]> = tokenizer
                    .events
                    .iter()
                    .filter_map(|event| match event {
                        TokenEvent::DirArg(start, end) => Some(&source[*start..*end]),
                        _ => None,
                    })
                    .collect();
                assert_eq!(args.as_slice(), [argument], "{source}");
            }
        }
    }

    enum Eof {
        AfterArgument,
        InsideArgument,
    }

    #[test]
    fn scanning_is_iterative_and_returns_the_outer_boundary() {
        let argument = cstr!("{}key{}", "[".repeat(20_000), "]".repeat(20_000));
        let source = cstr!("{argument}].trim=\"value\"");
        assert_eq!(scan_argument(source.as_bytes(), 0), (argument.len(), true));
        let source = cstr!("{argument}=\"value\"");
        assert_eq!(scan_argument(source.as_bytes(), 0), (argument.len(), false));
    }

    #[test]
    fn public_boundary_preserves_the_same_borrowed_input_and_recovery() {
        for (text, end, closed) in [
            ("keys['names]'][indices[index]]].camel", 30, true),
            ("keys[`name]${indices[index]}`]].stop", 30, true),
            ("keys['日本]']].camel", 15, true),
            ("]", 0, true),
            ("", 0, false),
            ("keys['broken", 12, false),
            ("keys['=>']].camel", 6, false),
            ("keys[index]=value", 11, false),
        ] {
            assert_eq!(dynamic_argument_boundary(text), (end, closed), "{text}");
            assert_eq!(
                dynamic_argument_boundary(text),
                scan_argument(text.as_bytes(), 0),
                "public wrapper must retain the existing scanner contract"
            );
        }
    }
}
