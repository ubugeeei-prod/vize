//! Reading back a rendered compiler error list.
//!
//! Gates that fail on template errors embed the list in their message through
//! [`CompilerErrorWithSource`](super::CompilerErrorWithSource)'s `Debug`
//! (`[CompilerError { code: …, message: "…", loc: Some(…) }, …]`). That text
//! is a pinned wire shape, so a reporter that wants the individual errors —
//! to group them, or to point at each one — reads them back here rather than
//! changing the message every consumer already depends on. Only what the
//! renderer wrote is accepted: an unexpected token makes the whole list
//! unreadable (`None`), so a caller can always fall back to the raw message.

use vize_l0::String;

/// One error read back from a rendered list. Offsets are into the source text
/// the list was rendered against (the template content for the SFC gate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedCompilerError {
    /// The `ErrorCode` variant name, as `Debug` printed it.
    pub code: String,
    /// The message, unescaped.
    pub message: String,
    /// Start and end byte offsets, when the error had a location.
    pub span: Option<(u32, u32)>,
}

/// Read a `Debug`-rendered `[CompilerError { … }, …]` list back.
#[must_use]
pub fn parse_rendered_error_list(text: &str) -> Option<std::vec::Vec<RenderedCompilerError>> {
    let mut reader = Reader { rest: text.trim() };
    reader.literal("[")?;
    let mut errors = std::vec::Vec::new();
    if reader.literal("]").is_some() {
        return reader.rest.is_empty().then_some(errors);
    }
    loop {
        errors.push(reader.error()?);
        if reader.literal(", ").is_some() {
            continue;
        }
        reader.literal("]")?;
        return reader.rest.is_empty().then_some(errors);
    }
}

struct Reader<'a> {
    rest: &'a str,
}

impl<'a> Reader<'a> {
    fn literal(&mut self, expected: &str) -> Option<()> {
        self.rest = self.rest.strip_prefix(expected)?;
        Some(())
    }

    fn error(&mut self) -> Option<RenderedCompilerError> {
        self.literal("CompilerError { code: ")?;
        let code = self.identifier()?;
        self.literal(", message: ")?;
        let message = self.string()?;
        self.literal(", loc: ")?;
        let span = if self.literal("None").is_some() {
            None
        } else {
            self.literal("Some(SourceLocation { start: ")?;
            let start = self.position()?;
            self.literal(", end: ")?;
            let end = self.position()?;
            self.literal(", source: ")?;
            self.string()?;
            self.literal(" })")?;
            Some((start, end))
        };
        self.literal(" }")?;
        Some(RenderedCompilerError {
            code: code.into(),
            message,
            span,
        })
    }

    /// `Position { offset: N, line: N, column: N }`, returning the offset:
    /// line and column are derived from the source text by the reader's
    /// caller, exactly as the renderer derived them.
    fn position(&mut self) -> Option<u32> {
        self.literal("Position { offset: ")?;
        let offset = self.number()?;
        self.literal(", line: ")?;
        self.number()?;
        self.literal(", column: ")?;
        self.number()?;
        self.literal(" }")?;
        Some(offset)
    }

    fn identifier(&mut self) -> Option<&'a str> {
        let end = self
            .rest
            .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .unwrap_or(self.rest.len());
        let (identifier, rest) = self.rest.split_at_checked(end)?;
        if identifier.is_empty() {
            return None;
        }
        self.rest = rest;
        Some(identifier)
    }

    fn number(&mut self) -> Option<u32> {
        let end = self
            .rest
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(self.rest.len());
        let (digits, rest) = self.rest.split_at_checked(end)?;
        let value = digits.parse().ok()?;
        self.rest = rest;
        Some(value)
    }

    /// A `Debug`-escaped string literal: `str::escape_debug`'s escapes.
    fn string(&mut self) -> Option<String> {
        let mut characters = self.rest.strip_prefix('"')?.char_indices();
        let mut value = String::default();
        while let Some((index, character)) = characters.next() {
            match character {
                '"' => {
                    self.rest = self.rest.get(index + 2..)?;
                    return Some(value);
                }
                '\\' => {
                    let (_, escape) = characters.next()?;
                    match escape {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '0' => value.push('\0'),
                        '\\' | '"' | '\'' => value.push(escape),
                        'u' => {
                            characters.next().filter(|(_, open)| *open == '{')?;
                            let mut code = 0u32;
                            loop {
                                let (_, digit) = characters.next()?;
                                if digit == '}' {
                                    break;
                                }
                                code = code.checked_mul(16)?.checked_add(digit.to_digit(16)?)?;
                            }
                            value.push(char::from_u32(code)?);
                        }
                        _ => return None,
                    }
                }
                _ => value.push(character),
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Write as _;

    use super::super::{CompilerError, CompilerErrorWithSource, ErrorCode};
    use super::{RenderedCompilerError, parse_rendered_error_list};
    use crate::relief::SourceLocation;
    use vize_l0::String;

    fn rendered(errors: &[CompilerError], source: &str) -> String {
        let mut text = String::default();
        write!(
            &mut text,
            "{:?}",
            CompilerErrorWithSource::list(errors, source)
        )
        .unwrap();
        text
    }

    #[test]
    fn a_rendered_list_reads_back_to_its_codes_messages_and_spans() {
        let source = "<div>\n  <p />😀{{ \"x\\y\" }}\n\t</div>";
        let errors = [
            CompilerError::with_message(
                ErrorCode::UnexpectedSolidusInTag,
                "Invalid self-closing syntax on non-void HTML element.",
                Some(SourceLocation::new(8, 13)),
            ),
            CompilerError::with_message(
                ErrorCode::InvalidExpression,
                "quote \" backslash \\ newline \n tab \t nul \0 bell \u{7} emoji 😀",
                Some(SourceLocation::new(17, 28)),
            ),
            CompilerError::with_message(ErrorCode::MissingEndTag, "no location", None),
        ];
        assert_eq!(
            parse_rendered_error_list(&rendered(&errors, source)),
            Some(
                errors
                    .iter()
                    .map(|error| RenderedCompilerError {
                        code: {
                            let mut code = String::default();
                            write!(&mut code, "{:?}", error.code).unwrap();
                            code
                        },
                        message: error.message.clone(),
                        span: error.loc.as_ref().map(|loc| (loc.span.start, loc.span.end)),
                    })
                    .collect()
            )
        );
    }

    #[test]
    fn an_empty_list_reads_back_empty() {
        assert_eq!(parse_rendered_error_list("[]"), Some(std::vec::Vec::new()));
    }

    #[test]
    fn anything_the_renderer_did_not_write_is_unreadable() {
        let source = "<p />";
        let errors = [CompilerError::with_message(
            ErrorCode::UnexpectedSolidusInTag,
            "m",
            Some(SourceLocation::new(0, 5)),
        )];
        let text = rendered(&errors, source);
        for broken in [
            "",
            "[",
            "[CompilerError { code: X }]",
            "Template compilation errors: []",
            &text.replace("loc: Some", "loc: Maybe"),
            &cstr_suffix(&text, " trailing"),
            &text.replace("\"m\"", "\"m"),
            &text.replace("\"m\"", "\"\\q\""),
        ] {
            assert_eq!(parse_rendered_error_list(broken), None, "{broken}");
        }
    }

    fn cstr_suffix(text: &str, suffix: &str) -> String {
        let mut joined = String::from(text);
        joined.push_str(suffix);
        joined
    }
}
