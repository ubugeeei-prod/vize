//! Supported model shape selected from the already decomposed original head.

use crate::markup::{DirectiveName, DirectivePrefix, NativeAttributeOperandError};
use vize_l0::{SourceBlock, Span};

pub(crate) struct ModelParts {
    pub(crate) trim: Option<Span>,
}

pub(crate) fn model_parts(
    parts: Option<DirectiveName>,
    block: SourceBlock<'_>,
) -> Result<Option<ModelParts>, NativeAttributeOperandError> {
    let Some(head) = parts else {
        return Ok(None);
    };
    if head.prefix != DirectivePrefix::Full || head.name.slice(block.root_source()) != "model" {
        return Ok(None);
    }
    if head.arg.is_some() {
        return Err(NativeAttributeOperandError::UnsupportedDirective);
    }
    let trim = if head.name.end == block.end() && head.modifiers.start == head.modifiers.end {
        None
    } else if head.name.end == head.modifiers.start
        && head.modifiers.end == block.end()
        && head.modifiers.slice(block.root_source()) == ".trim"
    {
        Some(Span::new(head.modifiers.start + 1, head.modifiers.end))
    } else {
        return Err(NativeAttributeOperandError::UnsupportedDirective);
    };
    Ok(Some(ModelParts { trim }))
}
