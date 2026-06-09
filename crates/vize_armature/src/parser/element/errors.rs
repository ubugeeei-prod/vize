//! Error reporting and recoverable-error message construction.

use vize_carton::{String, appends};
use vize_relief::errors::{CompilerError, ErrorCode};

use super::super::Parser;

impl<'a> Parser<'a> {
    /// Handle error
    pub(in crate::parser) fn on_error_impl(&mut self, code: ErrorCode, index: usize) {
        let len = self.source.len();
        let start = index.min(len);
        let end = (index + 1).min(len);
        let loc = self.create_loc(start, end);
        let error = if let Some(message) = self.recovery_error_message(code) {
            CompilerError::with_message(code, message, Some(loc))
        } else {
            CompilerError::new(code, Some(loc))
        };
        self.errors.push(error);
    }

    fn recovery_error_message(&self, code: ErrorCode) -> Option<String> {
        match code {
            ErrorCode::EofBeforeTagName => Some(
                "Unexpected end of input after `<`; treating it as text so parsing can continue."
                    .into(),
            ),
            ErrorCode::EofInTag => Some(
                "Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue."
                    .into(),
            ),
            ErrorCode::EofInComment => Some(
                "Comment is missing its closing `-->`; preserving the unfinished comment so parsing can finish."
                    .into(),
            ),
            ErrorCode::InvalidFirstCharacterOfTagName => Some(
                "Tag name starts with an invalid character; treating the malformed tag as text.".into(),
            ),
            ErrorCode::MissingAttributeValue => {
                let name = self
                    .current_attr
                    .as_ref()
                    .map(|attr| attr.name.as_str())
                    .or_else(|| self.current_dir.as_ref().map(|dir| dir.raw_name.as_str()))
                    .unwrap_or("attribute");
                let mut message = String::with_capacity(name.len() + 70);
                appends!(
                    message,
                    "Attribute `",
                    name,
                    "` is missing a value after `=`; continuing without the value."
                );
                Some(message)
            }
            ErrorCode::MissingDynamicDirectiveArgumentEnd => Some(
                "Dynamic directive argument is missing its closing `]`; inferred the argument end at the next tag boundary."
                    .into(),
            ),
            ErrorCode::MissingInterpolationEnd => {
                let delimiter = self.options.delimiters.1.as_str();
                let mut message = String::with_capacity(delimiter.len() + 97);
                appends!(
                    message,
                    "Interpolation is missing its closing delimiter `",
                    delimiter,
                    "`; treating the unfinished interpolation as text."
                );
                Some(message)
            }
            ErrorCode::UnexpectedCharacterInAttributeName => Some(
                "Attribute name contains an invalid character; inferred the nearest attribute boundary and continued."
                    .into(),
            ),
            ErrorCode::UnexpectedCharacterInUnquotedAttributeValue => Some(
                "Unquoted attribute value contains a character that should be quoted; keeping it in the value and continuing."
                    .into(),
            ),
            ErrorCode::UnexpectedEqualsSignBeforeAttributeName => Some(
                "Unexpected `=` before an attribute name; skipping it and continuing with the next attribute."
                    .into(),
            ),
            ErrorCode::MissingWhitespaceBetweenAttributes => Some(
                "Missing whitespace between attributes; inferred a new attribute boundary.".into(),
            ),
            ErrorCode::IncorrectlyClosedComment => Some(
                "Comment was closed as `--!>`; treating it as `-->` so parsing can continue."
                    .into(),
            ),
            ErrorCode::IncorrectlyOpenedComment => Some(
                "Declaration or comment syntax is malformed; skipping it until the next `>`.".into(),
            ),
            _ => None,
        }
    }
}
