//! Supported model shape selected from the already decomposed original head.

use crate::markup::{DirectiveName, DirectivePrefix, NativeAttributeOperandError};
use vize_l0::{SourceBlock, Span};

#[derive(Clone, Copy)]
pub(crate) enum ModelModifier {
    Trim(Span),
    Lazy(Span),
    Number(Span),
}

pub(crate) struct ModelParts {
    pub(crate) modifier: Option<ModelModifier>,
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
    let modifier = if head.name.end == block.end() && head.modifiers.start == head.modifiers.end {
        None
    } else if head.name.end == head.modifiers.start && head.modifiers.end == block.end() {
        let span = Span::new(head.modifiers.start + 1, head.modifiers.end);
        Some(match head.modifiers.slice(block.root_source()) {
            ".trim" => ModelModifier::Trim(span),
            ".lazy" => ModelModifier::Lazy(span),
            ".number" => ModelModifier::Number(span),
            _ => return Err(NativeAttributeOperandError::UnsupportedDirective),
        })
    } else {
        return Err(NativeAttributeOperandError::UnsupportedDirective);
    };
    Ok(Some(ModelParts { modifier }))
}
