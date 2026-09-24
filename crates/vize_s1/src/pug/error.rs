//! Recoverable pug surface diagnostics, by code and authored byte offset.
//!
//! Each code is a place where the pinned `pug` throws; the S1 parser
//! records it and recovers structurally instead (see the crate hole
//! policy), so rendering the tree to S2 diagnostics is the lowering's job.

/// A pug lexing/parsing error the surface parser recovered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PugErrorCode {
    /// `INVALID_INDENTATION`: tabs and spaces mixed in one indent.
    InvalidIndentation,
    /// `INCONSISTENT_INDENTATION`: an outdent to no open level.
    InconsistentIndentation,
    /// `NO_END_BRACKET`: an unclosed `(`, `#[`, `#{`.
    NoEndBracket,
    /// `BRACKET_MISMATCH`.
    BracketMismatch,
    /// `INVALID_CLASS_NAME`.
    InvalidClassName,
    /// `INVALID_ID`.
    InvalidId,
    /// `INVALID_KEY_CHARACTER` or a bracket error inside an attribute.
    InvalidAttribute,
    /// `UNEXPECTED_TEXT`: no lexer rule matched.
    UnexpectedText,
    /// A keyword without its required operand (`case`, `when`,
    /// `default …`, `while`).
    MalformedKeyword,
    /// `INVALID_TOKEN`: a token pug's parser does not accept here.
    InvalidToken,
    /// The source is too large to address with `u32` offsets; nothing was
    /// parsed (the whole source is the end-of-file token's leading).
    SourceTooLarge,
}

impl PugErrorCode {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidIndentation => {
                "Invalid indentation, you can use tabs or spaces but not both"
            }
            Self::InconsistentIndentation => "Inconsistent indentation",
            Self::NoEndBracket => "The end of the string reached with no closing bracket",
            Self::BracketMismatch => "Mismatched bracket",
            Self::InvalidClassName => "Class names must contain at least one letter or underscore",
            Self::InvalidId => "Invalid ID",
            Self::InvalidAttribute => "Invalid attribute",
            Self::UnexpectedText => "Unexpected text",
            Self::MalformedKeyword => "Malformed pug keyword",
            Self::InvalidToken => "Unexpected token",
            Self::SourceTooLarge => "Pug source is too large to address with u32 offsets",
        }
    }
}

/// One recovered error at an authored byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PugError {
    pub code: PugErrorCode,
    pub offset: u32,
}
