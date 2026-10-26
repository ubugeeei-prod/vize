//! Native HTML structure, with a fail-closed explicit-envelope insertion mode.
//!
//! Host rules come from `P: Profile`: name folding, self-closing, implied end
//! tags and the table content model each branch on one associated constant.
//! `html_structure` builds with `Document`; tests also build with `Component`.

use vize_l0::{Span, Vec};

use super::{DocumentHtmlRefusal as Refusal, DocumentHtmlStructure, Element};
use crate::markup::document::{DocumentTokenKind as Kind, NativeDocument};
use crate::markup::{Document, Profile};

mod insertion;
mod table;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Doctype,
    Html,
    Head,
    InHead,
    Body,
    InBody,
    InTable,
    InTableBody,
    InRow,
    InCell,
    InCaption,
    InColumnGroup,
    HtmlEnd,
    End,
}

/// One authored start tag, checked against its original frame.
#[derive(Clone)]
struct Tag {
    name: &'static str,
    span: Span,
    opening: Span,
    slash: bool,
    attributes: core::ops::Range<usize>,
    /// The `OpenTagName` event; implied closes end their content here.
    event: usize,
}

pub(super) fn construct<'o, 'a>(
    owner: &'o NativeDocument<'a>,
) -> Result<DocumentHtmlStructure<'o, 'a>, Refusal> {
    construct_as::<Document>(owner)
}

pub(super) fn construct_as<'o, 'a, P: Profile>(
    owner: &'o NativeDocument<'a>,
) -> Result<DocumentHtmlStructure<'o, 'a>, Refusal> {
    owner.normal_completion().map_err(Refusal::Lexical)?;
    let mut builder = Builder::<P> {
        owner,
        elements: Vec::new_in(&owner.allocator),
        stack: Vec::new_in(&owner.allocator),
        mode: Mode::Doctype,
        body_tail_text: None,
        tail_owner: None,
        profile: core::marker::PhantomData,
    };
    builder.run()?;
    if builder.mode != Mode::End || !builder.stack.is_empty() {
        return Err(Refusal::PendingStructure);
    }
    Ok(DocumentHtmlStructure {
        owner,
        elements: builder.elements,
        body_tail_text: builder.body_tail_text,
        tail_owner: builder.tail_owner,
    })
}

struct Builder<'o, 'a, P> {
    owner: &'o NativeDocument<'a>,
    elements: Vec<'a, Element>,
    stack: Vec<'a, usize>,
    mode: Mode,
    body_tail_text: Option<Span>,
    tail_owner: Option<usize>,
    profile: core::marker::PhantomData<P>,
}

