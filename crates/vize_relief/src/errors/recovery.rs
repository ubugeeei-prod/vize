//! Classification of parse errors whose recovery yields a complete tree.

use super::{CompilerError, ErrorCode};

/// Parse error codes for which the template parser documents a concrete
/// recovery strategy: parsing continues past the defect and still yields a
/// complete tree, so semantic analysis over that tree stays meaningful
/// (#3294).
///
/// This is the single source of truth for that contract. `vize_armature`'s
/// `recovery_error_message` consults
/// [`ErrorCode::has_documented_parse_recovery`] before building a message, so
/// the parser cannot describe a recovery for a code this list does not
/// classify, and a test there pins every entry below to a real message. Keep
/// the two in sync by editing this list.
pub const RECOVERED_PARSE_CODES: &[ErrorCode] = &[
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
];

impl ErrorCode {
    /// Returns true when the template parser documents a concrete recovery
    /// strategy for this code, i.e. it appears in [`RECOVERED_PARSE_CODES`].
    #[must_use]
    pub fn has_documented_parse_recovery(self) -> bool {
        RECOVERED_PARSE_CODES.contains(&self)
    }
}

impl CompilerError {
    /// Returns true when the parse defect behind this error still left a
    /// complete tree behind, so semantic analysis over that tree stays
    /// meaningful (#3294).
    ///
    /// This is deliberately broader than [`CompilerError::is_recoverable`],
    /// which additionally implies downstream codegen may proceed without
    /// gating. It covers every code in [`RECOVERED_PARSE_CODES`] (the codes
    /// `vize_armature` builds a recovery message for) plus the already
    /// recoverable ones.
    #[must_use]
    pub fn is_recovered_parse(&self) -> bool {
        self.code.has_documented_parse_recovery() || self.is_recoverable()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompilerError, ErrorCode};
    use super::RECOVERED_PARSE_CODES;

    #[test]
    fn recovered_parse_covers_the_documented_recovery_codes() {
        for &code in RECOVERED_PARSE_CODES {
            let error = CompilerError::new(code, None);
            assert!(error.is_recovered_parse(), "{code:?}");
            assert!(
                code.is_parse_error(),
                "{code:?} is not a parse-level code, so the parser cannot recover from it"
            );
        }
        // is_recoverable ⊂ is_recovered_parse, and needs no recovery message.
        assert!(CompilerError::new(ErrorCode::DuplicateAttribute, None).is_recovered_parse());
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
