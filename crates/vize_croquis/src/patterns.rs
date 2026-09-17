//! Shared RFC 823 semantic facts consumed by compiler and editor tooling.

use vize_armature::patterns::MatchArm;
use vize_carton::CompactString;

/// A subject is evaluated in the enclosing scope before arm bindings exist.
#[derive(Debug, Clone)]
pub struct MatchScopeData {
    pub subject: CompactString,
    pub start: u32,
    pub end: u32,
}

/// Parser spans are relative to the directive expression; `offset` rebases them.
#[derive(Debug, Clone)]
pub struct WhenScopeData {
    pub arm: MatchArm,
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct PatternDiagnostic {
    pub message: CompactString,
    pub start: u32,
    pub end: u32,
    pub warning: bool,
}