impl<P: Profile> Builder<'_, '_, P> {
    fn run(&mut self) -> Result<(), Refusal> {
        let mut index = 0;
        while let Some(event) = self.owner.events.get(index) {
            match event.kind() {
                Kind::OpenTagName => {
                    let name = event.span;
                    let open = index;
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
                    let tag = self.tag(name, gt, slash, attributes_start..index - 1, open)?;
                    self.start(tag)?;
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
                    let space = html_space(self.source(event.span)?);
                    self.text(event.span, space)?;
                }
                Kind::TextEntity => {
                    let value = event.decoded_entity().ok_or(Refusal::ExplicitEnvelope)?;
                    let mut space = true;
                    value.for_each(|ch| space &= matches!(ch, '\t' | '\n' | '\x0c' | '\r' | ' '));
                    self.text(event.span, space)?;
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

    /// Text belongs to body content and cells; table text may only be
    /// whitespace, which the browser keeps in place instead of fostering it.
    fn text(&mut self, span: Span, space: bool) -> Result<(), Refusal> {
        match self.mode {
            Mode::InBody | Mode::InCell | Mode::InCaption => Ok(()),
            Mode::InTable | Mode::InTableBody | Mode::InRow | Mode::InColumnGroup if !space => {
                Err(Refusal::TableText(span))
            }
            Mode::InTable | Mode::InTableBody | Mode::InRow | Mode::InColumnGroup => Ok(()),
            _ if !space => Err(Refusal::ExplicitEnvelope),
            Mode::HtmlEnd | Mode::End => {
                if !span.is_empty() {
                    self.body_tail_text.get_or_insert(span);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn source(&self, span: Span) -> Result<&str, Refusal> {
        self.owner
            .source()
            .get(span.start as usize..span.end as usize)
            .ok_or(Refusal::InvalidFrame(span))
    }

    fn tag(
        &self,
        name_span: Span,
        gt: u32,
        slash: bool,
        attributes: core::ops::Range<usize>,
        event: usize,
    ) -> Result<Tag, Refusal> {
        let name =
            name::<P>(self.source(name_span)?).ok_or(Refusal::UnsupportedElement(name_span))?;
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
        Ok(Tag {
            name,
            span: name_span,
            opening: Span::new(start, gt + 1),
            slash,
            attributes,
            event,
        })
    }

    fn start(&mut self, tag: Tag) -> Result<(), Refusal> {
        let next = match (self.mode, tag.name) {
            (Mode::Html, "html") => Mode::Head,
            (Mode::Head, "head") => Mode::InHead,
            (Mode::Body, "body") => Mode::InBody,
            (Mode::InHead, "base" | "link" | "meta") => Mode::InHead,
            (_, "html" | "head" | "body" | "base" | "link" | "meta") => {
                return Err(Refusal::ExplicitEnvelope);
            }
            (Mode::InBody, _) => return self.body_start(tag),
            (
                Mode::InTable
                | Mode::InTableBody
                | Mode::InRow
                | Mode::InCell
                | Mode::InCaption
                | Mode::InColumnGroup,
                _,
            ) => return self.table_start(tag),
            _ => return Err(Refusal::ExplicitEnvelope),
        };
        self.insert(tag)?;
        self.mode = next;
        Ok(())
    }

    fn close(&mut self, name_span: Span, event: usize) -> Result<(), Refusal> {
        let name =
            name::<P>(self.source(name_span)?).ok_or(Refusal::UnsupportedElement(name_span))?;
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
        // SourceRoot bounds the complete source to u32 before this producer.
        let closing = Span::new(start, (name_span.end as usize + gap + 1) as u32);
        let end = End {
            name,
            span: name_span,
            closing,
            event,
        };
        match (self.mode, name) {
            (Mode::InHead, "head") => {
                self.pop_matching(&end)?;
                self.mode = Mode::Body;
                Ok(())
            }
            (Mode::HtmlEnd, "html") => {
                self.pop_matching(&end)?;
                self.mode = Mode::End;
                Ok(())
            }
            (Mode::InBody, _) => self.body_end(&end),
            (
                Mode::InTable
                | Mode::InTableBody
                | Mode::InRow
                | Mode::InCell
                | Mode::InCaption
                | Mode::InColumnGroup,
                _,
            ) => self.table_end(&end),
            _ => Err(Refusal::ExplicitEnvelope),
        }
    }

    /// Insert at the current node. Void elements, and under a profile that
    /// self-closes any element a slashed tag, never become the current node.
    fn insert(&mut self, tag: Tag) -> Result<(), Refusal> {
        let closed = void(tag.name) || (tag.slash && P::SELF_CLOSING_ANY_ELEMENT);
        let parent = self.stack.last().copied();
        let index = self.elements.len();
        let content_end = tag.attributes.end + 1;
        self.elements.push(Element {
            name: tag.name,
            authored_name: tag.span,
            opening: tag.opening,
            closing: None,
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
            ignored_slash: tag.slash && !closed,
            attributes: tag.attributes,
            content_end,
        });
        if let Some(parent) = parent {
            let invalid = Refusal::InvalidFrame(tag.span);
            let previous = self.elements.get(parent).ok_or(invalid)?.last_child;
            if let Some(previous) = previous {
                self.elements.get_mut(previous).ok_or(invalid)?.next_sibling = Some(index);
            } else {
                self.elements.get_mut(parent).ok_or(invalid)?.first_child = Some(index);
            }
            self.elements.get_mut(parent).ok_or(invalid)?.last_child = Some(index);
        }
        if !closed {
            self.stack.push(index);
        }
        Ok(())
    }

    /// The current node's local name.
    fn current(&self) -> Option<&'static str> {
        self.name_at(self.stack.len().checked_sub(1)?)
    }

    fn name_at(&self, depth: usize) -> Option<&'static str> {
        let index = *self.stack.get(depth)?;
        self.elements.get(index).map(|element| element.name)
    }

    /// Pop the current node, closed by `closing` or implied at `event`.
    fn pop(&mut self, closing: Option<Span>, event: usize) -> Result<(), Refusal> {
        let index = self.stack.pop().ok_or(Refusal::PendingStructure)?;
        let element = self
            .elements
            .get_mut(index)
            .ok_or(Refusal::PendingStructure)?;
        element.closing = closing;
        element.content_end = event;
        Ok(())
    }

    /// Pop the current node with this authored end tag, which must name it.
    fn pop_matching(&mut self, end: &End) -> Result<(), Refusal> {
        if self.current() != Some(end.name) {
            return Err(Refusal::ImpliedEnd(end.span));
        }
        self.pop(Some(end.closing), end.event)
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

/// One authored end tag, checked against its original frame.
struct End {
    name: &'static str,
    span: Span,
    closing: Span,
    /// The `CloseTagName` event; implied closes end their content here.
    event: usize,
}

const NAMES: [&str; 55] = [
    "html",
    "head",
    "body",
    "base",
    "link",
    "meta",
    "div",
    "span",
    "br",
    "hr",
    "img",
    "input",
    "area",
    "embed",
    "source",
    "track",
    "wbr",
    "article",
    "aside",
    "footer",
    "header",
    "main",
    "nav",
    "section",
    "address",
    "blockquote",
    "figure",
    "figcaption",
    "hgroup",
    "search",
    "p",
    "ul",
    "ol",
    "menu",
    "li",
    "dl",
    "dt",
    "dd",
    "ruby",
    "rb",
    "rt",
    "rtc",
    "rp",
    "option",
    "optgroup",
    "table",
    "caption",
    "colgroup",
    "col",
    "tbody",
    "thead",
    "tfoot",
    "tr",
    "td",
    "th",
];

/// The admitted local name, compared without ASCII case under a folding profile.
fn name<P: Profile>(raw: &str) -> Option<&'static str> {
    NAMES.into_iter().find(|name| {
        if P::FOLD_NAME_CASE {
            raw.eq_ignore_ascii_case(name)
        } else {
            raw == *name
        }
    })
}

fn void(name: &str) -> bool {
    matches!(
        name,
        "base"
            | "link"
            | "meta"
            | "br"
            | "hr"
            | "img"
            | "input"
            | "area"
            | "embed"
            | "source"
            | "track"
            | "wbr"
            | "col"
    )
}

fn html_space(raw: &str) -> bool {
    raw.bytes()
        .all(|byte| matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '))
}

#[cfg(test)]
mod tests;
