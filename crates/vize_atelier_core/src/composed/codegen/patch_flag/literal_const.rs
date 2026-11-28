// DO NOT EDIT. Generated from crates/vize_atelier_core/src/codegen/patch_flag/literal_const.rs by `node tools/commands/generate/composed-copies.ts`.
// Edit the source and run the command again.
//! Non-inline `literal-const` props are patch-stable.
//!
//! Codegen has already rewritten the identifier to `$setup.name`. Vue records
//! `CAN_SKIP_PATCH` for that binding and emits no `PROPS` / `CLASS` / `STYLE`
//! flag. The binding is not hoisted. A bare name is either inline mode or a
//! shadowed local, and stays dynamic.

use crate::composed::is_simple_identifier;
use crate::composed::options::{BindingMetadata, BindingType};
use crate::composed::{DirectiveNode, ExpressionNode};

pub(super) fn is_setup_literal_const_access(
    dir: &DirectiveNode<'_>,
    bindings: Option<&BindingMetadata>,
) -> bool {
    let Some(bindings) = bindings else {
        return false;
    };
    let Some(ExpressionNode::Simple(simple)) = &dir.exp else {
        return false;
    };
    let Some(name) = simple.content.trim().strip_prefix("$setup.") else {
        return false;
    };
    is_simple_identifier(name)
        && matches!(bindings.bindings.get(name), Some(BindingType::LiteralConst))
}
