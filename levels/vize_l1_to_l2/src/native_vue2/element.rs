//! Static HTML owners; directives, namespaces and special carriers stay refused.

use super::{Context, Vue2HoleKind};
use vize_l0::{Span, Vec};
use vize_l1::dialect::vue2::surface::RetainedTextChildren;
use vize_l1::embed::prepare_attribute_value;
use vize_l1::markup::{Element, ElementClose};
use vize_l2::{
    artifact::RegionBuilder,
    op::{Attribute, Namespace},
};

impl<'a> Context<'_, 'a> {
    pub(super) fn element(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        element: &Element<'a>,
        children: Option<RetainedTextChildren<'_, 'a>>,
    ) {
        let start = self.span(element.open.lt_name.text).start;
        let end = match &element.close {
            ElementClose::Present(close) if !close.gt.is_missing() => self.span(close.gt.text).end,
            ElementClose::NotExpected if !element.open.gt.is_missing() => {
                self.span(element.open.gt.text).end
            }
            _ => {
                self.hole(
                    self.span(element.open.lt_name.text),
                    Vue2HoleKind::UnsupportedMarkup,
                );
                self.span(element.open.gt.text).end
            }
        };
        let span = Span::new(start, end);
        let tag = element.tag();
        if !vize_l0::is_html_tag(tag)
            || matches!(
                tag,
                "template" | "slot" | "script" | "style" | "textarea" | "title" | "pre"
            )
        {
            self.hole(span, Vue2HoleKind::UnsupportedTag);
        }
        let mut attributes = Vec::new_in(&self.allocator);
        for attribute in &element.open.attrs {
            let name = attribute.name.text;
            let name_span = self.span(name);
            let end = attribute.value.as_ref().map_or(name_span.end, |value| {
                value
                    .close_quote
                    .as_ref()
                    .map_or(self.span(value.content.text).end, |quote| {
                        self.span(quote.text).end
                    })
            });
            let attribute_span = Span::new(name_span.start, end);
            if name.starts_with([':', '@', '#', '.'])
                || name.starts_with("v-")
                || matches!(
                    name,
                    "is" | "key"
                        | "ref"
                        | "slot"
                        | "slot-scope"
                        | "scope"
                        | "style"
                        | "muted"
                        | "inline-template"
                )
            {
                self.hole(attribute_span, Vue2HoleKind::UnsupportedAttribute);
                continue;
            }
            let value = match &attribute.value {
                Some(value) if !attribute_encoding_admitted(value.content.text) => {
                    self.hole(attribute_span, Vue2HoleKind::AttributeEncoding);
                    continue;
                }
                Some(value) => match prepare_attribute_value(
                    self.allocator,
                    self.original.block().root_source(),
                    self.span(value.content.text),
                ) {
                    Ok(source) => Some(source.text()),
                    Err(_) => {
                        self.hole(attribute_span, Vue2HoleKind::Source);
                        continue;
                    }
                },
                None => None,
            };
            attributes.push(Attribute {
                name,
                value,
                span: attribute_span,
            });
        }
        let result = region.element(tag, Namespace::Html, attributes, span, |region, _| {
            if let Some(children) = children {
                self.children(region, children);
            }
        });
        self.built(result, span);
    }
}

// Vue 2's original HTML parser decodes only this attribute vocabulary by
// default. The shared HTML5 source preparation has a wider vocabulary, so it
// can be reused only after this exact intersection is proved. Newline flags,
// other numeric/named entities and undecodable ampersands remain explicit
// refusals until their original dialect-owned value contract is implemented.
fn attribute_encoding_admitted(raw: &str) -> bool {
    raw.match_indices('&').all(|(at, _)| {
        raw.get(at..).is_some_and(|tail| {
            ["&amp;", "&lt;", "&gt;", "&quot;", "&#39;"]
                .iter()
                .any(|entity| tail.starts_with(entity))
        })
    })
}
