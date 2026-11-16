use super::super::Span;
use vize_l0::String as CompactString;

/// A deep enum mismatch and the receiver enum names used in its diagnostic.
#[derive(Clone)]
pub(in crate::linter) struct EnumMismatch {
    pub names: Vec<CompactString>,
}

/// A checker type used as the receiving side of a direct assignment.
pub(in crate::linter) enum EnumReceiver {
    /// `checker.getTypeAtLocation(node)`.
    TypeAt(Span),
    /// `checker.getContextualType(node)`.
    Contextual(Span),
}

/// One non-call enum-flow check.
pub(in crate::linter) enum EnumCheckAsk {
    Pair {
        sender: Span,
        receiver: EnumReceiver,
        /// Primitive leaves of a bitwise expression. When all share the
        /// receiver enum, the expression is a safe enum bit flag.
        safe_operands: Vec<Span>,
    },
    Return {
        function: Span,
        returned: Span,
        receiver: Option<Span>,
        is_async: bool,
    },
    Heritage {
        sender: Span,
        member: CompactString,
        sources: Vec<HeritageSource>,
    },
    Computed {
        object: Span,
        property: Span,
    },
}

/// A class heritage type. `base_types` asks for the instance bases of the
/// class expression itself; otherwise `span` is an `implements` clause.
pub(in crate::linter) struct HeritageSource {
    pub span: Span,
    pub base_types: bool,
}

/// One call-like expression and its arguments in source order.
pub(in crate::linter) struct EnumCallAsk {
    pub call: Span,
    pub arguments: Vec<EnumCallArgument>,
    pub tagged_template: bool,
}

pub(in crate::linter) struct EnumCallArgument {
    /// The reporting node (a spread includes its `...`).
    pub node: Span,
    /// The spread operand, otherwise the same span as `node`.
    pub expression: Span,
    pub spread: bool,
}
