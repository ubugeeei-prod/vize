//! Structural evidence from the existing original-Program expression walk.

/// Original AST variants and arena values, without a framework transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind<'a> {
    Expression,
    Element,
    Fragment,
    FragmentOpening,
    FragmentClosing,
    Opening { self_closing: bool },
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

/// Balanced events are diagnostic evidence; callers cannot mint a body owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxEdge {
    Enter,
    Leave,
    Leaf,
}
