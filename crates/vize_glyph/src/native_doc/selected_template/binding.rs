//! Same-event static binding observation, normal custody and original value Doc.

use vize_l0::{Allocator, Span};
use vize_l1::markup::{NativeStaticBindingHead, NativeTemplateComponent};

use super::input::Input;
use super::{Doc, NativeTemplateRefusal, expression_document};

pub(super) fn value<'p, 'a, I: Input<'p, 'a>>(
    selected: &'p NativeTemplateComponent<'a>,
    input: &mut I,
    binding: NativeStaticBindingHead<'_, '_, 'a>,
    span: Span,
    offset: usize,
    allocator: &'a Allocator,
) -> Result<Option<Doc<'a>>, NativeTemplateRefusal> {
    let head = binding.head();
    let (index, operand) = input.binding(binding, offset)?;
    // The actual complete expression is parked before quote/admission/Doc
    // refusal. This does not expand missing-value shorthand or legalize a target.
    if head
        .attribute()
        .surface()
        .value
        .as_ref()
        .is_none_or(|value| value.open_quote.is_none())
    {
        return Err(NativeTemplateRefusal::UnquotedBindingValue { span });
    }
    let view = operand
        .admitted_for(selected, head.attribute().reborrow())
        .ok_or(NativeTemplateRefusal::BindingRejected {
            span,
            index,
            hole: operand.syntax().hole(),
        })?;
    let expression = expression_document(
        view.operand().syntax(),
        selected.component().block(),
        allocator,
    )
    .map_err(|refusal| NativeTemplateRefusal::BindingExpression { span, refusal })?;
    Ok(Some(expression.into_parts().1))
}
