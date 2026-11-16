//! Sub-requests an [`ExpressionQuery`](super::ExpressionQuery) carries.
use vize_l0::String as CompactString;

/// Heritage lookups for no-misused-promises' `checksVoidReturn.inheritedMethods`.
#[derive(Clone, Debug)]
pub(in crate::linter) struct HeritageQuery {
    /// Read the heritage types as the base types of the declared type
    /// (`extends`); otherwise the expression type is the heritage type.
    pub base_types: bool,
    /// Non-static member names of the class or interface.
    pub members: Vec<CompactString>,
}

/// Operand ranges for a computed member access.
#[derive(Clone, Copy, Debug)]
pub(in crate::linter) struct ComputedMemberQuery {
    pub object: (u32, u32),
    pub property: (u32, u32),
}
