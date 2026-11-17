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
        /// Bit-flag leaves of the returned expression, as for `Pair`.
        safe_operands: Vec<Span>,
    },
    Heritage {
        sender: Span,
        member: CompactString,
        sources: Vec<HeritageSource>,
        /// Bit-flag leaves of the initializer, as for `Pair`.
        safe_operands: Vec<Span>,
    },
    Computed {
        object: Span,
        property: Span,
    },
}

/// A class heritage type. `base_types` asks for the instance bases of the
/// class expression itself; otherwise `span` is an `implements` type, typed
/// as a type node (`implements Basket` names the interface type).
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
    /// An array or object literal argument: it takes its parameter, but
    /// typescript-eslint reports its elements and properties instead.
    pub literal: bool,
    /// Bit-flag leaves of a non-spread argument, as for `Pair`.
    pub safe_operands: Vec<Span>,
}

impl EnumCheckAsk {
    /// Every node span the check reads, in the batch's location order, and
    /// whether it is located as a type node.
    pub(in crate::linter) fn each_span(&self, add: &mut impl FnMut(Span, bool)) {
        match self {
            Self::Pair {
                sender,
                receiver,
                safe_operands,
            } => {
                add(*sender, false);
                match receiver {
                    EnumReceiver::TypeAt(span) | EnumReceiver::Contextual(span) => {
                        add(*span, false);
                    }
                }
                for operand in safe_operands {
                    add(*operand, false);
                }
            }
            Self::Return {
                function,
                returned,
                receiver,
                safe_operands,
                ..
            } => {
                add(*function, false);
                add(*returned, false);
                if let Some(receiver) = receiver {
                    add(*receiver, false);
                }
                for operand in safe_operands {
                    add(*operand, false);
                }
            }
            Self::Heritage {
                sender,
                sources,
                safe_operands,
                ..
            } => {
                add(*sender, false);
                for source in sources {
                    add(source.span, !source.base_types);
                }
                for operand in safe_operands {
                    add(*operand, false);
                }
            }
            Self::Computed { object, property } => {
                add(*object, false);
                add(*property, false);
            }
        }
    }
}

impl EnumCallAsk {
    /// Every node span the call check reads, in the batch's location order.
    pub(in crate::linter) fn each_span(&self, add: &mut impl FnMut(Span)) {
        add(self.call);
        for argument in self.arguments.iter().filter(|argument| !argument.literal) {
            add(argument.node);
            add(argument.expression);
            for operand in &argument.safe_operands {
                add(*operand);
            }
        }
    }
}
