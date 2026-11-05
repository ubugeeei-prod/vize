//! `CAN_SKIP_PATCH` for an interpolation child.
//!
//! Vue's `processExpression` records that only for an authored simple
//! identifier. Type erasure rewrites `pick<Item>` and `pick as T` down to
//! the bare name before this check runs; the span still covers the authored
//! bytes, which stay `NOT_CONSTANT`.

use crate::ExpressionNode;
use crate::is_simple_identifier;
use crate::options::{BindingMetadata, BindingType};

pub(super) fn is_constant_interpolation(
    expr: &ExpressionNode<'_>,
    bindings: Option<&BindingMetadata>,
    source: &str,
) -> bool {
    let Some(bindings) = bindings else {
        return false;
    };
    let ExpressionNode::Simple(simple) = expr else {
        return false;
    };
    // A generated node has no authored span. Fall back to its content, which
    // is the only text the check used to see.
    let covered = simple.loc.span.slice(source).trim();
    let name = if covered.is_empty() {
        simple.content.trim()
    } else {
        covered
    };
    // A non-inline rewrite (`$setup.pick`) or a `.value` read is not the
    // fast path. Erasure that leaves the identifier unchanged still matches.
    is_simple_identifier(name)
        && simple.content.trim() == name
        && matches!(
            bindings.bindings.get(name),
            Some(BindingType::LiteralConst | BindingType::SetupConst)
        )
}
