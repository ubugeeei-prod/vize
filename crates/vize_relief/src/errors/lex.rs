//! The L1 lexer's error codes, mapped onto the legacy compiler codes.

use vize_l1::markup::LexErrorCode;

use super::ErrorCode;

impl From<LexErrorCode> for ErrorCode {
    fn from(code: LexErrorCode) -> Self {
        match code {
            LexErrorCode::AbruptClosingOfEmptyComment => Self::AbruptClosingOfEmptyComment,
            LexErrorCode::EndTagWithAttributes => Self::EndTagWithAttributes,
            LexErrorCode::EndTagWithTrailingSolidus => Self::EndTagWithTrailingSolidus,
            LexErrorCode::EofBeforeTagName => Self::EofBeforeTagName,
            LexErrorCode::EofInCdata => Self::EofInCdata,
            LexErrorCode::EofInComment => Self::EofInComment,
            LexErrorCode::EofInTag => Self::EofInTag,
            LexErrorCode::IncorrectlyClosedComment => Self::IncorrectlyClosedComment,
            LexErrorCode::IncorrectlyOpenedComment => Self::IncorrectlyOpenedComment,
            LexErrorCode::InvalidFirstCharacterOfTagName => Self::InvalidFirstCharacterOfTagName,
            LexErrorCode::MissingAttributeValue => Self::MissingAttributeValue,
            LexErrorCode::MissingDynamicDirectiveArgumentEnd => {
                Self::MissingDynamicDirectiveArgumentEnd
            }
            LexErrorCode::MissingEndTagName => Self::MissingEndTagName,
            LexErrorCode::MissingInterpolationEnd => Self::MissingInterpolationEnd,
            LexErrorCode::MissingWhitespaceBetweenAttributes => {
                Self::MissingWhitespaceBetweenAttributes
            }
            LexErrorCode::NestedComment => Self::NestedComment,
            LexErrorCode::UnexpectedCharacterInAttributeName => {
                Self::UnexpectedCharacterInAttributeName
            }
            LexErrorCode::UnexpectedCharacterInUnquotedAttributeValue => {
                Self::UnexpectedCharacterInUnquotedAttributeValue
            }
            LexErrorCode::UnexpectedEqualsSignBeforeAttributeName => {
                Self::UnexpectedEqualsSignBeforeAttributeName
            }
            LexErrorCode::UnexpectedQuestionMarkInsteadOfTagName => {
                Self::UnexpectedQuestionMarkInsteadOfTagName
            }
            LexErrorCode::UnexpectedSolidusInTag => Self::UnexpectedSolidusInTag,
        }
    }
}

#[cfg(test)]
mod tests {
    use vize_l1::markup::LexErrorCode;

    use super::ErrorCode;

    #[test]
    fn lex_codes_keep_their_legacy_messages() {
        for code in LexErrorCode::ALL {
            assert_eq!(ErrorCode::from(code).message(), code.message(), "{code:?}");
        }
    }
}
