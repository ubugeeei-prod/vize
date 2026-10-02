//! Source preparation and the typed-embed boundary for the L1 redesign.
//!
//! Design: <https://github.com/ubugeeei-prod/vize/issues/6836>.
//! These records describe syntax and source coordinates, not semantic facts.
//! Construct source preparation uses native entity decoding with checked coordinates.
//! Existing markup parsing does not select grammars or produce typed trees yet.

pub mod source;
pub mod syntax;

pub use source::{
    DecodeMap, DecodeSegment, DecodeSegmentKind, EmbedSource, SourceError, prepare_attribute_value,
    prepare_vue_interpolation_in,
};

/// The syntactic role selected by the markup dialect, independently of language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Program,
    Expr,
    HandlerBody,
    /// Composite pattern/expression syntax; dialect construction is still pending.
    ForHead,
    SlotParams,
    /// Composite syntax; the dialect builds its constituent language pieces.
    FilterChain,
}

/// Host language resolved once from the file's container syntax.
///
/// File resolution and mismatched script-language diagnostics remain
/// unimplemented under #6836. This enum does not select an emitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Js,
    Ts,
}

/// Independent language and syntactic role of one embedded source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grammar {
    pub shape: Shape,
    pub lang: Lang,
}

/// Borrowed input for a separate L1 embedded-tree artifact.
///
/// `syntax::parse_once` retains OXC programs, expressions, handler bodies and
/// slot parameters with comments and typed holes. Registering artifacts by
/// existing L0 node identity, composite shapes and file language resolution
/// are still unfinished under #6836.
#[derive(Debug, Clone, Copy)]
pub struct Embed<'a> {
    pub grammar: Grammar,
    pub source: EmbedSource<'a>,
}
