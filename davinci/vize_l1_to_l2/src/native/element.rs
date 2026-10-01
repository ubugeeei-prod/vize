use super::pattern::Directive;
use super::{Context, NativeHoleKind};
use vize_l0::Span;
use vize_l1::dialect::vue3::VueDirectives;
use vize_l1::embed::prepare_attribute_value;
use vize_l1::markup::directive::DirectiveSyntax;
use vize_l1::{Attribute as SurfaceAttribute, Element};
use vize_l2::artifact::RegionBuilder;
use vize_l2::op::{Attribute, Namespace};

mod header;
pub(super) use header::{HeaderAdmission, PreparedElement, StructuralHeadMask};

enum PreparedAttribute<'a> {
    Static(Attribute<'a>),
    Directive(Directive),
}

impl<'a> Context<'a> {
    pub(super) fn element(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        element: &Element<'a>,
        parent: (Namespace, Option<&'a str>),
    ) {
        let header = self.prepare_element_header(region, element, parent.0);
        let mask = match self.structural_mask(&header, &[]) {
            Ok(mask) => mask,
            Err(kind) => {
                self.hole(region, kind, header.span);
                StructuralHeadMask::default()
            }
        };
        self.element_body(region, header, mask, parent);
    }

    pub(super) fn element_body(
        &mut self,
        region: &mut RegionBuilder<'_, 'a>,
        header: PreparedElement<'_, 'a>,
        mask: StructuralHeadMask<'_>,
        parent: (Namespace, Option<&'a str>),
    ) {
        let mask = if mask.belongs_to(header.carrier) {
            mask
        } else {
            self.hole(region, NativeHoleKind::DirectiveSyntax, header.span);
            StructuralHeadMask::default()
        };
        match header.admission {
            HeaderAdmission::PreCarrier => return,
            HeaderAdmission::MissingOwner | HeaderAdmission::UnsupportedTag => {
                self.children(region, &header.carrier.children, parent);
                return;
            }
            HeaderAdmission::Ready => {}
        }
        let PreparedElement {
            carrier,
            span,
            tag,
            namespace,
            children_namespace,
            native,
            attributes,
            directives,
            ..
        } = header;
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
                if !mask.consumes(directive.ordinal) {
                    self.directive(region, directive);
                }
            }
            self.children(region, &carrier.children, (children_namespace, Some(tag)));
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
        ordinal: usize,
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
                    ordinal,
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
