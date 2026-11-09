//! Svelte's template dialect, lowered from the L1 surface tree to neutral L2 ops.
//!
//! Text carries Svelte's tags: `{expr}`, `{@html expr}`, `{#if}` / `{:else if}` /
//! `{:else}` / `{/if}` and `{#each list as item}` / `{/each}`. They become
//! `ui.interpolation`, `ui.if` and `ui.for`. An attribute value that is one
//! `{expr}` is a binding: `on:event` and lowercase `onevent` are `ui.on`,
//! `bind:prop` is the two-way `ui.model` contract, anything else is `ui.bind`.

use poc_support::{Error, span_in};
use vize_l0::{Allocator, Box, Span, String, Vec, cstr};
use vize_l1::markup::{
    Attribute as SurfaceAttribute, Element, ElementClose, SurfaceChild, SurfaceTree,
};
use vize_l2::expr::ExprRef;
use vize_l2::op::{
    Attribute, BindOp, BindingContract, BindingOp, DynamicName, ElementOp, ForBinding, ForOp,
    IfBranch, IfOp, InterpolationOp, ModelOp, Namespace, OnOp, Op, Region, TextOp,
};

/// A `{...}` tag: its trimmed body and its whole authored span.
#[derive(Clone, Copy)]
struct Tag<'a> {
    body: &'a str,
    span: Span,
}

