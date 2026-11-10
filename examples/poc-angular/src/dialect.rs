//! Angular's template dialect, lowered from the L1 surface tree to neutral L2 ops.
//!
//! `{{ expr }}` is a `ui.interpolation`, `[prop]="expr"` a `ui.bind` and
//! `(event)="statement"` a `ui.on`. The built-in control flow blocks in text,
//! `@if (cond) { } @else if (cond) { } @else { }` and
//! `@for (item of list; track item) { }`, become `ui.if` and `ui.for`.
//! Structural directives (`*ngIf`), references and two-way bindings are refused.

use poc_support::{Error, span_in};
use vize_l0::{Allocator, Box, Span, String, Vec, cstr};
use vize_l1::markup::{Attribute as SurfaceAttribute, Element, ElementClose, SurfaceChild};
use vize_l2::expr::ExprRef;
use vize_l2::op::{
    Attribute, BindOp, BindingOp, DynamicName, ElementOp, ForBinding, ForOp, IfBranch, IfOp,
    InterpolationOp, Namespace, OnOp, Op, Region, TextOp,
};

enum Block<'a> {
    If(&'a str),
    For {
        item: &'a str,
        list: &'a str,
        track: &'a str,
    },
    Else(Option<&'a str>),
}

enum Item<'a> {
    Op(Op<'a>),
    Text(&'a str),
    Open(Block<'a>, Span),
    Close(Span),
}

enum Built<'a> {
    Op(Op<'a>),
    Text(&'a str),
}

type Items<'a> = std::iter::Peekable<std::vec::IntoIter<Item<'a>>>;

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
        let mut items = std::vec::Vec::new();
        for child in children {
            self.items(child, &mut items)?;
        }
        match self.build(&mut items.into_iter().peekable())? {
            (built, None) => Ok(self.settle(built)),
            (_, Some(close)) => Err(Error::new("unmatched `}`", close)),
        }
    }

    fn items(
        &self,
        child: &SurfaceChild<'a>,
        items: &mut std::vec::Vec<Item<'a>>,
    ) -> Result<(), Error> {
        match child {
            SurfaceChild::Element(element) => items.push(Item::Op(self.element(element)?)),
            SurfaceChild::Text(text) => self.split(text.text, items)?,
            SurfaceChild::Interpolation(interpolation) => {
                let open = span_in(self.source, interpolation.open.text);
                let close = span_in(self.source, interpolation.close.text);
                items.push(Item::Op(Op::Interpolation(Box::new_in(
                    InterpolationOp {
                        expression: self.expression(interpolation.content.text),
                        raw_html: false,
                        span: Span::new(open.start, close.end),
                    },
                    &self.allocator,
                ))));
            }
            SurfaceChild::Comment(_) => {}
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

    /// Split text around `@if`, `@for`, `@else` block heads and closing `}`.
    fn split(&self, text: &'a str, items: &mut std::vec::Vec<Item<'a>>) -> Result<(), Error> {
        let mut rest = text;
        while let Some(at) = rest.find(['@', '}']) {
            let (before, tail) = rest.split_at_checked(at).unwrap_or((rest, ""));
            if !before.is_empty() {
                items.push(Item::Text(before));
            }
            if let Some(after) = tail.strip_prefix('}') {
                items.push(Item::Close(span_in(
                    self.source,
                    tail.get(..1).unwrap_or_default(),
                )));
                rest = after;
                continue;
            }
            let (block, after) = self.block_head(tail)?;
            let head = tail.get(..tail.len() - after.len()).unwrap_or_default();
            items.push(Item::Open(block, span_in(self.source, head)));
            rest = after;
        }
        if !rest.is_empty() {
            items.push(Item::Text(rest));
        }
        Ok(())
    }

    /// Parse `@if (..) {`, `@for (..) {`, `@else {` or `@else if (..) {`
    /// at the start of `text`; returns the block and the text after its `{`.
    fn block_head(&self, text: &'a str) -> Result<(Block<'a>, &'a str), Error> {
        let fail = || {
            Error::new(
                "unsupported `@` block",
                span_in(self.source, text.get(..1).unwrap_or_default()),
            )
        };
        let (keyword, rest) = if let Some(rest) = text.strip_prefix("@if") {
            ("if", rest)
        } else if let Some(rest) = text.strip_prefix("@for") {
            ("for", rest)
        } else if let Some(rest) = text.strip_prefix("@else") {
            match rest.trim_start().strip_prefix("if") {
                Some(rest) => ("else if", rest),
                None => ("else", rest),
            }
        } else {
            return Err(fail());
        };
        let rest = rest.trim_start();
        let (head, rest) = if keyword == "else" {
            ("", rest)
        } else {
            let body = rest.strip_prefix('(').ok_or_else(fail)?;
            let close = matching_paren(body).ok_or_else(fail)?;
            let (head, after) = body.split_at_checked(close).unwrap_or((body, ""));
            (head, after.get(1..).unwrap_or_default())
        };
        let after = rest.trim_start().strip_prefix('{').ok_or_else(fail)?;
        let block = match keyword {
            "if" => Block::If(head),
            "else if" => Block::Else(Some(head)),
            "else" => Block::Else(None),
            _ => {
                let (binding, track) = head.split_once(';').ok_or_else(fail)?;
                let (item, list) = binding.split_once(" of ").ok_or_else(fail)?;
                let track = track.trim().strip_prefix("track ").ok_or_else(fail)?.trim();
                Block::For {
                    item: item.trim(),
                    list,
                    track,
                }
            }
        };
        Ok((block, after))
    }

    /// Build ops until a closing `}`, which is returned.
    fn build(
        &self,
        items: &mut Items<'a>,
    ) -> Result<(std::vec::Vec<Built<'a>>, Option<Span>), Error> {
        let mut built = std::vec::Vec::new();
        while let Some(item) = items.next() {
            match item {
                Item::Op(op) => built.push(Built::Op(op)),
                Item::Text(text) => built.push(Built::Text(text)),
                Item::Close(span) => return Ok((built, Some(span))),
                Item::Open(Block::If(condition), span) => {
                    built.push(Built::Op(self.if_block(condition, span, items)?));
                }
                Item::Open(Block::For { item, list, track }, span) => {
                    if track != item {
                        return Err(Error::new("only `track` by the item itself", span));
                    }
                    let (region, close) = self.build(items)?;
                    let close = close.ok_or_else(|| Error::new("unclosed @for", span))?;
                    built.push(Built::Op(Op::For(Box::new_in(
                        ForOp {
                            binding: ForBinding {
                                source: self.expression(list),
                                value: self.expression(item),
                                key: Some(self.expression(track)),
                                index: None,
                            },
                            region: self.settle(region),
                            host: None,
                            span: Span::new(span.start, close.end),
                        },
                        &self.allocator,
                    ))));
                }
                Item::Open(Block::Else(_), span) => {
                    return Err(Error::new("@else without @if", span));
                }
            }
        }
        Ok((built, None))
    }

    fn if_block(
        &self,
        condition: &'a str,
        open: Span,
        items: &mut Items<'a>,
    ) -> Result<Op<'a>, Error> {
        let mut branches = Vec::new_in(&self.allocator);
        let mut condition = Some(self.expression(condition));
        let mut start = open.start;
        loop {
            let (built, close) = self.build(items)?;
            let close = close.ok_or_else(|| Error::new("unclosed @if", open))?;
            let had_condition = condition.is_some();
            branches.push(IfBranch {
                condition,
                region: self.settle(built),
                span: Span::new(start, close.end),
            });
            while matches!(items.peek(), Some(Item::Text(text)) if text.trim().is_empty()) {
                items.next();
            }
            let Some(Item::Open(Block::Else(next), span)) = items.peek() else {
                break;
            };
            if !had_condition {
                return Err(Error::new("@else after @else", *span));
            }
            start = span.start;
            condition = next.map(|next| self.expression(next));
            items.next();
        }
        let end = branches.last().map_or(open.end, |branch| branch.span.end);
        Ok(Op::If(Box::new_in(
            IfOp {
                branches,
                span: Span::new(open.start, end),
            },
            &self.allocator,
        )))
    }

    /// Angular drops whitespace-only text, collapses runs to one space and
    /// trims a region's ends.
    fn settle(&self, built: std::vec::Vec<Built<'a>>) -> Region<'a> {
        let built: std::vec::Vec<Built<'a>> = built
            .into_iter()
            .filter(|item| !matches!(item, Built::Text(text) if text.trim().is_empty()))
            .collect();
        let last = built.len().saturating_sub(1);
        let mut ops = Vec::new_in(&self.allocator);
        for (index, item) in built.into_iter().enumerate() {
            let raw = match item {
                Built::Op(op) => {
                    ops.push(op);
                    continue;
                }
                Built::Text(raw) => raw,
            };
            let space = |ch: char| ch.is_ascii_whitespace();
            let mut text = String::default();
            if raw.starts_with(space) && index > 0 {
                text.push(' ');
            }
            text.push_str(
                &raw.split_ascii_whitespace()
                    .collect::<std::vec::Vec<_>>()
                    .join(" "),
            );
            if raw.ends_with(space) && index < last {
                text.push(' ');
            }
            ops.push(Op::Text(Box::new_in(
                TextOp {
                    content: self.allocator.alloc_str(&text),
                    span: span_in(self.source, raw),
                },
                &self.allocator,
            )));
        }
        Region { ops }
    }

    fn element(&self, element: &Element<'a>) -> Result<Op<'a>, Error> {
        let open = span_in(self.source, element.open.lt_name.text);
        let tag = element.tag();
        if tag.contains(['-', ':']) || tag.starts_with(|ch: char| ch.is_ascii_uppercase()) {
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
        let value = attribute.value.as_ref().map(|value| value.content.text);
        let end = attribute
            .value
            .as_ref()
            .map_or(span_in(self.source, name).end, |value| {
                let last = value.close_quote.as_ref().unwrap_or(&value.content);
                span_in(self.source, last.text).end
            });
        let span = Span::new(span_in(self.source, name).start, end);
        let modifiers = Vec::new_in(&self.allocator);
        let property = name
            .strip_prefix('[')
            .and_then(|name| name.strip_suffix(']'));
        let event = name
            .strip_prefix('(')
            .and_then(|name| name.strip_suffix(')'));
        match (property, event, value) {
            (Some(property), None, Some(value)) if !property.starts_with('(') => {
                bindings.push(BindingOp::Bind(Box::new_in(
                    BindOp {
                        name: Some(DynamicName::Static(property)),
                        modifiers,
                        value: Some(self.expression(value)),
                        span,
                    },
                    &self.allocator,
                )));
            }
            (None, Some(event), Some(value)) => {
                bindings.push(BindingOp::On(Box::new_in(
                    OnOp {
                        name: Some(DynamicName::Static(event)),
                        modifiers,
                        handler: Some(self.expression(value).into()),
                        span,
                    },
                    &self.allocator,
                )));
            }
            (None, None, _) if !name.starts_with(['*', '#', '[', '(']) => {
                attributes.push(Attribute { name, value, span });
            }
            _ => return Err(Error::new(cstr!("unsupported attribute {name}"), span)),
        }
        Ok(())
    }

    fn expression(&self, text: &'a str) -> ExprRef<'a> {
        let text = text.trim();
        ExprRef::parse_js_in(self.allocator, text, span_in(self.source, text))
    }
}

/// The offset of the `)` that closes the text before it.
fn matching_paren(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(offset),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}
