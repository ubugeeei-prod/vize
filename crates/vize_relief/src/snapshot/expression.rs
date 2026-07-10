use vize_carton::String;

use crate::{ConstantType, RuntimeHelper, SourceLocation};

use super::{SnapshotInterpolation, SnapshotText};

/// Owned form of a Relief expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotExpression {
    Simple(SnapshotSimpleExpression),
    Compound(SnapshotCompoundExpression),
}

impl SnapshotExpression {
    /// Source span covering the expression.
    pub const fn location(&self) -> &SourceLocation {
        match self {
            Self::Simple(expression) => &expression.location,
            Self::Compound(expression) => &expression.location,
        }
    }
}

/// Owned simple expression, including parser and transform annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSimpleExpression {
    pub content: String,
    pub is_static: bool,
    pub const_type: ConstantType,
    pub location: SourceLocation,
    /// Raw parsed JavaScript expression, when Relief retained one.
    pub js_raw: Option<String>,
    pub identifiers: Option<Vec<String>>,
    pub is_handler_key: bool,
    pub is_ref_transformed: bool,
}

/// Owned compound expression and its exact ordered parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotCompoundExpression {
    pub children: Vec<SnapshotCompoundChild>,
    pub location: SourceLocation,
    pub identifiers: Option<Vec<String>>,
    pub is_handler_key: bool,
}

/// One ordered part of a compound expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotCompoundChild {
    Simple(SnapshotSimpleExpression),
    Compound(SnapshotCompoundExpression),
    Interpolation(SnapshotInterpolation),
    Text(SnapshotText),
    String(String),
    Symbol(RuntimeHelper),
}

/// Owned parser result retained by a `v-for` directive or node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotForParseResult {
    pub source: SnapshotExpression,
    pub value: Option<SnapshotExpression>,
    pub key: Option<SnapshotExpression>,
    pub index: Option<SnapshotExpression>,
    pub finalized: bool,
}
