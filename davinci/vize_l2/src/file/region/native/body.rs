//! Only this private body consumes the original element's ordered children.

use super::{NativeVisibility, handler::ObservedHandler};
use crate::artifact::{ComponentFactory, RegionBuilder};
use crate::file::region::FileRegion;
use crate::file::{TemplateBody, TemplateChildRegion};
use crate::lang::js::file::native::NativeTemplateIssueKind as Kind;
use crate::op::Namespace;
use core::ops::DerefMut;
use vize_l0::{Span, ensure_sufficient_stack, id::NodeId};
use vize_l1::{
    ElementClose, SurfaceChild,
    markup::{NativeChild, NativeElement, NativeTemplateComponent},
};

pub(super) mod header;

pub(super) fn construct<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    ensure_sufficient_stack(|| construct_guarded(selected, child, region))
}

fn construct_guarded<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    child: NativeChild<'_, 'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let block = child.component().block();
    let (token, comment) = match child.surface() {
        SurfaceChild::Text(token) => (token, false),
        SurfaceChild::Comment(token) => (token, true),
        SurfaceChild::Element(_) => {
            return element(
                selected,
                child.into_element().ok_or(Kind::InvalidEvent)?,
                region,
            );
        }
        _ => return Err(Kind::UnsupportedChild),
    };
    let span = block.span_of(token.text).ok_or(Kind::InvalidEvent)?;
    if comment {
        let body = token
            .text
            .strip_prefix("<!--")
            .and_then(|text| text.strip_suffix("-->"))
            .ok_or(Kind::UnsupportedChild)?;
        return region.comment(body, span).map_err(Kind::Artifact);
    }
    // Preserve the root refusal: decoding and whitespace legalization require
    // their genuine lower providers, including inside an original Element.
    if token.text.contains('&')
        || token
            .text
            .bytes()
            .any(|byte| matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '))
    {
        return Err(Kind::UnsupportedChild);
    }
    region.text(token.text, span).map_err(Kind::Artifact)
}

fn element<'a: 'b, 'b, R>(
    selected: &NativeTemplateComponent<'a>,
    original: NativeElement<'_, 'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let surface = original.surface();
    let tag = surface.tag();
    // No caller tag, namespace, attribute list or body enters this route.
    // Special Vue owners and non-HTML modes need separate legalization.
    if !vize_l0::is_html_tag(tag)
        || matches!(
            tag,
            "template" | "script" | "style" | "pre" | "textarea" | "title"
        )
        || surface.open.is_verbatim()
        || surface.open.lt_name.is_missing()
        || surface.open.gt.is_missing()
        || surface
            .open
            .slash
            .as_ref()
            .is_some_and(|slash| slash.is_missing())
    {
        return Err(Kind::UnsupportedChild);
    }
    let block = original.component().block();
    let opening = block
        .span_of(surface.open.lt_name.text)
        .ok_or(Kind::InvalidEvent)?;
    let ending = match &surface.close {
        ElementClose::Present(close)
            if !close.lt_slash_name.is_missing() && !close.gt.is_missing() =>
        {
            block.span_of(close.gt.text)
        }
        ElementClose::NotExpected if !surface.open.gt.is_missing() => {
            block.span_of(surface.open.gt.text)
        }
        _ => return Err(Kind::UnsupportedChild),
    }
    .ok_or(Kind::InvalidEvent)?;
    let span = Span::new(opening.start, ending.end);
    let mut header = header::construct(&original, selected, region)?;
    if let Some(observed) = header.for_head.take() {
        return super::for_body::construct(original, selected, span, header, observed, region);
    }
    let header = header.resolve_handlers(region)?;
    element_ready(original, selected, span, header, region)
}

pub(super) fn element_ready<'a: 'b, 'b, R>(
    original: NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    span: Span,
    header: header::ReadyHeader<'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<NodeId, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let tag = original.surface().tag();
    let header::ReadyHeader {
        attributes,
        handlers,
    } = header;
    let mut result = Err(Kind::IncompleteChildren);
    let node = region
        .element_body(
            tag,
            Namespace::Html,
            attributes,
            span,
            OriginalBody {
                original,
                selected,
                handlers,
                result: &mut result,
            },
        )
        .map_err(Kind::Artifact)?;
    // A callback's normal return can contain a typed refusal. Never advance
    // the original root cursor or confer native completion on that prefix.
    result?;
    Ok(node)
}

struct OriginalBody<'selected, 'owner, 'result, 'a> {
    original: NativeElement<'owner, 'a>,
    selected: &'selected NativeTemplateComponent<'a>,
    handlers: alloc::vec::Vec<ObservedHandler<'a>>,
    result: &'result mut Result<(), Kind>,
}

impl<'a> TemplateBody<'a, NativeVisibility> for OriginalBody<'_, '_, '_, 'a> {
    fn run<'r, 'b>(self, region: &mut TemplateChildRegion<'r, 'b, 'a, NativeVisibility>, _: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        // Only a complete original header supplies these private prepared rows.
        // All attached nodes precede the same original parent's first child.
        for handler in self.handlers {
            let prepared = match region.inner.prepared_handler(handler) {
                Ok(prepared) => prepared,
                Err(kind) => {
                    *self.result = Err(kind);
                    return;
                }
            };
            if let Err(kind) = region.inner.attach_handler(prepared) {
                *self.result = Err(kind);
                return;
            }
        }
        for child in self.original.children() {
            if let Err(kind) = construct(self.selected, child, &mut region.inner) {
                *self.result = Err(kind);
                return;
            }
        }
        *self.result = Ok(());
    }
}
