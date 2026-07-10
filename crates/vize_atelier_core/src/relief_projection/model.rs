use vize_relief::{ElementType, ExpressionNode, Position, SimpleExpressionNode, SourceLocation};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub(crate) struct ReliefSpan {
    pub start: Position,
    pub end: Position,
}

impl ReliefSpan {
    pub const fn from_location(location: &SourceLocation) -> Self {
        Self {
            start: location.start,
            end: location.end,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ReliefExpressionRef<'a> {
    Source(&'a str),
    Node(&'a ExpressionNode<'a>),
}

impl<'a> ReliefExpressionRef<'a> {
    pub const fn from_expression(expression: &'a ExpressionNode<'a>) -> Self {
        Self::Node(expression)
    }

    pub fn text(self) -> &'a str {
        match self {
            Self::Source(text) => text,
            Self::Node(ExpressionNode::Simple(expression)) => expression.content.as_str(),
            Self::Node(ExpressionNode::Compound(expression)) => expression.loc.source.as_str(),
        }
    }

    pub const fn node(self) -> Option<&'a ExpressionNode<'a>> {
        match self {
            Self::Source(_) => None,
            Self::Node(expression) => Some(expression),
        }
    }

    pub fn is_simple(self, text: &str) -> bool {
        matches!(self.node(), Some(ExpressionNode::Simple(expression)) if expression.content == text)
    }
}

impl PartialEq for ReliefExpressionRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text()
    }
}

impl Eq for ReliefExpressionRef<'_> {}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) enum ReliefElementKind {
    Element,
    Component,
    SlotOutlet,
    Template,
}

impl ReliefElementKind {
    pub const fn is_component(self) -> bool {
        matches!(self, Self::Component)
    }

    pub const fn is_slot_outlet(self) -> bool {
        matches!(self, Self::SlotOutlet)
    }

    pub const fn is_template(self) -> bool {
        matches!(self, Self::Template)
    }
}

impl From<ElementType> for ReliefElementKind {
    fn from(kind: ElementType) -> Self {
        match kind {
            ElementType::Element => Self::Element,
            ElementType::Component => Self::Component,
            ElementType::Slot => Self::SlotOutlet,
            ElementType::Template => Self::Template,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ReliefModifiers<'a> {
    modifiers: &'a [SimpleExpressionNode<'a>],
}

impl<'a> ReliefModifiers<'a> {
    pub const fn new(modifiers: &'a [SimpleExpressionNode<'a>]) -> Self {
        Self { modifiers }
    }

    pub const fn is_empty(self) -> bool {
        self.modifiers.is_empty()
    }

    pub const fn len(self) -> usize {
        self.modifiers.len()
    }

    pub fn names(self) -> impl Iterator<Item = &'a str> {
        self.modifiers
            .iter()
            .map(|modifier| modifier.content.as_str())
    }

    pub fn contains(self, name: &str) -> bool {
        self.names().any(|candidate| candidate == name)
    }
}

impl PartialEq for ReliefModifiers<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.names().zip(other.names()).all(|(a, b)| a == b)
    }
}

impl Eq for ReliefModifiers<'_> {}
