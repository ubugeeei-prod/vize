//! Original petite-vue directive frames from the retained Document event tape.

use core::ops::Range;

use vize_l0::Span;

use super::{DocumentLexicalRefusal, DocumentToken, DocumentTokenKind as Kind, NativeDocument};
use crate::markup::QuoteType;

/// The two directive heads owned by petite-vue rather than core Vue DOM syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentPetiteDirectiveKind {
    Scope,
    Effect,
}

/// Why an original Document run cannot expose petite-vue directive frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentPetiteRefusal {
    Lexical(DocumentLexicalRefusal),
    InvalidFrame(Span),
}

/// A sealed cursor over original petite-vue directive attributes.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentPetiteDirectives;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentPetiteDirectives<'static, 'static>>();
/// ```
#[derive(Debug)]
pub struct DocumentPetiteDirectives<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    cursor: usize,
    element: Option<Span>,
}

/// One original `v-scope` or `v-effect` attribute frame.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentPetiteDirective;
/// fn forge<'o, 'a>(original: DocumentPetiteDirective<'o, 'a>) -> DocumentPetiteDirective<'o, 'a> {
///     DocumentPetiteDirective { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct DocumentPetiteDirective<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    kind: DocumentPetiteDirectiveKind,
    element: Span,
    name: Span,
    span: Span,
    parts: Range<usize>,
    value_parts: Range<usize>,
    value_span: Option<Span>,
    quote: QuoteType,
}

/// An original directive value. Authored bytes and decoded entity payloads
/// remain separate observations; no replacement buffer is constructed.
///
/// ```
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::markup::document::{DocumentPetiteDirectiveKind, NativeDocument};
/// let arena = Allocator::default();
/// let source = "<div v-scope=\"{ n: 1 }\"></div>";
/// let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
/// let directive = owner.petite_directives().unwrap().next().unwrap().unwrap();
/// assert_eq!(directive.kind(), DocumentPetiteDirectiveKind::Scope);
/// assert_eq!(directive.value().unwrap().authored(), "{ n: 1 }");
/// ```
#[derive(Debug)]
pub struct DocumentPetiteValue<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    parts: Range<usize>,
    span: Span,
    quote: QuoteType,
}

pub(super) fn directives<'o, 'a>(
    owner: &'o NativeDocument<'a>,
) -> Result<DocumentPetiteDirectives<'o, 'a>, DocumentPetiteRefusal> {
    owner
        .normal_completion()
        .map_err(DocumentPetiteRefusal::Lexical)?;
    Ok(DocumentPetiteDirectives {
        owner,
        cursor: 0,
        element: None,
    })
}

impl<'o, 'a> Iterator for DocumentPetiteDirectives<'o, 'a> {
    type Item = Result<DocumentPetiteDirective<'o, 'a>, DocumentPetiteRefusal>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.owner.events.get(self.cursor) {
            match event.kind() {
                Kind::OpenTagName => self.element = Some(event.span),
                Kind::OpenTagEnd | Kind::SelfClosingTag => self.element = None,
                Kind::DirectiveName => {
                    let kind = match event.span.slice(self.owner.source()) {
                        "v-scope" => DocumentPetiteDirectiveKind::Scope,
                        "v-effect" => DocumentPetiteDirectiveKind::Effect,
                        _ => {
                            self.cursor += 1;
                            continue;
                        }
                    };
                    return Some(self.read(kind, event.span));
                }
                _ => {}
            }
            self.cursor += 1;
        }
        None
    }
}

