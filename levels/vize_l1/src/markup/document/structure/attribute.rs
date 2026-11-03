//! Static HTML attribute readback from the original private start-tag events.

use vize_l0::Span;

use super::{DocumentHtmlElement, DocumentHtmlStructure, Element};
use crate::markup::{QuoteType, document::DocumentTokenKind as Kind, entity::DecodedEntity};

/// A borrowed iterator over the original tag's effective static attributes.
/// No attribute table, folded-name set or decoded value buffer is constructed.
#[derive(Debug)]
pub struct DocumentHtmlAttributes<'s, 'o, 'a> {
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    element: &'s Element,
    cursor: usize,
}

pub(super) fn attributes<'s, 'o, 'a>(
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    element: &'s Element,
) -> DocumentHtmlAttributes<'s, 'o, 'a> {
    DocumentHtmlAttributes {
        tree,
        element,
        cursor: element.attributes.start,
    }
}

impl<'s, 'o, 'a> Iterator for DocumentHtmlAttributes<'s, 'o, 'a> {
    type Item = DocumentHtmlAttribute<'s, 'o, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.element.attributes.end {
            let index = self.cursor;
            let attribute = read(self.tree, self.element, index)?;
            self.cursor = attribute.parts.end + 1;
            let duplicate = self
                .tree
                .owner
                .events
                .get(self.element.attributes.start..index)?
                .iter()
                .filter(|event| event.kind() == Kind::AttributeName)
                .any(|event| {
                    name_characters(event.span.slice(self.tree.owner.source()))
                        .eq(attribute.name_characters())
                });
            if !duplicate {
                return Some(attribute);
            }
        }
        None
    }
}

/// One privately derived static attribute of an authentic original HTML element.
/// Scalar readback implements HTML name/value normalization, never JS syntax,
/// URL resolution, Boolean-property semantics or petite-vue directive meaning.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlAttribute;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentHtmlAttribute<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlAttribute;
/// fn forge<'s, 'o, 'a>(original: DocumentHtmlAttribute<'s, 'o, 'a>) -> DocumentHtmlAttribute<'s, 'o, 'a> {
///     DocumentHtmlAttribute { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct DocumentHtmlAttribute<'s, 'o, 'a> {
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    element: &'s Element,
    name: Span,
    parts: core::ops::Range<usize>,
    quote: QuoteType,
    value: Option<Span>,
}

impl<'s, 'o, 'a> DocumentHtmlAttribute<'s, 'o, 'a> {
    #[must_use]
    pub fn element(&self) -> DocumentHtmlElement<'s, 'o, 'a> {
        DocumentHtmlElement {
            tree: self.tree,
            element: self.element,
        }
    }

    #[must_use]
    pub fn authored_name(&self) -> &'a str {
        self.name.slice(self.tree.owner.source())
    }

    #[must_use]
    pub fn name_span(&self) -> Span {
        self.name
    }

    /// Actual HTML name scalars: ASCII uppercase folds; authored NUL replaces.
    pub fn name_characters(&self) -> impl Iterator<Item = char> + 'a {
        name_characters(self.authored_name())
    }

    #[must_use]
    pub fn quote(&self) -> QuoteType {
        self.quote
    }

    /// Authored content excluding quotes; Boolean attributes have no content.
    #[must_use]
    pub fn value_span(&self) -> Option<Span> {
        self.value
    }

    #[must_use]
    pub fn authored_value(&self) -> Option<&'a str> {
        self.value.map(|span| span.slice(self.tree.owner.source()))
    }

    /// DOM attribute value scalars, directly from the original event payloads.
    /// Literal CRLF/CR preprocesses to LF; literal NUL becomes replacement.
    /// Original decoded references bypass literal input preprocessing, so
    /// `&#13;` retains CR and `&amp;amp;` is never decoded for a second time.
    pub fn value_characters(&self) -> impl Iterator<Item = char> + 's {
        let owner = self.tree.owner;
        owner
            .events
            .get(self.parts.clone())
            .into_iter()
            .flatten()
            .flat_map(move |event| {
                if let Some(value) = event.decoded_entity() {
                    match value {
                        DecodedEntity::Named(value) => Scalars::Named(value.chars()),
                        DecodedEntity::Numeric(value) => Scalars::Numeric(Some(value)),
                    }
                } else {
                    Scalars::Literal(event.span.slice(owner.source()).chars().peekable())
                }
            })
    }
}

fn name_characters(raw: &str) -> impl Iterator<Item = char> + '_ {
    raw.chars().map(|ch| {
        if ch == '\0' {
            '\u{fffd}'
        } else {
            ch.to_ascii_lowercase()
        }
    })
}

#[derive(Debug)]
enum Scalars<'a> {
    Named(core::str::Chars<'static>),
    Numeric(Option<char>),
    Literal(core::iter::Peekable<core::str::Chars<'a>>),
}

impl Iterator for Scalars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self {
            Self::Named(value) => value.next(),
            Self::Numeric(value) => value.take(),
            Self::Literal(value) => value.next().map(|ch| match ch {
                '\r' => {
                    if value.peek() == Some(&'\n') {
                        value.next();
                    }
                    '\n'
                }
                '\0' => '\u{fffd}',
                _ => ch,
            }),
        }
    }
}

// Only the private original lexer can populate this range. The completed run
// and enclosing tag producer establish callback order and complete boundaries.
fn read<'s, 'o, 'a>(
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    element: &'s Element,
    index: usize,
) -> Option<DocumentHtmlAttribute<'s, 'o, 'a>> {
    let events = &tree.owner.events;
    let name = events.get(index)?;
    if name.kind() != Kind::AttributeName {
        return None;
    }
    let name_end = events.get(index + 1)?;
    if name_end.kind() != Kind::AttributeNameEnd || name_end.span.start != name.span.end {
        return None;
    }
    let start = index + 2;
    let mut end = start;
    let (quote, coordinate) = loop {
        let event = events.get(end)?;
        match event.kind() {
            Kind::AttributeData | Kind::AttributeEntity if end < element.attributes.end => end += 1,
            Kind::AttributeEnd(quote) if end < element.attributes.end => {
                break (quote, event.span.start);
            }
            _ => return None,
        }
    };
    let value = if quote == QuoteType::NoValue {
        None
    } else {
        let source = tree.owner.source();
        let gap = source.get(name.span.end as usize..coordinate as usize)?;
        let gap = gap.trim_start_matches(['\t', '\n', '\x0c', '\r', ' ']);
        let gap = gap.strip_prefix('=')?;
        let gap = gap.trim_start_matches(['\t', '\n', '\x0c', '\r', ' ']);
        let content = match quote {
            QuoteType::Double => gap.strip_prefix('"')?,
            QuoteType::Single => gap.strip_prefix('\'')?,
            QuoteType::Unquoted => gap,
            QuoteType::NoValue => return None,
        };
        Some(Span::new(coordinate - content.len() as u32, coordinate))
    };
    Some(DocumentHtmlAttribute {
        tree,
        element,
        name: name.span,
        parts: start..end,
        quote,
        value,
    })
}

#[cfg(test)]
mod tests;
