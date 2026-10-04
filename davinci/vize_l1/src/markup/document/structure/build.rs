//! Native HTML structure, with a fail-closed explicit-envelope insertion mode.

use vize_l0::{Span, Vec};

use super::{DocumentHtmlRefusal as Refusal, DocumentHtmlStructure, Element};
use crate::markup::document::{DocumentTokenKind as Kind, DocumentTreePolicy, NativeDocument};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Doctype,
    Html,
    Head,
    InHead,
    Body,
    InBody,
    HtmlEnd,
    End,
}

pub(super) fn construct<'o, 'a>(
    owner: &'o NativeDocument<'a>,
) -> Result<DocumentHtmlStructure<'o, 'a>, Refusal> {
    owner.normal_completion().map_err(Refusal::Lexical)?;
    let mut builder = Builder {
        owner,
        elements: Vec::new_in(&owner.allocator),
        stack: Vec::new_in(&owner.allocator),
        mode: Mode::Doctype,
        body_tail_text: None,
    };
    builder.run()?;
    if builder.mode != Mode::End || !builder.stack.is_empty() {
        return Err(Refusal::PendingStructure);
    }
    Ok(DocumentHtmlStructure {
        owner,
        elements: builder.elements,
        body_tail_text: builder.body_tail_text,
    })
}

struct Builder<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    elements: Vec<'a, Element>,
    stack: Vec<'a, usize>,
    mode: Mode,
    body_tail_text: Option<Span>,
}

impl Builder<'_, '_> {
    fn run(&mut self) -> Result<(), Refusal> {
        let mut index = 0;
        while let Some(event) = self.owner.events.get(index) {
            match event.kind() {
                Kind::OpenTagName => {
                    let name = event.span;
                    index += 1;
                    let attributes_start = index;
                    let mut attribute_count = 0;
                    let mut end = None;
                    while let Some(part) = self.owner.events.get(index) {
                        index += 1;
                        match part.kind() {
                            Kind::OpenTagEnd | Kind::SelfClosingTag => {
                                end = Some((part.span.start, part.kind() == Kind::SelfClosingTag));
                                break;
                            }
                            Kind::AttributeName => {
                                attribute_count += 1;
                                if attribute_count > DocumentHtmlStructure::MAX_ATTRIBUTES {
                                    return Err(Refusal::AttributeCountLimit(name));
                                }
                            }
                            Kind::AttributeNameEnd
                            | Kind::AttributeData
                            | Kind::AttributeEntity
                            | Kind::AttributeEnd(_) => {}
                            Kind::DirectiveName
                            | Kind::DirectiveArgument
                            | Kind::DirectiveModifier => {
                                return Err(Refusal::VueSyntax);
                            }
                            _ => return Err(Refusal::InvalidFrame(name)),
                        }
                    }
                    let (gt, slash) = end.ok_or(Refusal::InvalidFrame(name))?;
                    self.open(name, gt, slash, attributes_start..index - 1)?;
                    continue;
                }
                Kind::CloseTagName => self.close(event.span, index)?,
                Kind::Declaration { terminated: true } => {
                    let raw = self.source(event.span)?;
                    if self.mode != Mode::Doctype || !raw.eq_ignore_ascii_case("<!DOCTYPE html>") {
                        return Err(Refusal::ExplicitEnvelope);
                    }
                    self.mode = Mode::Html;
                }
                Kind::Text => {
                    if self.mode != Mode::InBody && !html_space(self.source(event.span)?) {
                        return Err(Refusal::ExplicitEnvelope);
                    }
                    if matches!(self.mode, Mode::HtmlEnd | Mode::End) && !event.span.is_empty() {
                        self.body_tail_text.get_or_insert(event.span);
                    }
                }
                Kind::TextEntity if self.mode == Mode::InBody => {}
                Kind::TextEntity => {
                    let value = event.decoded_entity().ok_or(Refusal::ExplicitEnvelope)?;
                    let mut space = true;
                    value.for_each(|ch| space &= matches!(ch, '\t' | '\n' | '\x0c' | '\r' | ' '));
                    if !space {
                        return Err(Refusal::ExplicitEnvelope);
                    }
                    if matches!(self.mode, Mode::HtmlEnd | Mode::End) {
                        self.body_tail_text.get_or_insert(event.span);
                    }
                }
                Kind::Comment => self.comment(event.span)?,
                Kind::Interpolation
                | Kind::DirectiveName
                | Kind::DirectiveArgument
                | Kind::DirectiveModifier => return Err(Refusal::VueSyntax),
                _ => return Err(Refusal::NonHtmlToken),
            }
            index += 1;
        }
        Ok(())
    }

    fn source(&self, span: Span) -> Result<&str, Refusal> {
        self.owner
            .source()
            .get(span.start as usize..span.end as usize)
            .ok_or(Refusal::InvalidFrame(span))
    }

