use super::pattern::Directive;
use super::{Context, NativeHoleKind};
use vize_l0::Span;
use vize_l1::dialect::vue3::VueDirectives;
use vize_l1::embed::prepare_attribute_value;
use vize_l1::markup::directive::DirectiveSyntax;
use vize_l1::{Attribute as SurfaceAttribute, Element, ElementClose};
use vize_l2::artifact::RegionBuilder;
use vize_l2::op::{Attribute, Namespace};

enum PreparedAttribute<'a> {
    Static(Attribute<'a>),
    Directive(Directive),
}

impl<'a> Context<'a> {
    pub(super) fn element(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        element: &Element<'a>,
        parent: Namespace,
    ) {
        let start = self.token_span(&element.open.lt_name).start;
        let end = match &element.close {
            ElementClose::Present(close) if !close.gt.is_missing() => {
                Some(self.token_span(&close.gt).end)
            }
            ElementClose::NotExpected if !element.open.gt.is_missing() => {
                Some(self.token_span(&element.open.gt).end)
            }
            _ => None,
        };
        let span = Span::new(
            start,
            end.unwrap_or(self.token_span(&element.open.lt_name).end),
        );
        if end.is_none() {
            // An unavailable owner extent is not guessed or rescanned.
            self.hole(
                region,
                NativeHoleKind::MissingMarkup,
                self.token_span(&element.open.lt_name),
            );
        }
        if element.open.is_verbatim() {
            // Consume the real L1 policy before any ignored head/value work.
            self.hole(region, NativeHoleKind::PreCarrier, span);
            return;
        }
        let mut attributes = vize_l0::Vec::new_in(&self.allocator);
        let mut directives = vize_l0::Vec::new_in(&self.allocator);
        for attribute in &element.open.attrs {
            match self.attribute(region, attribute) {
                Some(PreparedAttribute::Static(attribute)) => attributes.push(attribute),
                Some(PreparedAttribute::Directive(directive)) => {
                    directives.push(directive);
                }
                None => {}
            }
        }
        if end.is_none() {
            self.children(region, &element.children, parent);
            return;
        }
        let tag = element.tag();
        if matches!(
            tag,
            "template" | "slot" | "component" | "script" | "style" | "annotation-xml"
        ) {
            self.hole(
                region,
                NativeHoleKind::UnsupportedTag,
                self.token_span(&element.open.lt_name),
            );
            self.children(region, &element.children, parent);
            return;
        }
        let namespace = match (parent, tag) {
            (_, "svg") => Namespace::Svg,
            (_, "math") => Namespace::MathMl,
            _ => parent,
        };
        let children_ns = match (namespace, tag) {
            (Namespace::Svg, "foreignObject" | "desc" | "title") => Namespace::Html,
            (Namespace::MathMl, "mi" | "mo" | "mn" | "ms" | "mtext") => Namespace::Html,
            _ => namespace,
        };
        let native =
            vize_l0::is_html_tag(tag) || vize_l0::is_svg_tag(tag) || vize_l0::is_math_ml_tag(tag);
        let children = |region: &mut RegionBuilder<'_, 'a>, node| {
            self.record(
                region,
                if native {
                    "native.element"
                } else {
                    "native.component"
                },
                Some(node),
                span,
                if native { "ui.element" } else { "ui.component" },
            );
            for directive in directives {
                self.directive(region, directive);
            }
            self.children(region, &element.children, children_ns);
        };
        let result = if native {
            region.element(tag, namespace, attributes, span, children)
        } else {
            region.component(tag, attributes, span, children)
        };
        if let Err(error) = result {
            self.hole(region, NativeHoleKind::Construction(error), span);
        }
    }

    fn attribute(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        attribute: &SurfaceAttribute<'a>,
    ) -> Option<PreparedAttribute<'a>> {
        let name_span = self.token_span(&attribute.name);
        let span = Span::new(
            name_span.start,
            attribute.value.as_ref().map_or_else(
                || {
                    attribute
                        .eq
                        .as_ref()
                        .map_or(name_span.end, |eq| self.token_span(eq).end)
                },
                |value| {
                    value.close_quote.as_ref().map_or_else(
                        || self.token_span(&value.content).end,
                        |quote| self.token_span(quote).end,
                    )
                },
            ),
        );
        match VueDirectives.decompose(attribute.name.text, name_span.start) {
            Ok(None) => {}
            Ok(Some(head)) => {
                return Some(PreparedAttribute::Directive(Directive {
                    head,
                    name_span,
                    span,
                    value: attribute
                        .value
                        .as_ref()
                        .map(|value| self.token_span(&value.content)),
                    missing: attribute.value.as_ref().is_some_and(|value| {
                        value.content.is_missing()
                            || value
                                .close_quote
                                .as_ref()
                                .is_some_and(|quote| quote.is_missing())
                    }),
                }));
            }
            Err(_) => {
                self.hole(region, NativeHoleKind::DirectiveSyntax, span);
                return None;
            }
        }
        let value = if let Some(value) = &attribute.value {
            if value.content.is_missing()
                || value
                    .close_quote
                    .as_ref()
                    .is_some_and(|quote| quote.is_missing())
            {
                self.hole(region, NativeHoleKind::MissingMarkup, span);
                return None;
            }
            match prepare_attribute_value(
                self.allocator,
                self.block.root_source(),
                self.token_span(&value.content),
            ) {
                Ok(source) => Some(source.text()),
                Err(error) => {
                    self.hole(region, NativeHoleKind::Source(error), span);
                    return None;
                }
            }
        } else {
            None
        };
        Some(PreparedAttribute::Static(Attribute {
            name: attribute.name.text,
            value,
            span,
        }))
    }
}

#[cfg(test)]
mod tests;
