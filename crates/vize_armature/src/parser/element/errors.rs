//! Error reporting and recoverable-error message construction.

use vize_relief::errors::{CompilerError, ErrorCode};
use vize_s0::{String, appends};

use super::super::Parser;

impl<'a> Parser<'a> {
    /// Handle error
    pub(in crate::parser) fn on_error_impl(&mut self, code: ErrorCode, index: usize) {
        if self.template_syntax.is_quirks() && code == ErrorCode::MissingWhitespaceBetweenAttributes
        {
            return;
        }

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
        // `RECOVERED_PARSE_CODES` (vize_relief) is the single source of truth
        // for which defects the parser recovers from, and patina keys its
        // analysis gating off the same classification (#3294). Consulting it
        // here means the message table below cannot claim a recovery the
        // classification does not know about.
        if !code.has_documented_parse_recovery() {
            return None;
        }
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
                    .map(|attr| attr.name)
                    .or_else(|| self.current_dir.as_ref().map(|dir| dir.raw_name))
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
            // Unreachable in practice: the guard above admits only
            // `RECOVERED_PARSE_CODES`, and every entry there has an arm above
            // (pinned by `recovery_messages_and_recovered_parse_classification_agree`).
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use vize_relief::errors::{CompilerError, ErrorCode, recovery::RECOVERED_PARSE_CODES};
    use vize_s0::Allocator;

    use crate::parser::Parser;

    /// `CompilerError::is_recovered_parse` (vize_relief) and the recovery
    /// messages constructed above must describe the same set of defects
    /// (#3294). Both sides now read the shared `RECOVERED_PARSE_CODES`
    /// classification rather than keeping their own copies, so this pins every
    /// classified code to a real message: adding a recovery message means
    /// adding the code to that list, and adding it to the list without a
    /// message fails here. `DuplicateAttribute` is recovered without a custom
    /// message, so it is the one allowed asymmetry.
    #[test]
    fn recovery_messages_and_recovered_parse_classification_agree() {
        let allocator = Allocator::new();
        let parser = Parser::new(&allocator, "");
        for &code in RECOVERED_PARSE_CODES {
            assert!(
                parser.recovery_error_message(code).is_some(),
                "expected a recovery message for {code:?}"
            );
            assert!(
                CompilerError::new(code, None).is_recovered_parse(),
                "expected is_recovered_parse for {code:?}"
            );
        }
        for code in [
            ErrorCode::MissingEndTagName,
            ErrorCode::UnexpectedNullCharacter,
            ErrorCode::CdataInHtmlContent,
        ] {
            assert!(parser.recovery_error_message(code).is_none(), "{code:?}");
            assert!(
                !CompilerError::new(code, None).is_recovered_parse(),
                "{code:?}"
            );
        }
        assert!(CompilerError::new(ErrorCode::DuplicateAttribute, None).is_recovered_parse());
        assert!(
            parser
                .recovery_error_message(ErrorCode::DuplicateAttribute)
                .is_none()
        );
    }
}
