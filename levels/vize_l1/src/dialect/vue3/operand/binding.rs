//! Static binding shape selected from the already decomposed original head.

use crate::markup::{ArgSyntax, DirectiveName, DirectivePrefix, NativeAttributeOperandError};
use vize_l0::{SourceBlock, Span};

pub(crate) fn static_binding_parts(
    parts: Option<DirectiveName>,
    block: SourceBlock<'_>,
) -> Result<Option<Span>, NativeAttributeOperandError> {
    let Some(head) = parts else {
        return Ok(None);
    };
    let expected = match head.prefix {
        DirectivePrefix::Bind => block.start().checked_add(1),
        DirectivePrefix::Full if head.name.slice(block.root_source()) == "bind" => {
            head.name.end.checked_add(1)
        }
        DirectivePrefix::Prop => return Err(NativeAttributeOperandError::UnsupportedDirective),
        _ => return Ok(None),
    };
    let Some(ArgSyntax::Static(argument)) = head.arg else {
        return Err(NativeAttributeOperandError::UnsupportedDirective);
    };
    // A closed dynamic argument followed by a tail may leave Static metadata.
    // Only the exact original prefix/argument geometry selects this family.
    if Some(argument.start) != expected
        || argument.end != block.end()
        || argument.start == argument.end
        || head.modifiers.start != head.modifiers.end
    {
        return Err(NativeAttributeOperandError::UnsupportedDirective);
    }
    Ok(Some(argument))
}
