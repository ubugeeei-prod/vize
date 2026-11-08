//! Lossless L1 template helpers shared by script rules.

use oxc_span::Span as OxcSpan;
use vize_l0::{Span, String};
use vize_l1::markup::directive::{ArgSyntax, DirectivePrefix, DirectiveSyntax, VueDirectives};
use vize_l1::markup::entity::{DecodedEntity, EntityContext, decode_one};
use vize_l1::markup::{AttrValue, Attribute, Element, ElementClose, SurfaceTree, Token};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DirectiveArgument<'a> {
    Static(&'a str),
    Dynamic(&'a str),
}

pub(super) struct Directive<'a> {
    pub name: &'a str,
    pub argument: Option<DirectiveArgument<'a>>,
    pub value: Option<DecodedSource<'a>>,
}

pub(super) fn directive<'a>(
    tree: &SurfaceTree<'a>,
    attribute: &'a Attribute<'a>,
    verbatim: bool,
) -> Option<Directive<'a>> {
    if verbatim {
        return None;
    }
    let raw = attribute.name.text;
    let parsed = VueDirectives.decompose(raw, 0).ok()??;
    let name = match parsed.prefix {
        DirectivePrefix::Full => slice(raw, parsed.name)?,
        DirectivePrefix::Bind | DirectivePrefix::Prop => "bind",
        DirectivePrefix::On => "on",
        DirectivePrefix::Slot => "slot",
    };
    let argument = parsed.arg.and_then(|argument| match argument {
        ArgSyntax::Static(span) => slice(raw, span).map(DirectiveArgument::Static),
        ArgSyntax::Dynamic(span) => slice(raw, span).map(DirectiveArgument::Dynamic),
    });
    Some(Directive {
        name,
        argument,
        value: attribute
            .value
            .as_ref()
            .and_then(|value| DecodedSource::attribute(tree.source, value)),
    })
}

pub(super) fn has_directive(element: &Element<'_>, name: &str, verbatim: bool) -> bool {
    element.open.attrs.iter().any(|attribute| {
        directive_name(attribute, verbatim).is_some_and(|candidate| candidate == name)
    })
}

pub(super) fn directive_name<'a>(attribute: &'a Attribute<'a>, verbatim: bool) -> Option<&'a str> {
    if verbatim {
        return None;
    }
    let raw = attribute.name.text;
    let parsed = VueDirectives.decompose(raw, 0).ok()??;
    Some(match parsed.prefix {
        DirectivePrefix::Full => slice(raw, parsed.name)?,
        DirectivePrefix::Bind | DirectivePrefix::Prop => "bind",
        DirectivePrefix::On => "on",
        DirectivePrefix::Slot => "slot",
    })
}

pub(super) fn element_verbatim(element: &Element<'_>, inherited: bool) -> bool {
    inherited
        || element.open.is_verbatim()
        || element
            .open
            .attrs
            .iter()
            .any(|attribute| directive_name(attribute, false) == Some("pre"))
}

pub(super) fn token_span(source: &str, token: &Token<'_>) -> Option<OxcSpan> {
    let start = source_offset(source, token.text)?;
    Some(OxcSpan::new(
        start,
        start.checked_add(token.text.len() as u32)?,
    ))
}

pub(super) fn element_span(source: &str, element: &Element<'_>) -> Option<OxcSpan> {
    let start = token_span(source, &element.open.lt_name)?.start;
    let end = match &element.close {
        ElementClose::Present(close) => token_span(source, &close.gt)?.end,
        ElementClose::Implicit | ElementClose::Missing | ElementClose::NotExpected => {
            token_span(source, &element.open.gt)?.end
        }
    };
    Some(OxcSpan::new(start, end))
}

pub(super) fn static_attribute_value<'a>(
    tree: &SurfaceTree<'a>,
    attribute: &'a Attribute<'a>,
) -> Option<DecodedSource<'a>> {
    DecodedSource::attribute(tree.source, attribute.value.as_ref()?)
}

pub(super) struct DecodedSource<'a> {
    authored: &'a str,
    decoded: Option<String>,
    offsets: Option<Vec<u32>>,
    pub start: u32,
}

impl<'a> DecodedSource<'a> {
    pub(super) fn interpolation(source: &'a str, token: &'a Token<'a>) -> Option<Self> {
        let raw = token.text;
        let trimmed = raw.trim();
        let leading = raw.len().checked_sub(raw.trim_start().len())?;
        let start = source_offset(source, raw)?.checked_add(leading as u32)?;
        Some(Self::decoded(trimmed, start, EntityContext::Text))
    }

    fn attribute(source: &'a str, value: &'a AttrValue<'a>) -> Option<Self> {
        let authored = value.content.text;
        let start = source_offset(source, authored)?;
        Some(Self::decoded(authored, start, EntityContext::Attribute))
    }

    fn decoded(authored: &'a str, start: u32, context: EntityContext) -> Self {
        if !authored.as_bytes().contains(&b'&') {
            return Self {
                authored,
                decoded: None,
                offsets: None,
                start,
            };
        }
        let mut decoded = String::default();
        let mut offsets = vec![0];
        let mut at = 0;
        while let Some(rest) = authored.get(at..)
            && let Some(first) = rest.chars().next()
        {
            let len = if let Some((entity, len)) = decode_one(rest.as_bytes(), context) {
                push_entity(&mut decoded, &mut offsets, entity, at as u32);
                len
            } else {
                decoded.push(first);
                offsets.extend(std::iter::repeat_n(at as u32, first.len_utf8() - 1));
                first.len_utf8()
            };
            at += len;
            offsets.push(at as u32);
        }
        if decoded == authored {
            Self {
                authored,
                decoded: None,
                offsets: None,
                start,
            }
        } else {
            Self {
                authored,
                decoded: Some(decoded),
                offsets: Some(offsets),
                start,
            }
        }
    }

    pub(super) fn text(&self) -> &str {
        self.decoded.as_deref().unwrap_or(self.authored)
    }

    pub(super) fn authored(&self) -> &str {
        self.authored
    }

    pub(super) fn authored_span(&self, span: OxcSpan) -> Option<OxcSpan> {
        self.text().get(span.start as usize..span.end as usize)?;
        let Some(offsets) = &self.offsets else {
            return Some(span);
        };
        Some(OxcSpan::new(
            *offsets.get(span.start as usize)?,
            *offsets.get(span.end as usize)?,
        ))
    }
}

fn push_entity(
    decoded: &mut String,
    offsets: &mut Vec<u32>,
    entity: DecodedEntity,
    authored_offset: u32,
) {
    entity.for_each(|character| {
        decoded.push(character);
        offsets.extend(std::iter::repeat_n(authored_offset, character.len_utf8()));
    });
    offsets.pop();
}

fn slice(source: &str, span: Span) -> Option<&str> {
    source.get(span.start as usize..span.end as usize)
}

fn source_offset(source: &str, slice: &str) -> Option<u32> {
    let source_start = source.as_ptr() as usize;
    let source_end = source_start.checked_add(source.len())?;
    let slice_start = slice.as_ptr() as usize;
    let slice_end = slice_start.checked_add(slice.len())?;
    (slice_start >= source_start && slice_end <= source_end)
        .then(|| u32::try_from(slice_start - source_start).ok())?
}
