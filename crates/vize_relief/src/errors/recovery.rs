//! Classification of parse errors whose recovery yields a complete tree.

use super::{CompilerError, ErrorCode};

impl CompilerError {
    /// Returns true when the template parser documents a concrete recovery
    /// strategy for this code: parsing continues past the defect and still
    /// yields a complete tree, so semantic analysis over that tree stays
    /// meaningful (#3294).
    ///
    /// This is deliberately broader than [`CompilerError::is_recoverable`],
    /// which additionally implies downstream codegen may proceed without
    /// gating. The code list mirrors the recovery messages constructed in
    /// `vize_armature`'s `recovery_error_message`; the armature parser tests
    /// pin the two lists together.
    #[must_use]
    pub fn is_recovered_parse(&self) -> bool {
        matches!(
            self.code,
            ErrorCode::EofBeforeTagName
                | ErrorCode::EofInTag
                | ErrorCode::EofInComment
                | ErrorCode::InvalidFirstCharacterOfTagName
                | ErrorCode::MissingAttributeValue
                | ErrorCode::MissingDynamicDirectiveArgumentEnd
                | ErrorCode::MissingInterpolationEnd
                | ErrorCode::UnexpectedCharacterInAttributeName
                | ErrorCode::UnexpectedCharacterInUnquotedAttributeValue
                | ErrorCode::UnexpectedEqualsSignBeforeAttributeName
                | ErrorCode::MissingWhitespaceBetweenAttributes
                | ErrorCode::IncorrectlyClosedComment
                | ErrorCode::IncorrectlyOpenedComment
        ) || self.is_recoverable()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompilerError, ErrorCode};

    #[test]
    fn recovered_parse_covers_the_documented_recovery_codes() {
        for code in [
            ErrorCode::EofBeforeTagName,
            ErrorCode::EofInTag,
            ErrorCode::EofInComment,
            ErrorCode::InvalidFirstCharacterOfTagName,
            ErrorCode::MissingAttributeValue,
            ErrorCode::MissingDynamicDirectiveArgumentEnd,
            ErrorCode::MissingInterpolationEnd,
            ErrorCode::UnexpectedCharacterInAttributeName,
            ErrorCode::UnexpectedCharacterInUnquotedAttributeValue,
            ErrorCode::UnexpectedEqualsSignBeforeAttributeName,
            ErrorCode::MissingWhitespaceBetweenAttributes,
            ErrorCode::IncorrectlyClosedComment,
            ErrorCode::IncorrectlyOpenedComment,
            // is_recoverable ⊂ is_recovered_parse
            ErrorCode::DuplicateAttribute,
        ] {
            let error = CompilerError::new(code, None);
            assert!(error.is_recovered_parse(), "{code:?}");
        }
    }

    #[test]
    fn recovered_parse_rejects_unrecovered_codes() {
        for code in [
            ErrorCode::MissingEndTagName,
            ErrorCode::UnexpectedNullCharacter,
            ErrorCode::CdataInHtmlContent,
        ] {
            let error = CompilerError::new(code, None);
            assert!(!error.is_recovered_parse(), "{code:?}");
        }
    }
}
