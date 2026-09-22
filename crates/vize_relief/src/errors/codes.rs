//! Stable, user-facing names for compiler error codes.
//!
//! A diagnostic shows `error[compiler/duplicate-attribute]` and `vize explain`
//! looks a code up by the same string, so the name is part of the CLI's
//! surface: it is the variant name in kebab case under `compiler/`, and it
//! never changes once published. The catalog (`vize_s0::i18n`) keys each
//! code's `message` and `help` on it in en, ja and zh;
//! `tests/tooling/davinci-diagnostic-catalog.test.ts` enumerates [`ErrorCode`]
//! from source and fails on a variant without a name, an entry or an `en`
//! message equal to [`ErrorCode::message`].

use super::ErrorCode;

impl ErrorCode {
    /// Every code, in declaration order.
    pub const ALL: [Self; 56] = [
        Self::AbruptClosingOfEmptyComment,
        Self::CdataInHtmlContent,
        Self::DuplicateAttribute,
        Self::EndTagWithAttributes,
        Self::EndTagWithTrailingSolidus,
        Self::EofBeforeTagName,
        Self::EofInCdata,
        Self::EofInComment,
        Self::EofInScriptHtmlCommentLikeText,
        Self::EofInTag,
        Self::IncorrectlyClosedComment,
        Self::IncorrectlyOpenedComment,
        Self::InvalidFirstCharacterOfTagName,
        Self::MissingAttributeValue,
        Self::MissingEndTagName,
        Self::MissingWhitespaceBetweenAttributes,
        Self::NestedComment,
        Self::UnexpectedCharacterInAttributeName,
        Self::UnexpectedCharacterInUnquotedAttributeValue,
        Self::UnexpectedEqualsSignBeforeAttributeName,
        Self::UnexpectedNullCharacter,
        Self::UnexpectedQuestionMarkInsteadOfTagName,
        Self::UnexpectedSolidusInTag,
        Self::InvalidEndTag,
        Self::MissingEndTag,
        Self::MissingInterpolationEnd,
        Self::MissingDynamicDirectiveArgumentEnd,
        Self::MissingDirectiveName,
        Self::MissingDirectiveModifier,
        Self::VIfNoExpression,
        Self::VIfSameKey,
        Self::VElseNoAdjacentIf,
        Self::VForNoExpression,
        Self::VForMalformedExpression,
        Self::VForTemplateKeyPlacement,
        Self::VBindNoExpression,
        Self::VBindSameNameShorthand,
        Self::VOnNoExpression,
        Self::VSlotUnexpectedDirectiveOnSlotOutlet,
        Self::VSlotMixedSlotUsage,
        Self::VSlotDuplicateSlotNames,
        Self::VSlotExtraneousDefaultSlotChildren,
        Self::VSlotMisplaced,
        Self::VModelNoExpression,
        Self::VModelMalformedExpression,
        Self::VModelOnScope,
        Self::VModelOnProps,
        Self::VModelArgOnElement,
        Self::VShowNoExpression,
        Self::InvalidExpression,
        Self::PrefixIdNotSupported,
        Self::ModuleModeNotSupported,
        Self::CacheHandlerNotSupported,
        Self::ScopeIdNotSupported,
        Self::UnhandledCodePath,
        Self::ExtendPoint,
    ];

