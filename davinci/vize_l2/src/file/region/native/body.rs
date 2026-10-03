//! Only this private body consumes the original element's ordered children.

use crate::artifact::{ComponentBody, ComponentFactory};
use crate::lang::js::file::native::NativeTemplateIssueKind as Kind;
use crate::op::Namespace;
use vize_l0::{Span, ensure_sufficient_stack, id::NodeId};
use vize_l1::{
    ElementClose, SurfaceChild,
    markup::{NativeChild, NativeElement},
};

pub(super) fn construct<'a, R: ComponentFactory<'a>>(
    child: NativeChild<'_, 'a>,
    region: &mut R,
) -> Result<NodeId, Kind> {
    ensure_sufficient_stack(|| construct_guarded(child, region))
}

fn construct_guarded<'a, R: ComponentFactory<'a>>(
    child: NativeChild<'_, 'a>,
    region: &mut R,
) -> Result<NodeId, Kind> {
    let block = child.component().block();
    let (token, comment) = match child.surface() {
        SurfaceChild::Text(token) => (token, false),
        SurfaceChild::Comment(token) => (token, true),
        SurfaceChild::Element(_) => {
            return element(child.into_element().ok_or(Kind::InvalidEvent)?, region);
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

fn element<'a, R: ComponentFactory<'a>>(
    original: NativeElement<'_, 'a>,
    region: &mut R,
) -> Result<NodeId, Kind> {
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
        || !surface.open.attrs.is_empty()
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
    let attributes = vize_l0::Vec::new_in(original.component().allocator());
    let mut result = Err(Kind::IncompleteChildren);
    let node = region
        .element(
            tag,
            Namespace::Html,
            attributes,
            span,
            OriginalBody {
                original,
                result: &mut result,
            },
        )
        .map_err(Kind::Artifact)?;
    // A callback's normal return can contain a typed refusal. Never advance
    // the original root cursor or confer native completion on that prefix.
    result?;
    Ok(node)
}

struct OriginalBody<'owner, 'result, 'a> {
    original: NativeElement<'owner, 'a>,
    result: &'result mut Result<(), Kind>,
}

impl<'a> ComponentBody<'a> for OriginalBody<'_, '_, 'a> {
    fn run<R: ComponentFactory<'a>>(self, region: &mut R, _: NodeId) {
        // This iterator is derived from the actual original parent. It is
        // private: callers cannot substitute, omit or replay nested events.
        for child in self.original.children() {
            if let Err(kind) = construct(child, region) {
                *self.result = Err(kind);
                return;
            }
        }
        *self.result = Ok(());
    }
}
