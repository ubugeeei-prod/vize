//! Same-event original attribute selection, expression custody and quoted Doc.

use vize_l0::{Allocator, Vec};
use vize_l1::{
    ElementClose,
    markup::{NativeElement, NativeTemplateComponent},
};

use super::super::{
    directive::name_document,
    template::{Cursor, attribute_with_name, open_element_with},
};
use super::input::Input;
use super::{Doc, NativeTemplateRefusal, TemplateRefusal, expression_document};

/// These original recovery/verbatim facts only select the unchanged refusal
/// route. Positive head authority is still minted by the real L1 receiver.
pub(super) fn eligible(element: &NativeElement<'_, '_>) -> bool {
    let element = element.surface();
    !element.open.is_verbatim()
        && !element.open.lt_name.is_missing()
        && !element.open.gt.is_missing()
        && element
            .open
            .slash
            .as_ref()
            .is_none_or(|slash| !slash.is_missing())
        && match &element.close {
            ElementClose::Missing => false,
            ElementClose::Present(close) => {
                !close.lt_slash_name.is_missing() && !close.gt.is_missing()
            }
            ElementClose::Implicit | ElementClose::NotExpected => true,
        }
}

pub(super) fn open_element<'p, 'a, I: Input<'p, 'a>>(
    selected: &'p NativeTemplateComponent<'a>,
    element: &NativeElement<'p, 'a>,
    input: &mut I,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
) -> Result<(), NativeTemplateRefusal> {
    let block = selected.component().block();
    open_element_with(
        element.surface(),
        element.attributes(),
        parts,
        cursor,
        allocator,
        depth,
        |attribute, parts, cursor| {
            let offset = cursor
                .offset
                .saturating_add(attribute.surface().name.leading.len());
            let head = selected
                .observe_attribute_head(attribute)
                .map_err(|kind| NativeTemplateRefusal::AttributeHead { offset, kind })?;
            let local =
                head.name_block()
                    .start()
                    .checked_sub(block.start())
                    .ok_or(TemplateRefusal::SourceMismatch { offset })? as usize;
            if local != offset {
                return Err(TemplateRefusal::SourceMismatch { offset }.into());
            }
            let name = name_document(head.name_block(), head.directive(), allocator)
                .map_err(|refusal| local_name_refusal(refusal, block.start(), offset))?;
            attribute_with_name(
                head.attribute().surface(),
                name,
                parts,
                cursor,
                allocator,
                |content, offset| {
                    if head.directive().is_none() {
                        return Ok(None);
                    }
                    let span = block
                        .span_of(content.text)
                        .ok_or(TemplateRefusal::SourceMismatch { offset })?;
                    if head.condition_kind().is_none() {
                        return Err(NativeTemplateRefusal::DirectiveValue { span });
                    }
                    let (index, operand) = input.attribute(&head, offset)?;
                    // Park the actual expression even when its quote frame refuses.
                    if head
                        .attribute()
                        .surface()
                        .value
                        .as_ref()
                        .is_none_or(|value| value.open_quote.is_none())
                    {
                        return Err(NativeTemplateRefusal::UnquotedConditionalValue { span });
                    }
                    let view = operand
                        .admitted_for(selected, head.attribute().reborrow())
                        .ok_or(NativeTemplateRefusal::AttributeRejected {
                            span,
                            index,
                            hole: operand.syntax().hole(),
                        })?;
                    let expression =
                        expression_document(view.operand().syntax(), block, allocator).map_err(
                            |refusal| NativeTemplateRefusal::AttributeExpression { span, refusal },
                        )?;
                    Ok(Some(expression.into_parts().1))
                },
            )
        },
    )
}

/// Only the checked name Doc uses file-absolute offsets. Template layout errors
/// remain selected-block local; embedded expression/source coordinates stay original.
fn local_name_refusal(
    refusal: TemplateRefusal,
    start: u32,
    offset: usize,
) -> NativeTemplateRefusal {
    let local = |value: usize| value.checked_sub(start as usize);
    match refusal {
        TemplateRefusal::Unsupported {
            offset: value,
            syntax,
        } => local(value).map(|offset| TemplateRefusal::Unsupported { offset, syntax }),
        TemplateRefusal::SourceMismatch { offset: value } => {
            local(value).map(|offset| TemplateRefusal::SourceMismatch { offset })
        }
        TemplateRefusal::Recovered { offset: value } => {
            local(value).map(|offset| TemplateRefusal::Recovered { offset })
        }
    }
    .unwrap_or(TemplateRefusal::SourceMismatch { offset })
    .into()
}
