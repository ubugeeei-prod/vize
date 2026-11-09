//! Astro's template dialect, lowered from the L1 surface tree to neutral L2 ops.
//!
//! `{expr}` in text is a `ui.interpolation` and an attribute value that is one
//! `{expr}` is a `ui.bind`. Astro embeds markup in expressions; this proof of
//! concept legalizes the two common idioms into structured control flow:
//! `{cond && <el/>}` becomes `ui.if` and `{list.map((item) => <el/>)}` becomes
//! `ui.for`. Any other embedded markup is refused.

use poc_support::{Error, span_in};
use vize_l0::{Allocator, Box, Span, Vec, cstr};
use vize_l1::markup::{Attribute as SurfaceAttribute, Element, ElementClose, SurfaceChild};
use vize_l2::expr::ExprRef;
use vize_l2::op::{
    Attribute, BindOp, BindingOp, DynamicName, ElementOp, ForBinding, ForOp, IfBranch, IfOp,
    InterpolationOp, Namespace, Op, Region, TextOp,
};

/// One piece of the children sequence.
enum Piece<'a> {
    Op(Op<'a>),
    Text(&'a str),
    /// A balanced `{expr}`.
    Expression(&'a str, Span),
    /// A `{` whose expression continues past the following markup.
    Open(&'a str, Span),
    /// The `}` closing the innermost [`Piece::Open`], with the text before it.
    Close(&'a str, Span),
}

type Pieces<'a> = std::vec::IntoIter<Piece<'a>>;

/// A built region and, inside an embedding, the text and span of its closing `}`.
type Built<'a> = (Region<'a>, Option<(&'a str, Span)>);

pub(crate) struct Lowering<'a> {
    allocator: &'a Allocator,
    source: &'a str,
}

impl<'a> Lowering<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { allocator, source }
    }

    pub(crate) fn region<'c>(
        &self,
        children: impl Iterator<Item = &'c SurfaceChild<'a>>,
    ) -> Result<Region<'a>, Error>
    where
        'a: 'c,
    {
        let mut pieces = std::vec::Vec::new();
        for child in children {
            self.pieces(child, &mut pieces)?;
        }
        match self.build(&mut pieces.into_iter())? {
            (region, None) => Ok(region),
            (_, Some((_, span))) => Err(Error::new("unmatched `}`", span)),
        }
    }

    fn pieces(
        &self,
        child: &SurfaceChild<'a>,
        pieces: &mut std::vec::Vec<Piece<'a>>,
    ) -> Result<(), Error> {
        match child {
            SurfaceChild::Element(element) => pieces.push(Piece::Op(self.element(element)?)),
            SurfaceChild::Text(text) => split(self.source, text.text, pieces),
            SurfaceChild::Comment(_) => {}
            SurfaceChild::Interpolation(interpolation) => {
                return Err(Error::new(
                    "unsupported `{{` text",
                    span_in(self.source, interpolation.open.text),
                ));
            }
            SurfaceChild::Cdata(token)
            | SurfaceChild::ProcessingInstruction(token)
            | SurfaceChild::Unexpected(token) => {
                return Err(Error::new(
                    "unsupported markup",
                    span_in(self.source, token.text),
                ));
            }
        }
        Ok(())
    }

    /// Build a region until the `}` that closes an enclosing embedding.
    fn build(&self, pieces: &mut Pieces<'a>) -> Result<Built<'a>, Error> {
        let mut ops = Vec::new_in(&self.allocator);
        while let Some(piece) = pieces.next() {
            match piece {
                Piece::Op(op) => ops.push(op),
                Piece::Text(text) => {
                    if !(text.trim().is_empty() && text.contains('\n')) {
                        ops.push(Op::Text(Box::new_in(
                            TextOp {
                                content: text,
                                span: span_in(self.source, text),
                            },
                            &self.allocator,
                        )));
                    }
                }
                Piece::Expression(text, span) => ops.push(Op::Interpolation(Box::new_in(
                    InterpolationOp {
                        expression: self.expression(text),
                        raw_html: false,
                        span,
                    },
                    &self.allocator,
                ))),
                Piece::Open(head, open) => {
                    let (region, close) = self.build(pieces)?;
                    let (tail, close) = close.ok_or_else(|| Error::new("unclosed `{`", open))?;
                    let span = Span::new(open.start, close.end);
                    ops.push(self.legalize(head, tail, region, span)?);
                }
                Piece::Close(tail, span) => return Ok((Region { ops }, Some((tail, span)))),
            }
        }
        Ok((Region { ops }, None))
    }

    /// `cond && <el/>` -> `ui.if`; `list.map((item) => <el/>)` -> `ui.for`.
    fn legalize(
        &self,
        head: &'a str,
        tail: &str,
        region: Region<'a>,
        span: Span,
    ) -> Result<Op<'a>, Error> {
        let head = head.trim();
        if let (Some(condition), "") = (head.strip_suffix("&&"), tail.trim()) {
            let mut branches = Vec::new_in(&self.allocator);
            branches.push(IfBranch {
                condition: Some(self.expression(condition)),
                region,
                span,
            });
            return Ok(Op::If(Box::new_in(
                IfOp { branches, span },
                &self.allocator,
            )));
        }
        let mapped = head
            .strip_suffix("=>")
            .and_then(|call| call.trim_end().rsplit_once(".map("))
            .map(|(list, item)| {
                (
                    list,
                    item.trim().trim_start_matches('(').trim_end_matches(')'),
                )
            })
            .filter(|_| tail.trim() == ")")
            .filter(|(_, item)| {
                !item.is_empty()
                    && item
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            });
        let Some((list, item)) = mapped else {
            return Err(Error::new("unsupported embedded markup", span));
        };
        Ok(Op::For(Box::new_in(
            ForOp {
                binding: ForBinding {
                    source: self.expression(list),
                    value: self.expression(item),
                    key: None,
                    index: None,
                },
                region,
                span,
            },
            &self.allocator,
        )))
    }

    fn element(&self, element: &Element<'a>) -> Result<Op<'a>, Error> {
        let open = span_in(self.source, element.open.lt_name.text);
        let tag = element.tag();
        if tag.contains(':') || tag.starts_with(|ch: char| ch.is_ascii_uppercase()) {
            return Err(Error::new(cstr!("unsupported <{tag}>"), open));
        }
        let end = match &element.close {
            ElementClose::Present(close) => span_in(self.source, close.gt.text).end,
            ElementClose::NotExpected => span_in(self.source, element.open.gt.text).end,
            ElementClose::Implicit | ElementClose::Missing => {
                return Err(Error::new(cstr!("unclosed <{tag}>"), open));
            }
        };
        let mut attributes = Vec::new_in(&self.allocator);
        let mut bindings = Vec::new_in(&self.allocator);
        for attribute in &element.open.attrs {
            self.attribute(attribute, &mut attributes, &mut bindings)?;
        }
        Ok(Op::Element(Box::new_in(
            ElementOp {
                tag,
                namespace: Namespace::Html,
                attributes,
                bindings,
                children: self.region(element.children.iter())?,
                span: Span::new(open.start, end),
            },
            &self.allocator,
        )))
    }

    fn attribute(
        &self,
        attribute: &SurfaceAttribute<'a>,
        attributes: &mut Vec<'a, Attribute<'a>>,
        bindings: &mut Vec<'a, BindingOp<'a>>,
    ) -> Result<(), Error> {
        let name = attribute.name.text;
        let end = attribute
            .value
            .as_ref()
            .map_or(span_in(self.source, name).end, |value| {
                let last = value.close_quote.as_ref().unwrap_or(&value.content);
                span_in(self.source, last.text).end
            });
        let span = Span::new(span_in(self.source, name).start, end);
        if name.contains(':') {
            return Err(Error::new(cstr!("unsupported directive {name}"), span));
        }
        let value = attribute.value.as_ref().map(|value| value.content.text);
        let expression = value
            .and_then(|value| value.strip_prefix('{'))
            .and_then(|value| value.strip_suffix('}'));
        match expression {
            Some(expression) => bindings.push(BindingOp::Bind(Box::new_in(
                BindOp {
                    name: Some(DynamicName::Static(name)),
                    modifiers: Vec::new_in(&self.allocator),
                    value: Some(self.expression(expression)),
                    span,
                },
                &self.allocator,
            ))),
            None if value.is_some_and(|value| value.contains('{')) => {
                return Err(Error::new(
                    "unsupported attribute text with expressions",
                    span,
                ));
            }
            None => attributes.push(Attribute { name, value, span }),
        }
        Ok(())
    }

    fn expression(&self, text: &'a str) -> ExprRef<'a> {
        let text = text.trim();
        ExprRef::parse_js_in(self.allocator, text, span_in(self.source, text))
    }
}

/// Split one text node into text, balanced `{expr}`, and the opening or
/// closing halves of expressions that embed markup.
fn split<'a>(source: &'a str, text: &'a str, pieces: &mut std::vec::Vec<Piece<'a>>) {
    let mut start = 0;
    let mut open: Option<usize> = None;
    let mut depth = 0usize;
    for (offset, ch) in text.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    push_text(text, start, offset, pieces);
                    open = Some(offset);
                }
                depth += 1;
            }
            '}' if depth == 0 => {
                let tail = text.get(start..offset).unwrap_or_default();
                let close = text.get(offset..=offset).unwrap_or_default();
                pieces.push(Piece::Close(tail, span_in(source, close)));
                start = offset + 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0
                    && let Some(at) = open.take()
                {
                    let whole = text.get(at..=offset).unwrap_or_default();
                    let body = whole.get(1..whole.len() - 1).unwrap_or_default();
                    pieces.push(Piece::Expression(body, span_in(source, whole)));
                    start = offset + 1;
                }
            }
            _ => {}
        }
    }
    match open {
        Some(at) if depth > 0 => {
            let head = text.get(at + 1..).unwrap_or_default();
            let brace = text.get(at..=at).unwrap_or_default();
            pieces.push(Piece::Open(head, span_in(source, brace)));
        }
        _ => push_text(text, start, text.len(), pieces),
    }
}

fn push_text<'a>(text: &'a str, start: usize, end: usize, pieces: &mut std::vec::Vec<Piece<'a>>) {
    if let Some(slice) = text.get(start..end).filter(|slice| !slice.is_empty()) {
        pieces.push(Piece::Text(slice));
    }
}
