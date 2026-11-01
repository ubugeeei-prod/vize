//! Closed select grammar offered only by the actual callback intersection.

use super::{NativeTemplateAttribute, NativeTemplateOwnerProfile, Refusal};
use crate::native::template::NativeTemplateAttributeKind;
use vize_l0::Span;
use vize_l1::markup::{ElementClose, NativeElement};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Parent {
    Ordinary,
    Select,
    Option,
    Textarea,
}

pub(super) fn check(
    profile: NativeTemplateOwnerProfile,
    parent: Parent,
    element: &NativeElement<'_, '_>,
    attributes: &[NativeTemplateAttribute<'_, '_>],
    opening: Span,
) -> Result<Parent, Refusal> {
    let tag = element.surface().tag();
    let refuse = || Refusal::UnsupportedContext { span: opening };
    if matches!(parent, Parent::Option | Parent::Textarea)
        || (parent == Parent::Select && tag != "option")
    {
        return Err(refuse());
    }
    if !profile.allows_select() {
        return Ok(Parent::Ordinary);
    }
    if tag != "select" && tag != "option" {
        return Ok(Parent::Ordinary);
    }
    if (tag == "option" && parent != Parent::Select)
        || !matches!(element.surface().close, ElementClose::Present(_))
    {
        return Err(refuse());
    }
    for attribute in attributes {
        // A runtime owner or dynamic name cannot inherit this grammar proof.
        // Values are never decoded/evaluated to manufacture an owner identity.
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
    Ok(if tag == "select" {
        Parent::Select
    } else {
        Parent::Option
    })
}