enum Item<'a> {
    Op(Op<'a>),
    Text(&'a str),
    Tag(Tag<'a>),
}

/// Ops of one region before text whitespace is settled.
enum Built<'a> {
    Op(Op<'a>),
    Text(&'a str),
}

type Items<'a> = std::vec::IntoIter<Item<'a>>;

pub(crate) struct Lowering<'a> {
    allocator: &'a Allocator,
    source: &'a str,
}

impl<'a> Lowering<'a> {
    pub(crate) fn new(allocator: &'a Allocator, source: &'a str) -> Self {
        Self { allocator, source }
    }

    /// The markup: every root child except the container's script and style blocks.
    pub(crate) fn root(&self, tree: &SurfaceTree<'a>) -> Result<Region<'a>, Error> {
        self.region(tree.children.iter().filter(|child| {
            !matches!(child, SurfaceChild::Element(element) if matches!(element.tag(), "script" | "style"))
        }))
    }

    fn region<'c>(
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
        let mut items = items.into_iter();
        let (built, end) = self.build(&mut items)?;
        match end {
            Some(tag) => Err(Error::new(cstr!("unexpected {{{}}}", tag.body), tag.span)),
            None => Ok(self.settle(built)),
        }
    }

    fn items(
        &self,
        child: &SurfaceChild<'a>,
        items: &mut std::vec::Vec<Item<'a>>,
    ) -> Result<(), Error> {
        match child {
            SurfaceChild::Element(element) => items.push(Item::Op(self.element(element)?)),
            SurfaceChild::Text(text) => {
                if text.text.contains('&') {
                    return Err(Error::new(
                        "unsupported character reference",
                        span_in(self.source, text.text),
                    ));
                }
                for piece in split_tags(self.source, text.text)? {
                    items.push(match piece {
                        Piece::Text(text) => Item::Text(text),
                        Piece::Tag(tag) => Item::Tag(tag),
                    });
                }
            }
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

    /// Build ops until a `{:...}` or `{/...}` tag, which is returned unconsumed.
    fn build(
        &self,
        items: &mut Items<'a>,
    ) -> Result<(std::vec::Vec<Built<'a>>, Option<Tag<'a>>), Error> {
        let mut built = std::vec::Vec::new();
        while let Some(item) = items.next() {
            match item {
                Item::Op(op) => built.push(Built::Op(op)),
                Item::Text(text) => built.push(Built::Text(text)),
                Item::Tag(tag) => {
                    if let Some(condition) = tag.body.strip_prefix("#if ") {
                        built.push(Built::Op(self.if_block(tag, condition, items)?));
                    } else if let Some(head) = tag.body.strip_prefix("#each ") {
                        built.push(Built::Op(self.each_block(tag, head, items)?));
                    } else if let Some(html) = tag.body.strip_prefix("@html ") {
                        built.push(Built::Op(self.interpolation(html, tag, true)));
                    } else if tag.body.starts_with([':', '/']) {
                        return Ok((built, Some(tag)));
                    } else if tag.body.starts_with(['#', '@']) {
                        return Err(Error::new(cstr!("unsupported {{{}}}", tag.body), tag.span));
                    } else {
                        built.push(Built::Op(self.interpolation(tag.body, tag, false)));
                    }
                }
            }
        }
        Ok((built, None))
    }

    /// Settle Svelte text whitespace: a whitespace run is one space, text
    /// that is only a line break disappears, and a region trims its ends.
    fn settle(&self, built: std::vec::Vec<Built<'a>>) -> Region<'a> {
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
            if raw.trim().is_empty() && raw.contains('\n') {
                continue;
            }
            let words: std::vec::Vec<&str> = raw.split_ascii_whitespace().collect();
            let space = |ch: char| ch.is_ascii_whitespace();
            let mut text = String::default();
            if raw.starts_with(space) && index > 0 {
                text.push(' ');
            }
            text.push_str(&words.join(" "));
            if raw.ends_with(space) && index < last && !words.is_empty() {
                text.push(' ');
            }
            if text.trim().is_empty() && (index == 0 || index == last) {
                continue;
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

    fn interpolation(&self, text: &'a str, tag: Tag<'a>, raw_html: bool) -> Op<'a> {
        Op::Interpolation(Box::new_in(
            InterpolationOp {
                expression: self.expression(text),
                raw_html,
                span: tag.span,
            },
            &self.allocator,
        ))
    }

    fn if_block(
        &self,
        open: Tag<'a>,
        condition: &'a str,
        items: &mut Items<'a>,
    ) -> Result<Op<'a>, Error> {
        let mut branches = Vec::new_in(&self.allocator);
        let mut condition = Some(self.expression(condition));
        let mut start = open.span.start;
        loop {
            let (built, end) = self.build(items)?;
            let end = end.ok_or_else(|| Error::new("unclosed {#if}", open.span))?;
            let branch_open = condition.is_some();
            branches.push(IfBranch {
                condition,
                region: self.settle(built),
                span: Span::new(start, end.span.start),
            });
            start = end.span.start;
            condition = match end.body {
                "/if" => {
                    return Ok(Op::If(Box::new_in(
                        IfOp {
                            branches,
                            span: Span::new(open.span.start, end.span.end),
                        },
                        &self.allocator,
                    )));
                }
                ":else" if branch_open => None,
                body => match body.strip_prefix(":else if ") {
                    Some(next) if branch_open => Some(self.expression(next)),
                    _ => return Err(Error::new(cstr!("unexpected {{{body}}}"), end.span)),
                },
            };
        }
    }

    fn each_block(
        &self,
        open: Tag<'a>,
        head: &'a str,
        items: &mut Items<'a>,
    ) -> Result<Op<'a>, Error> {
        let (list, item) = head
            .split_once(" as ")
            .filter(|(_, item)| {
                item.trim()
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            })
            .ok_or_else(|| Error::new("unsupported {#each} head", open.span))?;
        let (built, end) = self.build(items)?;
        match end {
            Some(end) if end.body == "/each" => Ok(Op::For(Box::new_in(
                ForOp {
                    binding: ForBinding {
                        source: self.expression(list),
                        value: self.expression(item),
                        key: None,
                        index: None,
                    },
                    region: self.settle(built),
                    span: Span::new(open.span.start, end.span.end),
                },
                &self.allocator,
            ))),
            Some(end) => Err(Error::new(cstr!("unexpected {{{}}}", end.body), end.span)),
            None => Err(Error::new("unclosed {#each}", open.span)),
        }
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
        let start = span_in(self.source, name).start;
        let Some(value) = &attribute.value else {
            attributes.push(Attribute {
                name,
                value: None,
                span: span_in(self.source, name),
            });
            return Ok(());
        };
        let content = value.content.text;
        let end = value
            .close_quote
            .as_ref()
            .map_or(span_in(self.source, content).end, |quote| {
                span_in(self.source, quote.text).end
            });
        let span = Span::new(start, end);
        let expression = match split_tags(self.source, content)?.as_slice() {
            [Piece::Tag(tag)] => self.expression(tag.body),
            pieces if pieces.iter().all(|piece| matches!(piece, Piece::Text(_))) => {
                attributes.push(Attribute {
                    name,
                    value: Some(content),
                    span,
                });
                return Ok(());
            }
            _ => return Err(Error::new("unsupported attribute text with tags", span)),
        };
        let modifiers = Vec::new_in(&self.allocator);
        let event = name.strip_prefix("on:").or_else(|| {
            name.strip_prefix("on").filter(|event| {
                !event.is_empty() && event.bytes().all(|byte| byte.is_ascii_lowercase())
            })
        });
        let binding = if let Some(event) = event {
            BindingOp::On(Box::new_in(
                OnOp {
                    name: Some(DynamicName::Static(event)),
                    modifiers,
                    handler: Some(expression.into()),
                    span,
                },
                &self.allocator,
            ))
        } else if let Some(property) = name.strip_prefix("bind:") {
            BindingOp::Model(Box::new_in(
                ModelOp {
                    contract: BindingContract {
                        read: expression,
                        write: expression,
                    },
                    argument: Some(DynamicName::Static(property)),
                    attributes: Vec::new_in(&self.allocator),
                    span,
                },
                &self.allocator,
            ))
        } else if name.contains(':') {
            return Err(Error::new(cstr!("unsupported directive {name}"), span));
        } else {
            BindingOp::Bind(Box::new_in(
                BindOp {
                    name: Some(DynamicName::Static(name)),
                    modifiers,
                    value: Some(expression),
                    span,
                },
                &self.allocator,
            ))
        };
        bindings.push(binding);
        Ok(())
    }

    fn expression(&self, text: &'a str) -> ExprRef<'a> {
        let text = text.trim();
        ExprRef::parse_js_in(self.allocator, text, span_in(self.source, text))
    }
}

enum Piece<'a> {
    Text(&'a str),
    Tag(Tag<'a>),
}

/// Split `text` (a slice of `source`) into plain text and `{...}` tags.
/// Braces nest, and braces inside string literals do not count.
fn split_tags<'a>(source: &'a str, text: &'a str) -> Result<std::vec::Vec<Piece<'a>>, Error> {
    let mut pieces = std::vec::Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let (before, tail) = rest.split_at_checked(open).unwrap_or((rest, ""));
        if !before.is_empty() {
            pieces.push(Piece::Text(before));
        }
        let close = matching_brace(tail)
            .ok_or_else(|| Error::new("unclosed `{`", span_in(source, tail)))?;
        let (whole, after) = tail.split_at_checked(close + 1).unwrap_or((tail, ""));
        let body = whole.get(1..close).unwrap_or_default().trim();
        pieces.push(Piece::Tag(Tag {
            body,
            span: span_in(source, whole),
        }));
        rest = after;
    }
    if !rest.is_empty() {
        pieces.push(Piece::Text(rest));
    }
    Ok(pieces)
}

/// The offset of the `}` that closes the `{` at offset 0.
fn matching_brace(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (offset, ch) in text.char_indices() {
        match (quote, ch) {
            (Some(_), _) if escaped => escaped = false,
            (Some(_), '\\') => escaped = true,
            (Some(open), ch) if ch == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'' | '`') => quote = Some(ch),
            (None, '{') => depth += 1,
            (None, '}') => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(offset);
                }
            }
            (None, _) => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Piece, matching_brace, split_tags};

    #[test]
    fn tags_nest_and_ignore_braces_in_strings() {
        assert_eq!(matching_brace("{a ? '}' : {b: 1}}"), Some(17));
        let source = "Clicked {count} times {#if x}";
        let pieces = split_tags(source, source).unwrap_or_default();
        let shape: Vec<&str> = pieces
            .iter()
            .map(|piece| match piece {
                Piece::Text(text) => text,
                Piece::Tag(tag) => tag.body,
            })
            .collect();
        assert_eq!(shape, ["Clicked ", "count", " times ", "#if x"]);
    }
}
