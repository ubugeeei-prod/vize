use super::{Context, Directive, NativeHoleKind, PreparedAttribute};
use crate::native::ConstructionFactory;
use vize_l0::Span;
use vize_l1::markup::directive::DirectivePrefix;
use vize_l1::{Element, ElementClose};
use vize_l2::op::{Attribute, Namespace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::native) enum HeaderAdmission {
    Ready,
    PreCarrier,
    MissingOwner,
    UnsupportedTag,
}

/// The exact carrier plus the only prepared observation of its attributes.
pub(in crate::native) struct PreparedElement<'surface, 'a> {
    pub carrier: &'surface Element<'a>,
    pub span: Span,
    pub admission: HeaderAdmission,
    pub tag: &'a str,
    pub namespace: Namespace,
    pub children_namespace: Namespace,
    pub native: bool,
    pub attributes: vize_l0::Vec<'a, Attribute<'a>>,
    pub directives: vize_l0::Vec<'a, Directive>,
}

/// Validated source ordinals, bound to their original carrier, consumed once.
#[derive(Default)]
pub(in crate::native) struct StructuralHeadMask<'mask> {
    carrier: Option<&'mask Element<'mask>>,
    ordinals: &'mask [usize],
}

impl StructuralHeadMask<'_> {
    pub(in crate::native) fn belongs_to(&self, carrier: &Element<'_>) -> bool {
        self.carrier
            .is_none_or(|original| core::ptr::eq(original, carrier))
    }

    pub(in crate::native) fn consumes(&self, ordinal: usize) -> bool {
        self.ordinals.contains(&ordinal)
    }
}

impl<'a> Context<'_, 'a> {
    pub(in crate::native) fn prepare_element_header<'surface, R: ConstructionFactory<'a>>(
        &mut self,
        region: &mut R,
        carrier: &'surface Element<'a>,
        parent: Namespace,
    ) -> PreparedElement<'surface, 'a> {
        let opening = self.token_span(&carrier.open.lt_name);
        let end = match &carrier.close {
            ElementClose::Present(close) if !close.gt.is_missing() => {
                Some(self.token_span(&close.gt).end)
            }
            ElementClose::NotExpected if !carrier.open.gt.is_missing() => {
                Some(self.token_span(&carrier.open.gt).end)
            }
            _ => None,
        };
        let span = Span::new(opening.start, end.unwrap_or(opening.end));
        if end.is_none() {
            self.hole(region, NativeHoleKind::MissingMarkup, opening);
        }
        let mut attributes = vize_l0::Vec::new_in(&self.allocator);
        let mut directives = vize_l0::Vec::new_in(&self.allocator);
        let pre = carrier.open.is_verbatim();
        let mut component_name = None;
        let mut selector = None;
        let mut is_seen = false;
        if !pre {
            for (ordinal, attribute) in carrier.open.attrs.iter().enumerate() {
                match self.attribute(region, ordinal, attribute) {
                    Some(PreparedAttribute::Static(attribute)) => {
                        if attribute.name == "is"
                            && !is_seen
                            && !matches!(carrier.tag(), "component" | "Component" | "slot")
                        {
                            is_seen = true;
                            component_name =
                                attribute.value.and_then(|value| value.strip_prefix("vue:"));
                            if component_name.is_some() {
                                if carrier.tag() != "template" {
                                    continue;
                                }
                                selector = Some(attributes.len());
                            }
                        }
                        attributes.push(attribute);
                    }
                    Some(PreparedAttribute::Directive(directive)) => {
                        directives.push(directive);
                    }
                    None => {}
                }
            }
        }
        let tag = carrier.tag();
        if tag == "template"
            && directives.iter().any(|directive| {
                directive.head.prefix == DirectivePrefix::Slot
                    || directive.head.prefix == DirectivePrefix::Full
                        && matches!(
                            directive.head.name.slice(self.block().root_source()),
                            "if" | "else-if" | "else" | "for" | "slot"
                        )
            })
        {
            component_name = None;
        }
        if component_name.is_some()
            && let Some(ordinal) = selector
        {
            attributes.remove(ordinal);
        }
        let unsupported = (tag != "template" || component_name.is_none())
            && matches!(
                tag,
                "template" | "slot" | "component" | "script" | "style" | "annotation-xml"
            );
        let admission = if pre {
            // The actual native mode was checked before ignored attribute work.
            // Retained raw descendants must never be re-admitted.
            self.hole(region, NativeHoleKind::PreCarrier, span);
            HeaderAdmission::PreCarrier
        } else if end.is_none() {
            HeaderAdmission::MissingOwner
        } else if unsupported {
            self.hole(region, NativeHoleKind::UnsupportedTag, opening);
            HeaderAdmission::UnsupportedTag
        } else {
            HeaderAdmission::Ready
        };
        let namespace = match (parent, tag) {
            (_, "svg") => Namespace::Svg,
            (_, "math") => Namespace::MathMl,
            _ => parent,
        };
        let children_namespace = match (namespace, tag) {
            (Namespace::Svg, "foreignObject" | "desc" | "title") => Namespace::Html,
            (Namespace::MathMl, "mi" | "mo" | "mn" | "ms" | "mtext") => Namespace::Html,
            _ => namespace,
        };
        PreparedElement {
            carrier,
            span,
            admission,
            tag: component_name.unwrap_or(tag),
            namespace,
            children_namespace,
            native: component_name.is_none()
                && (vize_l0::is_html_tag(tag)
                    || vize_l0::is_svg_tag(tag)
                    || vize_l0::is_math_ml_tag(tag)),
            attributes,
            directives,
        }
    }

    pub(in crate::native) fn structural_mask<'mask>(
        &self,
        header: &PreparedElement<'mask, 'a>,
        ordinals: &'mask [usize],
    ) -> Result<StructuralHeadMask<'mask>, NativeHoleKind> {
        for (at, ordinal) in ordinals.iter().enumerate() {
            if ordinals
                .get(..at)
                .is_some_and(|previous| previous.contains(ordinal))
            {
                return Err(NativeHoleKind::DirectiveSyntax);
            }
            let Some(directive) = header
                .directives
                .iter()
                .find(|head| head.ordinal == *ordinal)
            else {
                return Err(NativeHoleKind::DirectiveSyntax);
            };
            let name = self
                .block()
                .root_source()
                .get(directive.head.name.start as usize..directive.head.name.end as usize);
            if directive.head.prefix != DirectivePrefix::Full
                || !matches!(name, Some("if" | "else-if" | "else" | "for"))
                || directive.head.arg.is_some()
                || directive.head.modifiers.start != directive.head.modifiers.end
            {
                return Err(NativeHoleKind::DirectiveSyntax);
            }
        }
        Ok(StructuralHeadMask {
            carrier: Some(header.carrier),
            ordinals,
        })
    }
}

#[cfg(test)]
mod tests;