    /// The code a diagnostic shows and `vize explain` accepts.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::AbruptClosingOfEmptyComment => "compiler/abrupt-closing-of-empty-comment",
            Self::CdataInHtmlContent => "compiler/cdata-in-html-content",
            Self::DuplicateAttribute => "compiler/duplicate-attribute",
            Self::EndTagWithAttributes => "compiler/end-tag-with-attributes",
            Self::EndTagWithTrailingSolidus => "compiler/end-tag-with-trailing-solidus",
            Self::EofBeforeTagName => "compiler/eof-before-tag-name",
            Self::EofInCdata => "compiler/eof-in-cdata",
            Self::EofInComment => "compiler/eof-in-comment",
            Self::EofInScriptHtmlCommentLikeText => "compiler/eof-in-script-html-comment-like-text",
            Self::EofInTag => "compiler/eof-in-tag",
            Self::IncorrectlyClosedComment => "compiler/incorrectly-closed-comment",
            Self::IncorrectlyOpenedComment => "compiler/incorrectly-opened-comment",
            Self::InvalidFirstCharacterOfTagName => "compiler/invalid-first-character-of-tag-name",
            Self::MissingAttributeValue => "compiler/missing-attribute-value",
            Self::MissingEndTagName => "compiler/missing-end-tag-name",
            Self::MissingWhitespaceBetweenAttributes => {
                "compiler/missing-whitespace-between-attributes"
            }
            Self::NestedComment => "compiler/nested-comment",
            Self::UnexpectedCharacterInAttributeName => {
                "compiler/unexpected-character-in-attribute-name"
            }
            Self::UnexpectedCharacterInUnquotedAttributeValue => {
                "compiler/unexpected-character-in-unquoted-attribute-value"
            }
            Self::UnexpectedEqualsSignBeforeAttributeName => {
                "compiler/unexpected-equals-sign-before-attribute-name"
            }
            Self::UnexpectedNullCharacter => "compiler/unexpected-null-character",
            Self::UnexpectedQuestionMarkInsteadOfTagName => {
                "compiler/unexpected-question-mark-instead-of-tag-name"
            }
            Self::UnexpectedSolidusInTag => "compiler/unexpected-solidus-in-tag",
            Self::InvalidEndTag => "compiler/invalid-end-tag",
            Self::MissingEndTag => "compiler/missing-end-tag",
            Self::MissingInterpolationEnd => "compiler/missing-interpolation-end",
            Self::MissingDynamicDirectiveArgumentEnd => {
                "compiler/missing-dynamic-directive-argument-end"
            }
            Self::MissingDirectiveName => "compiler/missing-directive-name",
            Self::MissingDirectiveModifier => "compiler/missing-directive-modifier",
            Self::VIfNoExpression => "compiler/v-if-no-expression",
            Self::VIfSameKey => "compiler/v-if-same-key",
            Self::VElseNoAdjacentIf => "compiler/v-else-no-adjacent-if",
            Self::VForNoExpression => "compiler/v-for-no-expression",
            Self::VForMalformedExpression => "compiler/v-for-malformed-expression",
            Self::VForTemplateKeyPlacement => "compiler/v-for-template-key-placement",
            Self::VBindNoExpression => "compiler/v-bind-no-expression",
            Self::VBindSameNameShorthand => "compiler/v-bind-same-name-shorthand",
            Self::VOnNoExpression => "compiler/v-on-no-expression",
            Self::VSlotUnexpectedDirectiveOnSlotOutlet => {
                "compiler/v-slot-unexpected-directive-on-slot-outlet"
            }
            Self::VSlotMixedSlotUsage => "compiler/v-slot-mixed-slot-usage",
            Self::VSlotDuplicateSlotNames => "compiler/v-slot-duplicate-slot-names",
            Self::VSlotExtraneousDefaultSlotChildren => {
                "compiler/v-slot-extraneous-default-slot-children"
            }
            Self::VSlotMisplaced => "compiler/v-slot-misplaced",
            Self::VModelNoExpression => "compiler/v-model-no-expression",
            Self::VModelMalformedExpression => "compiler/v-model-malformed-expression",
            Self::VModelOnScope => "compiler/v-model-on-scope",
            Self::VModelOnProps => "compiler/v-model-on-props",
            Self::VModelArgOnElement => "compiler/v-model-arg-on-element",
            Self::VShowNoExpression => "compiler/v-show-no-expression",
            Self::InvalidExpression => "compiler/invalid-expression",
            Self::PrefixIdNotSupported => "compiler/prefix-id-not-supported",
            Self::ModuleModeNotSupported => "compiler/module-mode-not-supported",
            Self::CacheHandlerNotSupported => "compiler/cache-handler-not-supported",
            Self::ScopeIdNotSupported => "compiler/scope-id-not-supported",
            Self::UnhandledCodePath => "compiler/unhandled-code-path",
            Self::ExtendPoint => "compiler/extend-point",
        }
    }

    /// The code whose [`Self::code`] is `code`.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.code() == code)
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorCode;

    #[test]
    fn every_code_round_trips_through_its_name_and_names_are_unique() {
        for code in ErrorCode::ALL {
            assert_eq!(ErrorCode::from_code(code.code()), Some(code));
        }
        let mut names: Vec<&str> = ErrorCode::ALL.iter().map(|code| code.code()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), ErrorCode::ALL.len());
        assert_eq!(ErrorCode::from_code("compiler/no-such-code"), None);
    }
}
