//! One original child traversal for the bounded complete parser-output profile.

use vize_l0::{SmallVec, Span, is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_l1::markup::{NativeChildren, NativeComponent, NativeLintTagKind, SurfaceChild, Token};

use super::{
    attribute, header, header_rules,
    template::{
        NativeFirstOptionData, NativeTemplateAttribute, NativeTemplateAttributeProfile,
        NativeTemplateElement, NativeTemplateLintRefusal as Refusal, NativeTemplateOwnerProfile,
        NativeTemplateTextProfile,
    },
};

mod select;
mod textarea;

pub(in crate::native) fn component(component: &NativeComponent<'_>) -> Result<(), Refusal> {
    carrier(component, 0)?;
    let block = component.block();
    if block.start() != 0 || !core::ptr::eq(block.source(), block.root_source()) {
        return Err(Refusal::SourceMismatch);
    }
    Ok(())
}

/// Only the authentic selected constructor can supply this origin. The SFC
/// host additionally owns and validates its exact original Descriptor frames.
pub(in crate::native) fn selected_component(
    selected: &vize_l1::markup::NativeTemplateComponent<'_>,
) -> Result<(), Refusal> {
    carrier(selected.component(), selected.component().block().start())
}

fn carrier(component: &NativeComponent<'_>, base: u32) -> Result<(), Refusal> {
    let carrier = component.carrier();
    if let Some(error) = carrier.errors.first() {
        let offset = base
            .checked_add(error.offset)
            .ok_or(Refusal::SourceMismatch)?;
        return Err(Refusal::Recovered { offset });
    }
    if carrier.authored.is_some() || !carrier.unsupported.is_empty() {
        return Err(Refusal::UnsupportedComponent);
    }
    if !core::ptr::eq(carrier.tree.source, component.block().source()) {
        return Err(Refusal::SourceMismatch);
    }
    Ok(())
}

fn checked_token(component: &NativeComponent<'_>, token: &Token<'_>) -> Result<Span, Refusal> {
    attribute::token(component.block(), token)?;
    let span = attribute::span(component.block(), token.text)?;
    if !token
        .leading
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12))
    {
        return Err(Refusal::UnsupportedText { span });
    }
    Ok(span)
}

