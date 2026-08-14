// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
mod error_code_tests {
    use super::super::{CompilerError, ErrorCode};

    #[test]
    fn compiler_error_new() {
        let err = CompilerError::new(ErrorCode::EofInTag, None);
        assert_eq!(err.code, ErrorCode::EofInTag);
        assert_eq!(err.message, "EOF in tag.");
        assert!(err.loc.is_none());
    }

    #[test]
    fn compiler_error_with_message() {
        let err =
            CompilerError::with_message(ErrorCode::UnhandledCodePath, "custom error message", None);
        assert_eq!(err.code, ErrorCode::UnhandledCodePath);
        assert_eq!(err.message, "custom error message");
    }

    #[test]
    fn error_code_messages_not_empty() {
        let codes = [
            ErrorCode::AbruptClosingOfEmptyComment,
            ErrorCode::CdataInHtmlContent,
            ErrorCode::DuplicateAttribute,
            ErrorCode::EndTagWithAttributes,
            ErrorCode::EofInTag,
            ErrorCode::InvalidEndTag,
            ErrorCode::MissingEndTag,
            ErrorCode::MissingInterpolationEnd,
            ErrorCode::MissingDirectiveName,
            ErrorCode::MissingDirectiveModifier,
            ErrorCode::VIfNoExpression,
            ErrorCode::VForNoExpression,
            ErrorCode::VBindNoExpression,
            ErrorCode::VOnNoExpression,
            ErrorCode::VModelNoExpression,
            ErrorCode::VShowNoExpression,
            ErrorCode::PrefixIdNotSupported,
            ErrorCode::UnhandledCodePath,
            ErrorCode::ExtendPoint,
        ];
        for code in &codes {
            assert!(!code.message().is_empty(), "{:?} has empty message", code);
        }
    }

    #[test]
    fn is_recovery_only_for_extend_point() {
        assert!(ErrorCode::ExtendPoint.is_recovery());
        assert!(!ErrorCode::InvalidEndTag.is_recovery());
        assert!(!ErrorCode::UnexpectedSolidusInTag.is_recovery());
        assert!(!ErrorCode::DuplicateAttribute.is_recovery());
        assert!(!ErrorCode::UnhandledCodePath.is_recovery());
    }

    #[test]
    fn is_parse_error_true() {
        let parse_errors = [
            ErrorCode::AbruptClosingOfEmptyComment,
            ErrorCode::CdataInHtmlContent,
            ErrorCode::DuplicateAttribute,
            ErrorCode::EofInTag,
            ErrorCode::InvalidEndTag,
            ErrorCode::MissingEndTag,
            ErrorCode::MissingInterpolationEnd,
            ErrorCode::MissingDirectiveName,
            ErrorCode::MissingDirectiveModifier,
        ];
        for code in &parse_errors {
            assert!(code.is_parse_error(), "{:?} should be parse error", code);
        }
    }

    #[test]
    fn is_parse_error_false_for_transform() {
        assert!(!ErrorCode::VIfNoExpression.is_parse_error());
        assert!(!ErrorCode::VShowNoExpression.is_parse_error());
        assert!(!ErrorCode::PrefixIdNotSupported.is_parse_error());
    }

    #[test]
    fn is_transform_error_true() {
        let transform_errors = [
            ErrorCode::VIfNoExpression,
            ErrorCode::VIfSameKey,
            ErrorCode::VElseNoAdjacentIf,
            ErrorCode::VForNoExpression,
            ErrorCode::VBindNoExpression,
            ErrorCode::VOnNoExpression,
            ErrorCode::VModelNoExpression,
            ErrorCode::VShowNoExpression,
            ErrorCode::InvalidExpression,
        ];
        for code in &transform_errors {
            assert!(
                code.is_transform_error(),
                "{:?} should be transform error",
                code
            );
        }
    }

    #[test]
    fn is_transform_error_false() {
        // Parse errors should not be transform errors
        assert!(!ErrorCode::EofInTag.is_transform_error());
        assert!(!ErrorCode::MissingDirectiveModifier.is_transform_error());
        // Generic errors should not be transform errors
        assert!(!ErrorCode::PrefixIdNotSupported.is_transform_error());
    }

