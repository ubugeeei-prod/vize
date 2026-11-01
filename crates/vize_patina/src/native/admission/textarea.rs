//! Bounded original RCDATA ownership; no decoding or raw-text emulation.

use super::{NativeTemplateAttribute, NativeTemplateOwnerProfile, Refusal, select::Parent};
use crate::native::template::NativeTemplateAttributeKind;
use vize_l0::Span;
use vize_l1::markup::{ElementClose, NativeElement};

pub(super) fn check(
    profile: NativeTemplateOwnerProfile,
    parent: Parent,
    element: &NativeElement<'_, '_>,
    attributes: &[NativeTemplateAttribute<'_, '_>],
    opening: Span,
) -> Result<bool, Refusal> {
    if element.surface().tag() != "textarea"
        || profile != NativeTemplateOwnerProfile::ClosedSelectAndTextarea
    {
        return Ok(false);
    }
    if parent != Parent::Ordinary || !matches!(element.surface().close, ElementClose::Present(_)) {
        return Err(Refusal::UnsupportedContext { span: opening });
    }
    for attribute in attributes {
        if matches!(
            attribute.kind(),
            NativeTemplateAttributeKind::Static { name: "is", .. }
                | NativeTemplateAttributeKind::Bind { name: "is", .. }
                | NativeTemplateAttributeKind::DynamicBind { .. }
        ) {
            return Err(Refusal::UnsupportedAttribute {
                span: attribute.range(),
            });
        }
    }
    Ok(true)
}
