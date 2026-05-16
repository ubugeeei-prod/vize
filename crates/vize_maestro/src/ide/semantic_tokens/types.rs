//! Type definitions for semantic tokens.
//!
//! Defines token types, modifiers, and internal token representation.

use tower_lsp::lsp_types::{SemanticTokenModifier, SemanticTokenType};

/// Token types supported by the semantic tokens provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TokenType {
    Namespace = 0,
    Type = 1,
    Class = 2,
    Enum = 3,
    Interface = 4,
    Struct = 5,
    TypeParameter = 6,
    Parameter = 7,
    Variable = 8,
    Property = 9,
    EnumMember = 10,
    Event = 11,
    Function = 12,
    Method = 13,
    Macro = 14,
    Keyword = 15,
    Modifier = 16,
    Comment = 17,
    String = 18,
    Number = 19,
    Regexp = 20,
    Operator = 21,
    Decorator = 22,
}

impl TokenType {
    /// Get all token types for legend.
    pub fn legend() -> Vec<SemanticTokenType> {
        vec![
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::TYPE,
            SemanticTokenType::CLASS,
            SemanticTokenType::ENUM,
            SemanticTokenType::INTERFACE,
            SemanticTokenType::STRUCT,
            SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::EVENT,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::METHOD,
            SemanticTokenType::MACRO,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::MODIFIER,
            SemanticTokenType::COMMENT,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::REGEXP,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::DECORATOR,
        ]
    }
}

/// Token modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TokenModifier {
    Declaration = 0,
    Definition = 1,
    Readonly = 2,
    Static = 3,
    Deprecated = 4,
    Abstract = 5,
    Async = 6,
    Modification = 7,
    Documentation = 8,
    DefaultLibrary = 9,
}

impl TokenModifier {
    /// Get all token modifiers for legend.
    pub fn legend() -> Vec<SemanticTokenModifier> {
        vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::STATIC,
            SemanticTokenModifier::DEPRECATED,
            SemanticTokenModifier::ABSTRACT,
            SemanticTokenModifier::ASYNC,
            SemanticTokenModifier::MODIFICATION,
            SemanticTokenModifier::DOCUMENTATION,
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ]
    }

    /// Encode modifiers as a bitmask.
    pub fn encode(modifiers: &[TokenModifier]) -> u32 {
        modifiers
            .iter()
            .fold(0u32, |acc, m| acc | (1 << (*m as u32)))
    }
}

/// A semantic token with absolute position.
#[derive(Debug, Clone)]
pub(crate) struct AbsoluteToken {
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: u32,
    pub modifiers: u32,
}
