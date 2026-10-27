//! Component value provenance and prop construction.

use vize_atelier_core::SimpleExpressionNode;
use vize_carton::{Box, Vec};

use super::EventModifiers;

/// The origin of a component value. Compiler-owned model artifacts carry
/// their source operand separately from authored JavaScript expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropValueKind {
    Expression,
    /// The original model target, emitted as an assignment callback.
    ModelUpdate,
    /// A compiler-owned object of escaped modifier names and true values.
    ModelModifiers,
    /// `v-text` on a component: its `textContent` prop renders the value as
    /// interpolated text.
    DisplayString,
    /// An opaque runtime slots object carried outside the authored slot IR.
    /// Codegen consumes this marker instead of emitting it as a component prop.
    ForwardedSlots,
}

/// IR prop
#[derive(Debug)]
pub struct IRProp<'a> {
    pub key: Box<'a, SimpleExpressionNode<'a>>,
    pub values: Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
    pub is_component: bool,
    pub value_kind: PropValueKind,
    /// Parsed component listener guards and options, shared with native events.
    pub handler_modifiers: Option<&'a EventModifiers<'a>>,
}

impl<'a> IRProp<'a> {
    pub const fn new(
        key: Box<'a, SimpleExpressionNode<'a>>,
        values: Vec<'a, Box<'a, SimpleExpressionNode<'a>>>,
        is_component: bool,
    ) -> Self {
        Self {
            key,
            values,
            is_component,
            value_kind: PropValueKind::Expression,
            handler_modifiers: None,
        }
    }

    pub const fn with_value_kind(mut self, kind: PropValueKind) -> Self {
        self.value_kind = kind;
        self
    }
}
