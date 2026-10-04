//! One original child traversal for the bounded complete parser-output profile.

use vize_l0::{Span, is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_l1::{
    SurfaceChild, Token,
    markup::{NativeChildren, NativeComponent, NativeLintTagKind},
};

use super::{
    super::{attribute, header, header_rules},
    NativeTemplateLintRefusal as Refusal,
};

pub(super) fn component(component: &NativeComponent<'_>) -> Result<(), Refusal> {
    let carrier = component.carrier();
    if let Some(error) = carrier.errors.first() {
        return Err(Refusal::Recovered {
            offset: error.offset,
        });
    }
    if carrier.authored.is_some() || !carrier.unsupported.is_empty() {
        return Err(Refusal::UnsupportedComponent);
    }
    let block = component.block();
    if block.start() != 0
        || !core::ptr::eq(block.source(), block.root_source())
        || !core::ptr::eq(carrier.tree.source, block.source())
    {
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

pub(super) fn children<'a>(
    component: &NativeComponent<'a>,
    children: NativeChildren<'_, 'a>,
) -> Result<(), Refusal> {
    for child in children {
        if !core::ptr::eq(child.component(), component) {
            return Err(Refusal::SourceMismatch);
        }
        vize_l0::recursion::ensure_sufficient_stack(|| {
            match child.surface() {
                SurfaceChild::Element(_) => {
                    let element = child.into_element().ok_or(Refusal::SourceMismatch)?;
                    // One strict header iteration validates all actual original
                    // attributes before using any root-only callback outcome.
                    let mut refused_attribute = None;
                    let (receipt, opening) =
                        header_rules::inspect_attributes(&element, |original, binding| {
                            let (range, supported) = match binding {
                                header::Binding::Static { name, range, .. }
                                    if name.bytes().all(|byte| {
                                        byte.is_ascii_alphanumeric()
                                            || matches!(byte, b'_' | b'-' | b':' | b'.')
                                    }) =>
                                {
                                    (range, true)
                                }
                                header::Binding::Static { range, .. }
                                | header::Binding::Bind { range, .. }
                                | header::Binding::Other { range, .. } => (range, false),
                            };
                            // Token custody is already checked by strict_binding.
                            // Reject any recovered non-whitespace header gap too.
                            if !supported || !header_gaps(original.surface()) {
                                refused_attribute.get_or_insert(range);
                            }
                        })?;
                    if let Some(span) = refused_attribute {
                        return Err(Refusal::UnsupportedAttribute { span });
                    }
                    let tag = element.surface().tag();
                    if receipt.kind() != NativeLintTagKind::Element
                        || receipt.header_is_literal()
                        || element.surface().open.is_verbatim()
                        || !is_html_tag(tag)
                        || is_svg_tag(tag)
                        || is_math_ml_tag(tag)
                        || excluded_owner(tag)
                    {
                        return Err(Refusal::UnsupportedContext { span: opening });
                    }
                    checked_token(component, &element.surface().open.lt_name)?;
                    checked_token(component, &element.surface().open.gt)?;
                    if let Some(slash) = &element.surface().open.slash {
                        checked_token(component, slash)?;
                    }
                    if let vize_l1::ElementClose::Present(close) = &element.surface().close {
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
                    self::children(component, element.children())
                }
                SurfaceChild::Text(token) => {
                    let span = checked_token(component, token)?;
                    // Unterminated native interpolation can be original Text.
                    // Text custody alone cannot certify the legacy parser result.
                    if token.text.contains("{{") || token.text.contains('<') {
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
    Ok(())
}

fn gap(token: &Token<'_>) -> bool {
    token
        .leading
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12))
}

fn header_gaps(attribute: &vize_l1::Attribute<'_>) -> bool {
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
