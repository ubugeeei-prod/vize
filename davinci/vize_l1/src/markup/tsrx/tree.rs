//! TSRX markup nodes. Every span is an exact authored byte range of the
//! module source; every embedded expression, statement and binding is the
//! host parser's own AST node from the same parse.

use oxc_ast::ast::{BindingIdentifier, Expression, ForStatementLeft, FormalParameters, Statement};
use vize_l0::{Box, Span, Vec};

/// One markup value: what a region, a render output or a markup child holds.
#[derive(Debug)]
pub enum Node<'a> {
    Element(Box<'a, Element<'a>>),
    Fragment(Box<'a, Fragment<'a>>),
    /// `@{ … }`
    CodeBlock(Box<'a, Block<'a>>),
    If(Box<'a, If<'a>>),
    For(Box<'a, For<'a>>),
    Switch(Box<'a, Switch<'a>>),
    Try(Box<'a, Try<'a>>),
}

impl Node<'_> {
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Element(node) => node.span,
            Self::Fragment(node) => node.span,
            Self::CodeBlock(node) => node.span,
            Self::If(node) => node.span,
            Self::For(node) => node.span,
            Self::Switch(node) => node.span,
            Self::Try(node) => node.span,
        }
    }
}

/// A child of an element or fragment.
#[derive(Debug)]
pub enum Child<'a> {
    /// JSX text, as written (entities undecoded, layout whitespace included).
    Text(Span),
    /// A comment between children.
    Comment(Comment),
    /// `{ expression }`, or `{}` / `{/* … */}` with no expression.
    Expression(ExpressionChild<'a>),
    Node(Node<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    /// `/* … */`
    Block,
    /// `// …` up to the end of the line.
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    pub span: Span,
}

#[derive(Debug)]
pub struct ExpressionChild<'a> {
    /// From `{` through `}`.
    pub span: Span,
    pub expression: Option<Expression<'a>>,
}

#[derive(Debug)]
pub struct Element<'a> {
    /// From `<` through the closing tag's `>` (or the self-closing `/>`).
    pub span: Span,
    pub name: TagName<'a>,
    pub attributes: Vec<'a, Attribute<'a>>,
    pub children: Vec<'a, Child<'a>>,
    /// `<style>` / `<script>` body kept as raw text.
    pub raw_text: Option<Span>,
    /// The closing tag, absent when self-closing.
    pub closing: Option<Span>,
}

impl Element<'_> {
    #[must_use]
    pub fn is_self_closing(&self) -> bool {
        self.closing.is_none()
    }
}

#[derive(Debug)]
pub enum TagName<'a> {
    /// An identifier, a dashed name or a member chain such as `Foo.Bar`.
    Static(Span),
    /// `{ expression }`: an identifier, a member chain or a string literal.
    Dynamic {
        span: Span,
        expression: Expression<'a>,
    },
}

#[derive(Debug)]
pub struct Fragment<'a> {
    /// From `<>` through `</>`.
    pub span: Span,
    pub children: Vec<'a, Child<'a>>,
}

#[derive(Debug)]
pub enum Attribute<'a> {
    /// `name`, `name="text"`, `name={expression}`.
    Named {
        span: Span,
        name: Span,
        value: Option<AttributeValue<'a>>,
    },
    /// `{name}`, short for `name={name}`.
    Shorthand {
        span: Span,
        expression: Expression<'a>,
    },
    /// `{...expression}`.
    Spread {
        span: Span,
        expression: Expression<'a>,
    },
}

#[derive(Debug)]
pub enum AttributeValue<'a> {
    /// A quoted string, quotes included.
    String(Span),
    /// From `{` through `}`.
    Expression {
        span: Span,
        expression: Expression<'a>,
    },
}

/// The `{ … }` of a statement container or a directive branch: setup
/// statements first, then at most one rendered output node.
#[derive(Debug)]
pub struct Block<'a> {
    /// The braces, or `@{ … }` for a statement container.
    pub span: Span,
    pub statements: Vec<'a, Statement<'a>>,
    pub render: Option<Node<'a>>,
}

#[derive(Debug)]
pub struct If<'a> {
    pub span: Span,
    /// `@if`, each `@else if`, then the optional `@else` (without a test).
    pub branches: Vec<'a, IfBranch<'a>>,
}

#[derive(Debug)]
pub struct IfBranch<'a> {
    pub span: Span,
    pub test: Option<Expression<'a>>,
    pub body: Block<'a>,
}

#[derive(Debug)]
pub struct For<'a> {
    pub span: Span,
    pub r#await: bool,
    pub left: ForStatementLeft<'a>,
    pub right: Expression<'a>,
    pub index: Option<BindingIdentifier<'a>>,
    pub key: Option<Expression<'a>>,
    pub body: Block<'a>,
    pub empty: Option<Block<'a>>,
}

#[derive(Debug)]
pub struct Switch<'a> {
    pub span: Span,
    pub discriminant: Expression<'a>,
    pub cases: Vec<'a, SwitchCase<'a>>,
}

#[derive(Debug)]
pub struct SwitchCase<'a> {
    pub span: Span,
    /// `None` for `@default`.
    pub test: Option<Expression<'a>>,
    pub body: Block<'a>,
}

#[derive(Debug)]
pub struct Try<'a> {
    pub span: Span,
    pub body: Block<'a>,
    pub pending: Option<Block<'a>>,
    pub catch: Option<Catch<'a>>,
}

#[derive(Debug)]
pub struct Catch<'a> {
    pub span: Span,
    /// `(error, reset)`, when written.
    pub params: Option<Box<'a, FormalParameters<'a>>>,
    pub body: Block<'a>,
}
