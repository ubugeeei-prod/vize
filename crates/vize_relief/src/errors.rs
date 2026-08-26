//! Compiler error types and codes.
mod compatibility;
pub mod recovery;
mod render;
use crate::SourceLocation;
pub use render::CompilerErrorWithSource;
use thiserror::Error;
use vize_s0::{CompactString, ToCompactString};
/// Compiler error
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct CompilerError {
    pub code: ErrorCode,
    pub message: CompactString,
    pub loc: Option<SourceLocation>,
}

impl CompilerError {
    pub fn new(code: ErrorCode, loc: Option<SourceLocation>) -> Self {
        Self {
            message: code.message().to_compact_string(),
            code,
            loc,
        }
    }

    pub fn with_message(
        code: ErrorCode,
        message: impl Into<CompactString>,
        loc: Option<SourceLocation>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            loc,
        }
    }

    /// Returns true when this diagnostic is a parser-level warning that
    /// downstream codegen can recover from. Mirrors `@vue/compiler-sfc`'s
    /// classification: a duplicate attribute is reported but does not
    /// gate render emission (#958).
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(self.code, ErrorCode::DuplicateAttribute) || self.is_compatibility_notice()
    }
}

/// Error codes for compiler errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ErrorCode {
    // Parse errors
    AbruptClosingOfEmptyComment = 0,
    CdataInHtmlContent = 1,
    DuplicateAttribute = 2,
    EndTagWithAttributes = 3,
    EndTagWithTrailingSolidus = 4,
    EofBeforeTagName = 5,
    EofInCdata = 6,
    EofInComment = 7,
    EofInScriptHtmlCommentLikeText = 8,
    EofInTag = 9,
    IncorrectlyClosedComment = 10,
    IncorrectlyOpenedComment = 11,
    InvalidFirstCharacterOfTagName = 12,
    MissingAttributeValue = 13,
    MissingEndTagName = 14,
    MissingWhitespaceBetweenAttributes = 15,
    NestedComment = 16,
    UnexpectedCharacterInAttributeName = 17,
    UnexpectedCharacterInUnquotedAttributeValue = 18,
    UnexpectedEqualsSignBeforeAttributeName = 19,
    UnexpectedNullCharacter = 20,
    UnexpectedQuestionMarkInsteadOfTagName = 21,
    UnexpectedSolidusInTag = 22,

    // Vue-specific parse errors
    InvalidEndTag = 23,
    MissingEndTag = 24,
    MissingInterpolationEnd = 25,
    MissingDynamicDirectiveArgumentEnd = 26,
    MissingDirectiveName = 27,
    MissingDirectiveModifier = 28,

    // Transform errors
    VIfNoExpression = 29,
    VIfSameKey = 30,
    VElseNoAdjacentIf = 31,
    VForNoExpression = 32,
    VForMalformedExpression = 33,
    VForTemplateKeyPlacement = 34,
    VBindNoExpression = 35,
    VBindSameNameShorthand = 36,
    VOnNoExpression = 37,
    VSlotUnexpectedDirectiveOnSlotOutlet = 38,
    VSlotMixedSlotUsage = 39,
    VSlotDuplicateSlotNames = 40,
    VSlotExtraneousDefaultSlotChildren = 41,
    VSlotMisplaced = 42,
    VModelNoExpression = 43,
    VModelMalformedExpression = 44,
    VModelOnScope = 45,
    VModelOnProps = 46,
    VModelArgOnElement = 47,
    VShowNoExpression = 48,
    /// A template expression failed to parse as JavaScript/TypeScript.
    /// Mirrors `@vue/compiler-core`'s `X_INVALID_EXPRESSION`.
    InvalidExpression = 49,

    // Generic errors
    PrefixIdNotSupported = 50,
    ModuleModeNotSupported = 51,
    CacheHandlerNotSupported = 52,
    ScopeIdNotSupported = 53,

    // Extended errors
    UnhandledCodePath = 100,
    ExtendPoint = 1000,
}

