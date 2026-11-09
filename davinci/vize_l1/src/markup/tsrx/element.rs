//! Elements, fragments, attributes, text, comments and expression children.

use oxc_ast::ast::Expression;
use oxc_parser::{Kind, MarkupSite};
use vize_l0::{Box, Span, Vec};

use super::diagnostic as error;
use super::grammar::{
    Scan, find, is_identifier_part, is_identifier_start, line_end, opens, skip_trivia, span_of,
};
use super::tree::{
    Attribute, AttributeValue, Child, Comment, CommentKind, Element, ExpressionChild, Fragment,
    Node, TagName,
};

impl<'a> Scan<'_, '_, 'a> {
    /// An element or fragment whose `<` is at `start`.
    pub(super) fn markup(&mut self, start: u32) -> Option<Node<'a>> {
        if self.byte(start + 1) == Some(b'>') {
            let (children, closing) = self.children(start + 2, None)?;
            let span = Span::new(start, closing.end);
            return Some(Node::Fragment(Box::new_in(
                Fragment { span, children },
                &self.alloc,
            )));
        }
        let (name, mut offset) = self.tag_name(start + 1)?;
        let mut attributes = Vec::new_in(&self.alloc);
        let self_closing = loop {
            offset = skip_trivia(self.bytes(), offset);
            match self.byte(offset) {
                Some(b'/') if self.byte(offset + 1) == Some(b'>') => break true,
                Some(b'>') => break false,
                Some(b'{') => {
                    let attribute = self.braced_attribute(offset)?;
                    offset = attribute_end(&attribute);
                    attributes.push(attribute);
                }
                Some(byte) if is_identifier_start(byte) => {
                    let attribute = self.named_attribute(offset)?;
                    offset = attribute_end(&attribute);
                    attributes.push(attribute);
                }
                None => {
                    let name = self.text(name_span(&name));
                    return self.fail(error::unclosed_tag(name, Span::new(offset, offset)));
                }
                Some(_) => return self.fail(error::unexpected(Span::new(offset, offset + 1))),
            }
        };
        if self_closing {
            let span = Span::new(start, offset + 2);
            let element = Element {
                span,
                name,
                attributes,
                children: Vec::new_in(&self.alloc),
                raw_text: None,
                closing: None,
            };
            return Some(Node::Element(Box::new_in(element, &self.alloc)));
        }
        let body = offset + 1;
        let (children, raw_text, closing) = match self.raw_text_kind(&name, body) {
            Some(tag) => {
                let (raw, closing) = self.raw_text(tag, body)?;
                (Vec::new_in(&self.alloc), Some(raw), closing)
            }
            None => {
                let (children, closing) = self.children(body, Some(&name))?;
                (children, None, closing)
            }
        };
        let span = Span::new(start, closing.end);
        let element = Element {
            span,
            name,
            attributes,
            children,
            raw_text,
            closing: Some(closing),
        };
        Some(Node::Element(Box::new_in(element, &self.alloc)))
    }

    fn tag_name(&mut self, offset: u32) -> Option<(TagName<'a>, u32)> {
        match self.byte(offset) {
            Some(b'{') => {
                let (expression, end) = self.braced_expression(offset)?;
                if !is_dynamic_tag(&expression) {
                    return self.fail(error::dynamic_tag_expression(span_of(&expression)));
                }
                Some((
                    TagName::Dynamic {
                        span: Span::new(offset, end),
                        expression,
                    },
                    end,
                ))
            }
            Some(byte) if is_identifier_start(byte) => {
                let bytes = self.bytes();
                let len = bytes.get(offset as usize..).map_or(0, |rest| {
                    rest.iter()
                        .take_while(|byte| {
                            is_identifier_part(**byte) || matches!(byte, b'-' | b'.')
                        })
                        .count()
                });
                let end = offset + len as u32;
                if self.byte(end) == Some(b':') {
                    let end = offset
                        + self.bytes().get(offset as usize..).map_or(0, |rest| {
                            rest.iter()
                                .take_while(|byte| {
                                    is_identifier_part(**byte) || matches!(byte, b'-' | b':')
                                })
                                .count() as u32
                        });
                    let name = self.text(Span::new(offset, end));
                    return self.fail(error::namespaced_element(name, Span::new(offset - 1, end)));
                }
                if self.byte(end) == Some(b'<') {
                    // Type arguments on a component tag are not supported yet.
                    return self.fail(error::unexpected(Span::new(end, end + 1)));
                }
                Some((TagName::Static(Span::new(offset, end)), end))
            }
            _ => self.fail(error::unexpected(Span::new(offset, offset + 1))),
        }
    }

    fn braced_attribute(&mut self, open: u32) -> Option<Attribute<'a>> {
        self.seat(open + 1)?;
        if self.token().0 == Kind::Dot3 {
            self.cursor.bump();
            let expression = self.cursor.expression();
            if self.cursor.failed() {
                return None;
            }
            let close = self.expect(Kind::RCurly)?;
            return Some(Attribute::Spread {
                span: Span::new(open, close.end),
                expression,
            });
        }
        let expression = self.cursor.expression();
        if self.cursor.failed() {
            return None;
        }
        if !matches!(expression, Expression::Identifier(_)) {
            return self.fail(error::expected("identifier", span_of(&expression)));
        }
        let close = self.expect(Kind::RCurly)?;
        Some(Attribute::Shorthand {
            span: Span::new(open, close.end),
            expression,
        })
    }

    fn named_attribute(&mut self, start: u32) -> Option<Attribute<'a>> {
        let len = self.bytes().get(start as usize..).map_or(0, |rest| {
            rest.iter()
                .take_while(|byte| is_identifier_part(**byte) || matches!(byte, b'-' | b':' | b'.'))
                .count()
        });
        let name = Span::new(start, start + len as u32);
        let equals = skip_trivia(self.bytes(), name.end);
        if self.byte(equals) != Some(b'=') {
            return Some(Attribute::Named {
                span: name,
                name,
                value: None,
            });
        }
        let value_start = skip_trivia(self.bytes(), equals + 1);
        let value = match self.byte(value_start) {
            Some(quote @ (b'"' | b'\'')) => {
                let Some(close) = find(self.bytes(), value_start as usize + 1, &[quote]) else {
                    let at = Span::new(value_start, self.source.len() as u32);
                    return self.fail(error::expected(if quote == b'"' { "\"" } else { "'" }, at));
                };
                AttributeValue::String(Span::new(value_start, close as u32 + 1))
            }
            Some(b'{') => {
                self.seat(value_start + 1)?;
                if self.token().0 == Kind::Dot3 {
                    let at = Span::new(value_start, value_start + 1);
                    return self.fail(error::attribute_value_spread(at));
                }
                let (expression, end) = self.braced_expression(value_start)?;
                AttributeValue::Expression {
                    span: Span::new(value_start, end),
                    expression,
                }
            }
            _ => return self.fail(error::unexpected(Span::new(value_start, value_start + 1))),
        };
        let end = match &value {
            AttributeValue::String(span) | AttributeValue::Expression { span, .. } => span.end,
        };
        Some(Attribute::Named {
            span: Span::new(start, end),
            name,
            value: Some(value),
        })
    }

    /// Children until the matching closing tag; `open` is `None` for a fragment.
    fn children(
        &mut self,
        mut offset: u32,
        open: Option<&TagName<'a>>,
    ) -> Option<(Vec<'a, Child<'a>>, Span)> {
        let body_start = offset;
        let mut children = Vec::new_in(&self.alloc);
        let mut text_start = offset;
        loop {
            let bytes = self.bytes();
            let at = offset as usize;
            let current = bytes.get(at).copied();
            let next = bytes.get(at + 1).copied();
            let special = match current {
                None | Some(b'<' | b'{' | b'}') => true,
                Some(b'@') => opens(bytes, offset, MarkupSite::Expression),
                Some(b'/') => {
                    next == Some(b'*')
                        || (next == Some(b'/')
                            && (offset == body_start
                                || bytes.get(at - 1).is_some_and(|byte| {
                                    byte.is_ascii_whitespace() || matches!(byte, b'>' | b'}')
                                })))
                }
                Some(_) => false,
            };
            if !special {
                offset += 1;
                continue;
            }
            if text_start < offset {
                children.push(Child::Text(Span::new(text_start, offset)));
            }
            match (current, next) {
                (None | Some(b'}'), _) => {
                    let name = open.map_or("", |name| self.text(name_span(name)));
                    return self.fail(error::unclosed_tag(name, Span::new(offset, offset + 1)));
                }
                (Some(b'<'), Some(b'/')) => {
                    let closing = self.closing_tag(offset, open)?;
                    return Some((children, closing));
                }
                (Some(b'<'), _) | (Some(b'@'), _) => {
                    let node = self.node(offset)?;
                    offset = node.span().end;
                    children.push(Child::Node(node));
                }
                (Some(b'{'), _) => {
                    let child = self.expression_child(offset)?;
                    offset = child.span.end;
                    children.push(Child::Expression(child));
                }
                (Some(_), Some(b'*')) => {
                    let Some(end) = find(bytes, at + 2, b"*/") else {
                        return self.fail(error::expected("*/", Span::new(offset, offset + 2)));
                    };
                    let span = Span::new(offset, end as u32 + 2);
                    children.push(Child::Comment(Comment {
                        kind: CommentKind::Block,
                        span,
                    }));
                    offset = span.end;
                }
                (Some(_), _) => {
                    let span = Span::new(offset, line_end(bytes, at) as u32);
                    children.push(Child::Comment(Comment {
                        kind: CommentKind::Line,
                        span,
                    }));
                    offset = span.end;
                }
            }
            text_start = offset;
        }
    }

    fn expression_child(&mut self, open: u32) -> Option<ExpressionChild<'a>> {
        self.seat(open + 1)?;
        let (kind, span) = self.token();
        if kind == Kind::RCurly {
            return Some(ExpressionChild {
                span: Span::new(open, span.end),
                expression: None,
            });
        }
        if kind == Kind::Dot3 {
            return self.fail(error::spread_child(span));
        }
        let expression = self.cursor.expression();
        if self.cursor.failed() {
            return None;
        }
        let (kind, span) = self.token();
        if kind == Kind::Semicolon {
            return self.fail(error::trailing_semicolon(span));
        }
        let close = self.expect(Kind::RCurly)?;
        Some(ExpressionChild {
            span: Span::new(open, close.end),
            expression: Some(expression),
        })
    }

    fn closing_tag(&mut self, start: u32, open: Option<&TagName<'a>>) -> Option<Span> {
        let name_start = start + 2;
        let name_end = match self.byte(name_start) {
            Some(b'>') => name_start,
            Some(b'{') => {
                let (_, end) = self.braced_expression(name_start)?;
                end
            }
            _ => {
                let len = self.bytes().get(name_start as usize..).map_or(0, |rest| {
                    rest.iter()
                        .take_while(|byte| {
                            is_identifier_part(**byte) || matches!(byte, b'-' | b'.')
                        })
                        .count()
                });
                name_start + len as u32
            }
        };
        let gt = skip_trivia(self.bytes(), name_end);
        if self.byte(gt) != Some(b'>') {
            return self.fail(error::expected(">", Span::new(gt, gt + 1)));
        }
        let found = self.text(Span::new(name_start, name_end));
        let expected = open.map_or("", |name| self.text(name_span(name)));
        if found != expected {
            let at = Span::new(start, gt + 1);
            return self.fail(if open.is_none() && !found.is_empty() {
                error::unexpected_closing_tag(at)
            } else {
                error::mismatched_closing_tag(expected, found, at)
            });
        }
        Some(Span::new(start, gt + 1))
    }

    /// `<script>` bodies are always raw text; `<style>` bodies are raw CSS unless
    /// the body is an expression container (plain TSX `<style>{css}</style>`).
    fn raw_text_kind(&self, name: &TagName<'a>, body: u32) -> Option<&'static [u8]> {
        let TagName::Static(span) = name else {
            return None;
        };
        match self.text(*span) {
            "script" => Some(b"script"),
            "style" => {
                let first = self
                    .bytes()
                    .get(body as usize..)?
                    .iter()
                    .find(|byte| !byte.is_ascii_whitespace());
                (first != Some(&b'{')).then_some(b"style")
            }
            _ => None,
        }
    }

    /// Raw text through `</tag`, optional whitespace and `>`.
    fn raw_text(&mut self, tag: &[u8], body: u32) -> Option<(Span, Span)> {
        let bytes = self.bytes();
        let mut from = body as usize;
        loop {
            let Some(close) = find(bytes, from, b"</") else {
                let name = core::str::from_utf8(tag).unwrap_or_default();
                let at = Span::new(bytes.len() as u32, bytes.len() as u32);
                return self.fail(error::unclosed_tag(name, at));
            };
            let name_end = close + 2 + tag.len();
            let matches_case_insensitive = bytes
                .get(close + 2..name_end)
                .is_some_and(|name| name.eq_ignore_ascii_case(tag));
            if !matches_case_insensitive {
                from = close + 2;
                continue;
            }
            let gt = skip_whitespace(bytes, name_end);
            let exact = bytes.get(close + 2..name_end) == Some(tag);
            if exact && bytes.get(gt) == Some(&b'>') {
                return Some((
                    Span::new(body, close as u32),
                    Span::new(close as u32, gt as u32 + 1),
                ));
            }
            if tag == b"script" {
                let at = Span::new(close as u32, name_end as u32);
                return self.fail(error::script_end_tag_in_body(self.text(at), at));
            }
            from = close + 2;
        }
    }
}