impl<'o, 'a> DocumentPetiteDirectives<'o, 'a> {
    fn read(
        &mut self,
        kind: DocumentPetiteDirectiveKind,
        name: Span,
    ) -> Result<DocumentPetiteDirective<'o, 'a>, DocumentPetiteRefusal> {
        let Some(element) = self.element else {
            self.cursor += 1;
            return Err(DocumentPetiteRefusal::InvalidFrame(name));
        };
        let start = self.cursor;
        let mut index = start + 1;
        let mut value_start = None;
        let mut value_end = name.end;
        while let Some(event) = self.owner.events.get(index) {
            match event.kind() {
                Kind::DirectiveArgument | Kind::DirectiveModifier | Kind::AttributeNameEnd => {}
                Kind::AttributeData | Kind::AttributeEntity => {
                    value_start.get_or_insert(index);
                    value_end = event.span.end;
                }
                Kind::AttributeEnd(quote) => {
                    let frame_end = match quote {
                        QuoteType::Single | QuoteType::Double => event
                            .span
                            .end
                            .checked_add(1)
                            .ok_or(DocumentPetiteRefusal::InvalidFrame(event.span))?,
                        QuoteType::NoValue | QuoteType::Unquoted => event.span.end,
                    };
                    let value_parts = value_start.unwrap_or(index)..index;
                    let value_span = match quote {
                        QuoteType::NoValue => None,
                        _ => Some(Span::new(
                            value_start
                                .and_then(|part| self.owner.events.get(part))
                                .map_or(event.span.start, |part| part.span.start),
                            value_end.max(event.span.start),
                        )),
                    };
                    self.cursor = index + 1;
                    return Ok(DocumentPetiteDirective {
                        owner: self.owner,
                        kind,
                        element,
                        name,
                        span: Span::new(name.start, frame_end),
                        parts: start + 1..index,
                        value_parts,
                        value_span,
                        quote,
                    });
                }
                _ => {
                    self.cursor = index + 1;
                    return Err(DocumentPetiteRefusal::InvalidFrame(event.span));
                }
            }
            index += 1;
        }
        self.cursor = self.owner.events.len();
        Err(DocumentPetiteRefusal::InvalidFrame(name))
    }
}

impl<'o, 'a> DocumentPetiteDirective<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.owner
    }

    #[must_use]
    pub fn kind(&self) -> DocumentPetiteDirectiveKind {
        self.kind
    }

    #[must_use]
    pub fn element_name(&self) -> &'a str {
        self.element.slice(self.owner.source())
    }

    #[must_use]
    pub fn authored_name(&self) -> &'a str {
        self.name.slice(self.owner.source())
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub fn authored(&self) -> &'a str {
        self.span.slice(self.owner.source())
    }

    pub fn argument(&self) -> Option<DocumentToken<'o, 'a>> {
        self.owner
            .events
            .get(self.parts.clone())
            .into_iter()
            .flatten()
            .find(|event| event.kind() == Kind::DirectiveArgument)
            .map(|event| DocumentToken {
                owner: self.owner,
                event,
            })
    }

    pub fn modifiers(&self) -> impl Iterator<Item = DocumentToken<'o, 'a>> + '_ {
        self.owner
            .events
            .get(self.parts.clone())
            .into_iter()
            .flatten()
            .filter(|event| event.kind() == Kind::DirectiveModifier)
            .map(|event| DocumentToken {
                owner: self.owner,
                event,
            })
    }

    #[must_use]
    pub fn value(&self) -> Option<DocumentPetiteValue<'o, 'a>> {
        self.value_span.map(|span| DocumentPetiteValue {
            owner: self.owner,
            parts: self.value_parts.clone(),
            span,
            quote: self.quote,
        })
    }
}

impl<'o, 'a> DocumentPetiteValue<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.owner
    }

    #[must_use]
    pub fn quote(&self) -> QuoteType {
        self.quote
    }

    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub fn authored(&self) -> &'a str {
        self.span.slice(self.owner.source())
    }

    pub fn tokens(&self) -> impl Iterator<Item = DocumentToken<'o, 'a>> + '_ {
        self.owner
            .events
            .get(self.parts.clone())
            .into_iter()
            .flatten()
            .map(|event| DocumentToken {
                owner: self.owner,
                event,
            })
    }
}

#[cfg(test)]
mod tests;