    fn open(
        &mut self,
        name_span: Span,
        gt: u32,
        slash: bool,
        attributes: core::ops::Range<usize>,
    ) -> Result<(), Refusal> {
        let raw = self.source(name_span)?;
        let name = name(raw).ok_or_else(|| unsupported(raw, name_span))?;
        let start = name_span
            .start
            .checked_sub(1)
            .ok_or(Refusal::InvalidFrame(name_span))?;
        let bytes = self.owner.source().as_bytes();
        if gt + 1 - start > DocumentHtmlStructure::MAX_OPENING_BYTES {
            return Err(Refusal::OpeningByteLimit(name_span));
        }
        if bytes.get(start as usize) != Some(&b'<')
            || bytes.get(gt as usize) != Some(&b'>')
            || (slash && gt.checked_sub(1).and_then(|p| bytes.get(p as usize)) != Some(&b'/'))
        {
            return Err(Refusal::InvalidFrame(name_span));
        }
        let void = matches!(
            name,
            "base" | "link" | "meta" | "br" | "hr" | "img" | "input"
        );
        self.mode = match (self.mode, name) {
            (Mode::Html, "html") => Mode::Head,
            (Mode::Head, "head") => Mode::InHead,
            (Mode::Body, "body") => Mode::InBody,
            (Mode::InHead, "base" | "link" | "meta") => Mode::InHead,
            (Mode::InBody, "div" | "span" | "br" | "hr" | "img" | "input") => Mode::InBody,
            _ => return Err(Refusal::ExplicitEnvelope),
        };
        let parent = self.stack.last().copied();
        let index = self.elements.len();
        let content_end = attributes.end + 1;
        self.elements.push(Element {
            name,
            authored_name: name_span,
            opening: Span::new(start, gt + 1),
            closing: None,
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
            ignored_slash: slash && !void,
            attributes,
            content_end,
        });
        if let Some(parent) = parent {
            let previous = self
                .elements
                .get(parent)
                .ok_or(Refusal::InvalidFrame(name_span))?
                .last_child;
            if let Some(previous) = previous {
                self.elements
                    .get_mut(previous)
                    .ok_or(Refusal::InvalidFrame(name_span))?
                    .next_sibling = Some(index);
            } else {
                self.elements
                    .get_mut(parent)
                    .ok_or(Refusal::InvalidFrame(name_span))?
                    .first_child = Some(index);
            }
            self.elements
                .get_mut(parent)
                .ok_or(Refusal::InvalidFrame(name_span))?
                .last_child = Some(index);
        }
        if !void {
            self.stack.push(index);
        }
        Ok(())
    }

    fn close(&mut self, name_span: Span, event_index: usize) -> Result<(), Refusal> {
        let raw = self.source(name_span)?;
        let name = name(raw).ok_or_else(|| unsupported(raw, name_span))?;
        let start = name_span
            .start
            .checked_sub(2)
            .ok_or(Refusal::InvalidFrame(name_span))?;
        let source = self.owner.source();
        if source.get(start as usize..name_span.start as usize) != Some("</") {
            return Err(Refusal::InvalidFrame(name_span));
        }
        let tail = source
            .get(name_span.end as usize..)
            .ok_or(Refusal::InvalidFrame(name_span))?;
        let gap = tail.find('>').ok_or(Refusal::InvalidFrame(name_span))?;
        if !html_space(tail.get(..gap).ok_or(Refusal::InvalidFrame(name_span))?) {
            return Err(Refusal::InvalidFrame(name_span));
        }
        let index = self
            .stack
            .last()
            .copied()
            .ok_or(Refusal::ImpliedEnd(name_span))?;
        if self
            .elements
            .get(index)
            .ok_or(Refusal::InvalidFrame(name_span))?
            .name
            != name
        {
            return Err(Refusal::ImpliedEnd(name_span));
        }
        self.mode = match (self.mode, name) {
            (Mode::InHead, "head") => Mode::Body,
            (Mode::InBody, "body") => Mode::HtmlEnd,
            (Mode::HtmlEnd, "html") => Mode::End,
            (Mode::InBody, "div" | "span") => Mode::InBody,
            _ => return Err(Refusal::ExplicitEnvelope),
        };
        self.stack.pop();
        // SourceRoot bounds the complete source to u32 before this producer.
        let end = name_span.end as usize + gap + 1;
        let element = self
            .elements
            .get_mut(index)
            .ok_or(Refusal::InvalidFrame(name_span))?;
        element.closing = Some(Span::new(start, end as u32));
        element.content_end = event_index;
        Ok(())
    }

    fn comment(&self, content: Span) -> Result<(), Refusal> {
        let source = self.owner.source();
        let start = content.start.checked_sub(4).ok_or(Refusal::NonHtmlToken)?;
        let end = (content.end as usize)
            .checked_add(3)
            .ok_or(Refusal::NonHtmlToken)?;
        if source.get(start as usize..content.start as usize) != Some("<!--")
            || source.get(content.end as usize..end) != Some("-->")
        {
            return Err(Refusal::NonHtmlToken);
        }
        let raw = self.source(content)?;
        if raw.contains("<!--") || raw.contains("-->") || raw.contains("--!>") {
            return Err(Refusal::NonHtmlToken);
        }
        Ok(())
    }
}

fn name(raw: &str) -> Option<&'static str> {
    [
        "html", "head", "body", "div", "span", "base", "link", "meta", "br", "hr", "img", "input",
    ]
    .into_iter()
    .find(|name| raw.eq_ignore_ascii_case(name))
}

fn unsupported(raw: &str, span: Span) -> Refusal {
    if [
        "table", "thead", "tbody", "tfoot", "tr", "td", "th", "caption", "col", "colgroup",
    ]
    .iter()
    .any(|tag| raw.eq_ignore_ascii_case(tag))
    {
        Refusal::UnsupportedPolicy(DocumentTreePolicy::TableContentModel)
    } else if ["p", "li", "dt", "dd", "rt", "rp", "option", "optgroup"]
        .iter()
        .any(|tag| raw.eq_ignore_ascii_case(tag))
    {
        Refusal::UnsupportedPolicy(DocumentTreePolicy::ImpliedEndTags)
    } else {
        Refusal::UnsupportedElement(span)
    }
}

fn html_space(raw: &str) -> bool {
    raw.bytes()
        .all(|byte| matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '))
}