    #[test]
    fn boundary_error_codes() {
        // MissingDirectiveModifier (28) is the last parse error
        assert!(ErrorCode::MissingDirectiveModifier.is_parse_error());
        assert!(!ErrorCode::MissingDirectiveModifier.is_transform_error());

        // VIfNoExpression (29) is the first transform error
        assert!(!ErrorCode::VIfNoExpression.is_parse_error());
        assert!(ErrorCode::VIfNoExpression.is_transform_error());

        // InvalidExpression (49) is the last transform error
        assert!(ErrorCode::InvalidExpression.is_transform_error());
        assert!(!ErrorCode::InvalidExpression.is_parse_error());

        // PrefixIdNotSupported (50) is neither
        assert!(!ErrorCode::PrefixIdNotSupported.is_parse_error());
        assert!(!ErrorCode::PrefixIdNotSupported.is_transform_error());
    }

    #[test]
    fn mutual_exclusion() {
        let all_codes = [
            ErrorCode::AbruptClosingOfEmptyComment,
            ErrorCode::CdataInHtmlContent,
            ErrorCode::DuplicateAttribute,
            ErrorCode::EndTagWithAttributes,
            ErrorCode::EndTagWithTrailingSolidus,
            ErrorCode::EofBeforeTagName,
            ErrorCode::EofInCdata,
            ErrorCode::EofInComment,
            ErrorCode::EofInScriptHtmlCommentLikeText,
            ErrorCode::EofInTag,
            ErrorCode::IncorrectlyClosedComment,
            ErrorCode::IncorrectlyOpenedComment,
            ErrorCode::InvalidFirstCharacterOfTagName,
            ErrorCode::MissingAttributeValue,
            ErrorCode::MissingEndTagName,
            ErrorCode::MissingWhitespaceBetweenAttributes,
            ErrorCode::NestedComment,
            ErrorCode::UnexpectedCharacterInAttributeName,
            ErrorCode::UnexpectedCharacterInUnquotedAttributeValue,
            ErrorCode::UnexpectedEqualsSignBeforeAttributeName,
            ErrorCode::UnexpectedNullCharacter,
            ErrorCode::UnexpectedQuestionMarkInsteadOfTagName,
            ErrorCode::UnexpectedSolidusInTag,
            ErrorCode::InvalidEndTag,
            ErrorCode::MissingEndTag,
            ErrorCode::MissingInterpolationEnd,
            ErrorCode::MissingDynamicDirectiveArgumentEnd,
            ErrorCode::MissingDirectiveName,
            ErrorCode::MissingDirectiveModifier,
            ErrorCode::VIfNoExpression,
            ErrorCode::VIfSameKey,
            ErrorCode::VElseNoAdjacentIf,
            ErrorCode::VForNoExpression,
            ErrorCode::VForMalformedExpression,
            ErrorCode::VForTemplateKeyPlacement,
            ErrorCode::VBindNoExpression,
            ErrorCode::VBindSameNameShorthand,
            ErrorCode::VOnNoExpression,
            ErrorCode::VSlotUnexpectedDirectiveOnSlotOutlet,
            ErrorCode::VSlotMixedSlotUsage,
            ErrorCode::VSlotDuplicateSlotNames,
            ErrorCode::VSlotExtraneousDefaultSlotChildren,
            ErrorCode::VSlotMisplaced,
            ErrorCode::VModelNoExpression,
            ErrorCode::VModelMalformedExpression,
            ErrorCode::VModelOnScope,
            ErrorCode::VModelOnProps,
            ErrorCode::VModelArgOnElement,
            ErrorCode::VShowNoExpression,
            ErrorCode::InvalidExpression,
            ErrorCode::PrefixIdNotSupported,
            ErrorCode::ModuleModeNotSupported,
            ErrorCode::CacheHandlerNotSupported,
            ErrorCode::ScopeIdNotSupported,
            ErrorCode::UnhandledCodePath,
            ErrorCode::ExtendPoint,
        ];
        for code in &all_codes {
            assert!(
                !(code.is_parse_error() && code.is_transform_error()),
                "{:?} should not be both parse and transform error",
                code
            );
        }
    }
}