fn skip_whitespace(bytes: &[u8], from: usize) -> usize {
    bytes.get(from..).map_or(from, |rest| {
        from + rest
            .iter()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count()
    })
}

fn name_span(name: &TagName<'_>) -> Span {
    match name {
        TagName::Static(span) => *span,
        // The text between the braces, as the closing tag repeats it.
        TagName::Dynamic { span, .. } => Span::new(span.start, span.end),
    }
}

fn attribute_end(attribute: &Attribute<'_>) -> u32 {
    match attribute {
        Attribute::Named { span, .. }
        | Attribute::Shorthand { span, .. }
        | Attribute::Spread { span, .. } => span.end,
    }
}

/// An identifier, a non-optional member chain from an identifier or `this` with
/// simple computed keys, or a string literal.
fn is_dynamic_tag(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_) | Expression::StringLiteral(_) => true,
        _ => is_tag_member(expression),
    }
}

fn is_tag_member(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_) | Expression::ThisExpression(_) => true,
        Expression::StaticMemberExpression(member) => {
            !member.optional && is_tag_member(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            !member.optional
                && is_tag_member(&member.object)
                && matches!(
                    member.expression,
                    Expression::Identifier(_)
                        | Expression::StringLiteral(_)
                        | Expression::NumericLiteral(_)
                        | Expression::StaticMemberExpression(_)
                        | Expression::ComputedMemberExpression(_)
                )
        }
        _ => false,
    }
}
