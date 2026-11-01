//! Bare model shape selected from the already decomposed original head.

use crate::markup::{DirectiveName, DirectivePrefix, NativeAttributeOperandError};
use vize_l0::SourceBlock;

pub(crate) fn model_parts(
    parts: Option<DirectiveName>,
    block: SourceBlock<'_>,
) -> Result<bool, NativeAttributeOperandError> {
    let Some(head) = parts else {
        return Ok(false);
    };
    if head.prefix != DirectivePrefix::Full || head.name.slice(block.root_source()) != "model" {
        return Ok(false);
    }
    if head.arg.is_some()
        || head.name.end != block.end()
        || head.modifiers.start != head.modifiers.end
    {
        return Err(NativeAttributeOperandError::UnsupportedDirective);
    }
    Ok(true)
}
