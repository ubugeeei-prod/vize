//! Attributes and their values: the in-tag half of the builder.

use vize_armature::tokenizer::QuoteType;

use super::Builder;
use crate::event::EventKind;
use crate::surface::{AttrValue, Attribute, Token};

impl<'a> Builder<'a, '_> {
    /// `events[i]` is the first `AttrName` piece. Directive pieces all
    /// record as `AttrName`, so the raw name token spans from the first
    /// piece to the `AttrNameEnd` offset — the authored spelling, sigils
    /// and modifiers included.
    pub(super) fn attribute(&mut self) -> Attribute<'a> {
        let first = self.events[self.i];
        let name_start = first.start as usize;
        let mut name_end = first.end as usize;
        self.i += 1;
        loop {
            match self.events.get(self.i) {
                Some(next) if next.kind == EventKind::AttrName => {
                    name_end = next.end as usize;
                    self.i += 1;
                }
                Some(next) if next.kind == EventKind::AttrNameEnd => {
                    name_end = next.start as usize;
                    self.i += 1;
                    break;
                }
                _ => break,
            }
        }
        let name = self.token_at(name_start, name_end);
        // Value pieces (split at entities) until the AttrEnd event.
        let mut data: Option<(usize, usize)> = None;
        loop {
            match self.events.get(self.i) {
                Some(next) if next.kind == EventKind::AttrData => {
                    let (s, e) = (next.start as usize, next.end as usize);
                    data = Some(data.map_or((s, e), |(from, _)| (from, e)));
                    self.i += 1;
                }
                Some(next) if next.kind == EventKind::AttrEnd => {
                    let (quote, endq) = (next.quote(), next.start as usize);
                    self.i += 1;
                    let (eq, value) = self.attr_value(quote, endq, data);
                    return Attribute { name, eq, value };
                }
                _ => {
                    return Attribute {
                        name,
                        eq: None,
                        value: None,
                    };
                }
            }
        }
    }

    /// The `=` between the cursor and `before`, if authored (absent in
    /// the recovered `a"v"` form).
    fn eq_token(&mut self, before: usize) -> Option<Token<'a>> {
        let eq = self.find_byte(b'=', self.cursor, before)?;
        Some(self.token_at(eq, eq + 1))
    }

    fn attr_value(
        &mut self,
        quote: QuoteType,
        endq: usize,
        data: Option<(usize, usize)>,
    ) -> (Option<Token<'a>>, Option<AttrValue<'a>>) {
        match quote {
            QuoteType::NoValue => (None, None),
            QuoteType::Unquoted => match data {
                Some((s, e)) => {
                    let eq = self.eq_token(s);
                    let content = self.token_at(s, e);
                    (eq, Some(AttrValue::unquoted(content)))
                }
                None => {
                    // `a= >`: a value was announced and never written —
                    // the content is a typed `Missing` hole.
                    let Some(eq) = self.eq_token(endq) else {
                        return (None, None);
                    };
                    let content = self.missing_to(self.cursor);
                    (Some(eq), Some(AttrValue::unquoted(content)))
                }
            },
            QuoteType::Single | QuoteType::Double => {
                let qc = if quote == QuoteType::Single {
                    b'\''
                } else {
                    b'"'
                };
                let open_pos = match data {
                    Some((s, _)) => Some(s - 1),
                    None => self.find_byte(qc, self.cursor, self.src.len()),
                };
                let Some(oq) = open_pos else {
                    // Defensive: no quote byte found; treat as valueless.
                    return (None, None);
                };
                let eq = self.eq_token(oq);
                let open_quote = self.token_at(oq, oq + 1);
                let content = match data {
                    Some((s, e)) => self.token_at(s, e),
                    None => self.token_at(self.cursor, self.cursor),
                };
                let close_quote =
                    if endq >= self.cursor && self.src.as_bytes().get(endq) == Some(&qc) {
                        self.token_at(endq, endq + 1)
                    } else {
                        // Unterminated at EOF: a typed `Missing` hole.
                        self.missing_to(self.cursor)
                    };
                (
                    eq,
                    Some(AttrValue {
                        open_quote: Some(open_quote),
                        content,
                        close_quote: Some(close_quote),
                    }),
                )
            }
        }
    }
}
