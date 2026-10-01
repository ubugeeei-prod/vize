//! Source preparation and the typed-embed boundary for the L1 redesign.
//!
//! Design: <https://github.com/ubugeeei-prod/vize/issues/6836>.
//! These records describe syntax and source coordinates, not semantic facts.
//! Attribute preparation uses native entity decoding with checked coordinates.
//! Existing markup parsing does not select grammars or produce typed trees yet.

pub mod source;

pub use source::{
    DecodeMap, DecodeSegment, DecodeSegmentKind, EmbedSource, SourceError, prepare_attribute_value,
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
/// Resolution, mismatched script-language diagnostics and language providers
/// remain unimplemented under #6836. This enum does not select an emitter.
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

/// Borrowed source contract for a future separate L1 embedded-tree artifact.
///
/// TODO (#6836): parse an OXC tree once, retain comments and typed holes, and
/// register the artifact by the L0 node identity after #6833 extraction. No new
/// identity allocator, tree parser or legacy-backed semantics is supplied here.
#[derive(Debug, Clone, Copy)]
pub struct Embed<'a> {
    pub grammar: Grammar,
    pub source: EmbedSource<'a>,
}