impl ErrorCode {
    pub fn message(&self) -> &'static str {
        match self {
            Self::AbruptClosingOfEmptyComment => "Illegal comment.",
            Self::CdataInHtmlContent => "CDATA section is allowed only in XML context.",
            Self::DuplicateAttribute => "Duplicate attribute.",
            Self::EndTagWithAttributes => "End tag cannot have attributes.",
            Self::EndTagWithTrailingSolidus => "Trailing solidus not allowed in end tags.",
            Self::EofBeforeTagName => "Unexpected EOF in tag.",
            Self::EofInCdata => "EOF in CDATA section.",
            Self::EofInComment => "EOF in comment.",
            Self::EofInScriptHtmlCommentLikeText => "EOF in script.",
            Self::EofInTag => "EOF in tag.",
            Self::IncorrectlyClosedComment => "Incorrectly closed comment.",
            Self::IncorrectlyOpenedComment => "Incorrectly opened comment.",
            Self::InvalidFirstCharacterOfTagName => "Invalid first character of tag name.",
            Self::MissingAttributeValue => "Attribute value expected.",
            Self::MissingEndTagName => "End tag name expected.",
            Self::MissingWhitespaceBetweenAttributes => "Whitespace expected between attributes.",
            Self::NestedComment => "Nested comments are not allowed.",
            Self::UnexpectedCharacterInAttributeName => "Unexpected character in attribute name.",
            Self::UnexpectedCharacterInUnquotedAttributeValue => {
                "Unexpected character in unquoted attribute value."
            }
            Self::UnexpectedEqualsSignBeforeAttributeName => {
                "Unexpected equals sign before attribute name."
            }
            Self::UnexpectedNullCharacter => "Unexpected null character.",
            Self::UnexpectedQuestionMarkInsteadOfTagName => "Invalid tag name.",
            Self::UnexpectedSolidusInTag => "Unexpected solidus in tag.",

            Self::InvalidEndTag => "Invalid end tag.",
            Self::MissingEndTag => "Element is missing end tag.",
            Self::MissingInterpolationEnd => "Interpolation end sign was not found.",
            Self::MissingDynamicDirectiveArgumentEnd => {
                "End bracket for dynamic directive argument was not found."
            }
            Self::MissingDirectiveName => "Directive name is missing.",
            Self::MissingDirectiveModifier => "Directive modifier is expected.",

            Self::VIfNoExpression => "v-if/v-else-if is missing expression.",
            Self::VIfSameKey => "v-if/v-else-if branches must use unique keys.",
            Self::VElseNoAdjacentIf => "v-else/v-else-if has no adjacent v-if.",
            Self::VForNoExpression => "v-for is missing expression.",
            Self::VForMalformedExpression => "v-for has invalid expression.",
            Self::VForTemplateKeyPlacement => {
                "<template v-for> key should be placed on the <template> tag."
            }
            Self::VBindNoExpression => "v-bind is missing expression.",
            Self::VBindSameNameShorthand => "v-bind shorthand requires prop name.",
            Self::VOnNoExpression => "v-on is missing expression.",
            Self::VSlotUnexpectedDirectiveOnSlotOutlet => {
                "Unexpected custom directive on <slot> outlet."
            }
            Self::VSlotMixedSlotUsage => "Mixed v-slot usage with named slots detected.",
            Self::VSlotDuplicateSlotNames => "Duplicate slot names detected.",
            Self::VSlotExtraneousDefaultSlotChildren => {
                "Extraneous children found when component already has an explicit default slot."
            }
            Self::VSlotMisplaced => "v-slot can only be used on components or <template> tags.",
            Self::VModelNoExpression => "v-model is missing expression.",
            Self::VModelMalformedExpression => {
                "v-model value must be a valid JavaScript member expression."
            }
            Self::VModelOnScope => "v-model cannot be used on v-for or v-slot scope variables.",
            Self::VModelOnProps => "v-model cannot be used on props.",
            Self::VModelArgOnElement => "v-model argument is not supported on plain elements.",
            Self::VShowNoExpression => "v-show is missing expression.",
            Self::InvalidExpression => "Error parsing JavaScript expression.",

            Self::PrefixIdNotSupported => "prefixIdentifiers option is not supported in this mode.",
            Self::ModuleModeNotSupported => "ES module mode is not supported in this mode.",
            Self::CacheHandlerNotSupported => "cacheHandlers option is not supported in this mode.",
            Self::ScopeIdNotSupported => "scopeId option is not supported in this mode.",

            Self::UnhandledCodePath => "Unhandled code path.",
            Self::ExtendPoint => "Extension point.",
        }
    }

    pub fn is_parse_error(&self) -> bool {
        (*self as u16) < (Self::VIfNoExpression as u16)
    }

    pub fn is_transform_error(&self) -> bool {
        let code = *self as u16;
        code >= (Self::VIfNoExpression as u16) && code < (Self::PrefixIdNotSupported as u16)
    }

    /// Returns true when this code is a recovery-level diagnostic (a warning)
    /// rather than a hard error. `ExtendPoint` is the code the parser uses for
    /// HTML tree-construction recovery notes (e.g. self-closing rewrites,
    /// fostered elements, auto-closed `<p>`). These describe spec-compliant
    /// repairs the parser already applied, so downstream consumers — notably
    /// the `vize_canon` virtual-TS codegen — must NOT treat them as a reason to
    /// abort and fall back to a stub component.
    #[must_use]
    pub fn is_recovery(&self) -> bool {
        matches!(self, Self::ExtendPoint)
    }
}

/// Result type for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;

#[cfg(test)]
mod tests;
