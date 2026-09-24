//! Parsed long-form patterns for the RFC 823 compiler and tooling consumers.
//!
//! Spans are expression-relative bytes. `parse_match_attribute` rebases decoded
//! spans into the authored attribute; consumers then add the attribute offset.
//! Grammar follows the Vue reference at 83c5fcc2f (see patterns/LICENSE).

mod aggregate;
mod attribute;
mod parser;
#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
mod tokens;

pub use attribute::{attribute_source_offset, parse_match_attribute};
use oxc_span::Span;
use vize_s0::String;

/// A fully parsed arm. No JavaScript expressions are evaluated by this parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub bindings: Vec<PatternBinding>,
    pub guard: Option<PatternExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchPattern {
    pub span: Span,
    pub kind: PatternKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternKind {
    Wildcard,
    Literal(PatternExpression),
    Value(PatternExpression),
    Binding(PatternBinding),
    As {
        pattern: Box<MatchPattern>,
        binding: PatternBinding,
    },
    Or(Vec<MatchPattern>),
    Object {
        properties: Vec<PatternProperty>,
        rest: Option<PatternRest>,
    },
    Array {
        elements: Vec<MatchPattern>,
        rest: Option<PatternRest>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternBinding {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternExpression {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternProperty {
    /// An ECMAScript string literal denoting the property key.
    pub key: PatternExpression,
    /// Canonical UTF-16 key, retaining lone surrogates for duplicate checks.
    pub key_value: Vec<u16>,
    pub pattern: MatchPattern,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternRest {
    pub binding: Option<PatternBinding>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternSyntaxError {
    pub message: String,
    pub offset: u32,
}

/// Parse one complete `v-when` expression, rejecting unsupported or trailing syntax.
pub fn parse_match_pattern(source: &str) -> Result<MatchArm, PatternSyntaxError> {
    parser::PatternParser::new(source).arm()
}