pub(in crate::native) fn children<'o, 'a>(
    component: &'o NativeComponent<'a>,
    children: NativeChildren<'o, 'a>,
    profile: NativeTemplateAttributeProfile,
    visit_element: &mut impl FnMut(&NativeTemplateElement<'o, 'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    // Preserve the existing bare-template/setup hosts' exact checked-text
    // behavior. Only the distinct template-only SFC host freezes and supplies
    // a new configured callback intersection below.
    children_with_text_profile(
        component,
        children,
        profile,
        NativeTemplateTextProfile::LiteralText,
        visit_element,
    )
}

pub(in crate::native) fn children_with_text_profile<'o, 'a>(
    component: &'o NativeComponent<'a>,
    children: NativeChildren<'o, 'a>,
    profile: NativeTemplateAttributeProfile,
    text_profile: NativeTemplateTextProfile,
    visit_element: &mut impl FnMut(&NativeTemplateElement<'o, 'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    children_with_owner_profile(
        component,
        children,
        profile,
        text_profile,
        NativeTemplateOwnerProfile::Strict,
        visit_element,
    )
}

pub(in crate::native) fn children_with_owner_profile<'o, 'a>(
    component: &'o NativeComponent<'a>,
    children: NativeChildren<'o, 'a>,
    profile: NativeTemplateAttributeProfile,
    text_profile: NativeTemplateTextProfile,
    owner_profile: NativeTemplateOwnerProfile,
    visit_element: &mut impl FnMut(&NativeTemplateElement<'o, 'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    walk(
        component,
        children,
        profile,
        text_profile,
        owner_profile,
        select::Parent::Ordinary,
        visit_element,
    )
    .map(|_| ())
}

fn walk<'o, 'a>(
    component: &'o NativeComponent<'a>,
    children: NativeChildren<'o, 'a>,
    profile: NativeTemplateAttributeProfile,
    text_profile: NativeTemplateTextProfile,
    owner_profile: NativeTemplateOwnerProfile,
    parent: select::Parent,
    visit_element: &mut impl FnMut(&NativeTemplateElement<'o, 'a>) -> Result<(), Refusal>,
) -> Result<Option<NativeFirstOptionData<'o, 'a>>, Refusal> {
    let mut first_option = None;
    for child in children {
        if !core::ptr::eq(child.component(), component) {
            return Err(Refusal::SourceMismatch);
        }
        vize_l0::recursion::ensure_sufficient_stack(|| {
            match child.surface() {
                SurfaceChild::Element(_) => {
                    let element = child.into_element().ok_or(Refusal::SourceMismatch)?;
                    // The same original header iteration checks and retains the
                    // whole admitted header before dispatching any callbacks.
                    let mut refused_attribute = None;
                    let mut attributes: SmallVec<[NativeTemplateAttribute<'o, 'a>; 8]> =
                        SmallVec::new();
                    let visit = |checked: header::CheckedAttribute<'o, 'a>, binding| {
                        if !header_gaps(checked.original().surface()) {
                            refused_attribute.get_or_insert(checked.range());
                        } else {
                            match NativeTemplateAttribute::from_checked(checked, binding, profile) {
                                Ok(attribute) => attributes.push(attribute),
                                Err(range) => {
                                    refused_attribute.get_or_insert(range);
                                }
                            }
                        }
                        Ok(())
                    };
                    let (receipt, opening) = if profile == NativeTemplateAttributeProfile::Events {
                        header_rules::inspect_events(&element, visit)?
                    } else if profile == NativeTemplateAttributeProfile::Bindings {
                        header_rules::inspect_checked::<true>(&element, visit)?
                    } else {
                        header_rules::inspect_checked::<false>(&element, visit)?
                    };
                    if let Some(span) = refused_attribute {
                        return Err(Refusal::UnsupportedAttribute { span });
                    }
                    let tag = element.surface().tag();
                    let child_parent = if textarea::check(
                        owner_profile,
                        parent,
                        &element,
                        &attributes,
                        opening,
                    )? {
                        select::Parent::Textarea
                    } else {
                        select::check(owner_profile, parent, &element, &attributes, opening)?
                    };
                    if receipt.kind() != NativeLintTagKind::Element
                        || receipt.header_is_literal()
                        || element.surface().open.is_verbatim()
                        || !is_html_tag(tag)
                        || is_svg_tag(tag)
                        || is_math_ml_tag(tag)
                        || (excluded_owner(tag)
                            && !matches!(
                                child_parent,
                                select::Parent::Option | select::Parent::Textarea
                            ))
                    {
                        return Err(Refusal::UnsupportedContext { span: opening });
                    }
                    checked_token(component, &element.surface().open.lt_name)?;
                    checked_token(component, &element.surface().open.gt)?;
                    if let Some(slash) = &element.surface().open.slash {
                        checked_token(component, slash)?;
                    }
                    if let vize_l1::markup::ElementClose::Present(close) = &element.surface().close
                    {
                        checked_token(component, &close.lt_slash_name)?;
                        checked_token(component, &close.gt)?;
                        if close
                            .lt_slash_name
                            .text
                            .strip_prefix("</")
                            .is_none_or(|name| !name.eq_ignore_ascii_case(tag))
                        {
                            return Err(Refusal::UnsupportedContext { span: opening });
                        }
                    }
                    let children = element.children();
                    let mut view = NativeTemplateElement::new(element, attributes);
                    if parent == select::Parent::Select && first_option.is_none() {
                        first_option = Some(NativeFirstOptionData::from_checked(&view, opening)?);
                    }
                    if matches!(
                        child_parent,
                        select::Parent::Select | select::Parent::Textarea
                    ) {
                        // Only this bounded owner is postorder: its complete
                        // original children must pass before proof/reporting.
                        let first = walk(
                            component,
                            children,
                            profile,
                            text_profile,
                            owner_profile,
                            child_parent,
                            visit_element,
                        )?;
                        if child_parent == select::Parent::Select {
                            view.admit_closed_select(first)?;
                        } else {
                            view.admit_closed_textarea();
                        }
                        return visit_element(&view);
                    }
                    visit_element(&view)?;
                    // Retained header storage is bounded by the largest header,
                    // including possible SmallVec spill, not by ancestor depth.
                    drop(view);
                    walk(
                        component,
                        children,
                        profile,
                        text_profile,
                        owner_profile,
                        child_parent,
                        visit_element,
                    )
                    .map(|_| ())
                }
                SurfaceChild::Text(token) => {
                    let span = checked_token(component, token)?;
                    // Unterminated native interpolation can be original Text.
                    // Text custody alone cannot certify the legacy parser result.
                    if token.text.contains("{{")
                        || token.text.contains('<')
                        || (parent == select::Parent::Textarea && token.text.contains('&'))
                        || ((text_profile == NativeTemplateTextProfile::WhitespaceOnly
                            || parent == select::Parent::Select)
                            && !token
                                .text
                                .bytes()
                                .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12)))
                    {
                        return Err(Refusal::UnsupportedText { span });
                    }
                    Ok(())
                }
                SurfaceChild::Comment(token) => Err(Refusal::Comment {
                    span: checked_token(component, token)?,
                }),
                SurfaceChild::Interpolation(interpolation) => {
                    let open = checked_token(component, &interpolation.open)?;
                    let close = checked_token(component, &interpolation.close)?;
                    Err(Refusal::Interpolation {
                        span: Span::new(open.start, close.end),
                    })
                }
                SurfaceChild::Cdata(token)
                | SurfaceChild::ProcessingInstruction(token)
                | SurfaceChild::Unexpected(token) => Err(Refusal::UnexpectedChild {
                    span: checked_token(component, token)?,
                }),
            }
        })?;
    }
    Ok(first_option)
}

fn gap(token: &Token<'_>) -> bool {
    token
        .leading
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12))
}

fn header_gaps(attribute: &vize_l1::markup::Attribute<'_>) -> bool {
    gap(&attribute.name)
        && attribute.eq.as_ref().is_none_or(gap)
        && attribute.value.as_ref().is_none_or(|value| {
            value.open_quote.as_ref().is_none_or(gap)
                && gap(&value.content)
                && value.close_quote.as_ref().is_none_or(gap)
        })
}

// Conservative initial source grammar boundaries, including the own opener:
// inherited receipts alone cannot exclude these recovery/raw owners. This is
// neither a DOM claim nor a second recovery/parser implementation.
fn excluded_owner(tag: &str) -> bool {
    matches!(
        tag,
        "table"
            | "template"
            | "script"
            | "style"
            | "title"
            | "textarea"
            | "iframe"
            | "noscript"
            | "p"
            | "form"
            | "a"
            | "button"
            | "li"
            | "dt"
            | "dd"
            | "option"
            | "optgroup"
            | "b"
            | "big"
            | "code"
            | "em"
            | "font"
            | "i"
            | "nobr"
            | "s"
            | "small"
            | "strike"
            | "strong"
            | "tt"
            | "u"
    )
}
