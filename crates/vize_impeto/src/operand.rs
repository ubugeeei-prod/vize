//! Value operands stored beside the compact operation graph.
//!
//! Text belongs to the S3 arena, not to an earlier stage's scratch arena.
//! Opaque and foreign values retain their classification and must not be
//! interpreted as JavaScript merely because their source text looks familiar.

use vize_s0::Span;

use crate::op::{OpId, RegionId};

/// The semantic position occupied by an operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandRole {
    Tag,
    Namespace,
    Attribute,
    Text,
    Comment,
    BindingKind,
    Name,
    Value,
    Modifier,
    Condition,
    ForSource,
    ForValue,
    ForKey,
    ForIndex,
    ModelRead,
    ModelWrite,
    ModelAttribute,
    Params,
}

impl OperandRole {
    pub const ALL: [Self; 18] = [
        Self::Tag,
        Self::Namespace,
        Self::Attribute,
        Self::Text,
        Self::Comment,
        Self::BindingKind,
        Self::Name,
        Self::Value,
        Self::Modifier,
        Self::Condition,
        Self::ForSource,
        Self::ForValue,
        Self::ForKey,
        Self::ForIndex,
        Self::ModelRead,
        Self::ModelWrite,
        Self::ModelAttribute,
        Self::Params,
    ];

    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.as_str() == text)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tag => "tag",
            Self::Namespace => "namespace",
            Self::Attribute => "attribute",
            Self::Text => "text",
            Self::Comment => "comment",
            Self::BindingKind => "binding-kind",
            Self::Name => "name",
            Self::Value => "value",
            Self::Modifier => "modifier",
            Self::Condition => "condition",
            Self::ForSource => "for-source",
            Self::ForValue => "for-value",
            Self::ForKey => "for-key",
            Self::ForIndex => "for-index",
            Self::ModelRead => "model-read",
            Self::ModelWrite => "model-write",
            Self::ModelAttribute => "model-attribute",
            Self::Params => "params",
        }
    }
}

/// Missing, literal, and expression values are deliberately distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Absent,
    Literal,
    Js,
    Opaque,
    Foreign,
    Filter,
}

impl ValueKind {
    pub const ALL: [Self; 6] = [
        Self::Absent,
        Self::Literal,
        Self::Js,
        Self::Opaque,
        Self::Foreign,
        Self::Filter,
    ];

    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == text)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Literal => "literal",
            Self::Js => "js",
            Self::Opaque => "opaque",
            Self::Foreign => "foreign",
            Self::Filter => "vue.filter",
        }
    }
}

/// Expression identity is not semantic equality, especially for opaque values.
#[derive(Debug, Clone, Copy)]
pub struct OperandValue<'a> {
    pub kind: ValueKind,
    pub text: &'a str,
    /// Opaque reason or foreign dialect, empty for other kinds.
    pub qualifier: &'a str,
    pub span: Span,
}

impl OperandValue<'_> {
    /// Check the representation, without interpreting expression source.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.span.start <= self.span.end
            && (self.kind != ValueKind::Absent || self.text.is_empty())
            && match self.kind {
                ValueKind::Opaque | ValueKind::Foreign => !self.qualifier.is_empty(),
                _ => self.qualifier.is_empty(),
            }
    }
}

/// One operand. Repeated attributes and modifiers retain authored order.
#[derive(Debug, Clone, Copy)]
pub struct Operand<'a> {
    pub op: OpId,
    pub role: OperandRole,
    /// Materialized element/component/outlet addressed by an attached binding.
    pub target: Option<OpId>,
    /// Branch region selected by a condition operand.
    pub region: Option<RegionId>,
    /// Static attribute name, absent outside attribute roles.
    pub name: Option<&'a str>,
    pub value: OperandValue<'a>,
}

const _: () = assert!(!core::mem::needs_drop::<Operand<'static>>());
