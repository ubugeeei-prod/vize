use vize_carton::String;

use crate::SourceLocation;

use super::{SnapshotExpression, SnapshotForParseResult, SnapshotSimpleExpression, SnapshotText};

/// Owned element property in original attribute order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotProp {
    Attribute(SnapshotAttribute),
    Directive(Box<SnapshotDirective>),
}

impl SnapshotProp {
    /// Complete source span of the attribute or directive.
    pub const fn location(&self) -> &SourceLocation {
        match self {
            Self::Attribute(attribute) => &attribute.location,
            Self::Directive(directive) => &directive.location,
        }
    }
}

/// Owned static attribute syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotAttribute {
    pub name: String,
    pub name_location: SourceLocation,
    pub value: Option<SnapshotText>,
    pub location: SourceLocation,
}

/// Owned directive syntax without render or semantic normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotDirective {
    /// Normalized directive name, such as `bind` or `on`.
    pub name: String,
    /// Exact spelling, including shorthand and modifiers, when retained.
    pub raw_name: Option<String>,
    pub expression: Option<SnapshotExpression>,
    pub argument: Option<SnapshotExpression>,
    pub modifiers: Vec<SnapshotSimpleExpression>,
    pub for_parse_result: Option<SnapshotForParseResult>,
    pub shorthand: bool,
    pub location: SourceLocation,
}
