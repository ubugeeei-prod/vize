use super::pattern::Directive;
use super::{Context, NativeHoleKind};
use vize_l0::Span;
use vize_l1::dialect::vue3::VueDirectives;
use vize_l1::embed::prepare_attribute_value;
use vize_l1::markup::directive::DirectiveSyntax;
use vize_l1::{Attribute as SurfaceAttribute, Element};
use vize_l2::artifact::{ComponentBody, ComponentFactory};
use vize_l2::op::{Attribute, Namespace};

mod header;
pub(super) use header::{HeaderAdmission, PreparedElement, StructuralHeadMask};

enum PreparedAttribute<'a> {
    Static(Attribute<'a>),
    Directive(Directive),
}

impl<'a> Context<'a> {
    pub(super) fn element<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
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

    pub(super) fn element_body<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
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
            admission: _,
        } = header;
        let children = ElementBody {
            cx: self,
            carrier,
            mask,
            directives,
            span,
            native,
            tag,
            children_namespace,
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

    fn attribute<R: ComponentFactory<'a>>(
        &mut self,
        region: &mut R,
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

struct ElementBody<'ctx, 'surface, 'mask, 'a> {
    cx: &'ctx mut Context<'a>,
    carrier: &'surface Element<'a>,
    mask: StructuralHeadMask<'mask>,
    directives: vize_l0::Vec<'a, Directive>,
    span: Span,
    native: bool,
    tag: &'a str,
    children_namespace: Namespace,
}

impl<'a> ComponentBody<'a> for ElementBody<'_, '_, '_, 'a> {
    fn run<R: ComponentFactory<'a>>(self, region: &mut R, node: vize_l0::id::NodeId) {
        self.cx.record(
            region,
            if self.native {
                "native.element"
            } else {
                "native.component"
            },
            Some(node),
            self.span,
            if self.native {
                "ui.element"
            } else {
                "ui.component"
            },
        );
        for directive in self.directives {
            if !self.mask.consumes(directive.ordinal) {
                self.cx.directive(region, directive);
            }
        }
        self.cx.children(
            region,
            &self.carrier.children,
            (self.children_namespace, Some(self.tag)),
        );
    }
}

#[cfg(test)]
mod tests;
