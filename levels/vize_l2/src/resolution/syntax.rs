//! Structural evidence from the existing original-Program expression walk.

/// Original AST variants and arena values, without a framework transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind<'a> {
    Expression,
    IdentifierExpression(&'a str),
    /// The original numeric literal's IEEE-754 bits, not reparsed source text.
    NumberExpression(u64),
    StringExpression(&'a str),
    BooleanExpression(bool),
    NullExpression,
    Element,
    Fragment,
    FragmentOpening,
    FragmentClosing,
    Opening {
        self_closing: bool,
    },
    Closing,
    Intrinsic(&'a str),
    Component(&'a str),
    Member,
    Property(&'a str),
    Attribute(&'a str),
    AttributeName(&'a str),
    AttributeString(&'a str),
    SpreadAttribute,
    Container,
    Empty,
    Text(&'a str),
    SpreadChild,
}

impl SyntaxKind<'_> {
    #[must_use]
    pub const fn is_expression(self) -> bool {
        matches!(
            self,
            Self::Expression
                | Self::IdentifierExpression(_)
                | Self::NumberExpression(_)
                | Self::StringExpression(_)
                | Self::BooleanExpression(_)
                | Self::NullExpression
        )
    }
}

/// Balanced events are diagnostic evidence; callers cannot mint a body owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxEdge {
    Enter,
    Leave,
    Leaf,
}
