//! Compatibility-node coordinates, expressions, and structural classification.

use memchr::memchr;
use oxc_allocator::CloneIn;
use oxc_ast_visit::VisitMut;
use vize_l0::{Box, Span};
use vize_l1::markup::directive::{DirectivePrefix, DirectiveSyntax, VueDirectives};
use vize_l1::markup::entity::{EntityContext, decode_one};
use vize_l1::markup::{AttrValue, Attribute, Element, Token};
use vize_l2::expr::ExprRef;
use vize_relief::{
    ConstantType, ElementType, ExpressionNode, Namespace, RetainedJsAst, RetainedJsAstKind,
    SimpleExpressionNode, SourceLocation,
};

use super::CompatibilityRoot;

impl<'t, 'a> CompatibilityRoot<'t, 'a> {
    pub(super) fn expression(
        &self,
        fallback: &'a str,
        span: Span,
        is_static: bool,
    ) -> ExpressionNode<'a> {
        if is_static {
            return ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(fallback, true, SourceLocation { span }),
                &self.allocator,
            ));
        }
        let retained = self.retained_expression(fallback, span);
        let content = retained.map_or(fallback, |expression| {
            if expression.span() == span {
                expression.source()
            } else {
                fallback
            }
        });
        let mut node = SimpleExpressionNode::new(content, is_static, SourceLocation { span });
        if let Some(ExprRef::Js(js)) = retained {
            let mut ast = js.ast.clone_in(self.allocator.as_oxc());
            let offset = js.span.start.saturating_sub(span.start);
            DecodedExpressionSpans {
                expression: js,
                offset,
            }
            .visit_expression(&mut ast);
            node.js_ast = Some(RetainedJsAst {
                ast: self
                    .allocator
                    .as_oxc()
                    .alloc(RetainedJsAstKind::Expression(ast)),
                raw: content,
            });
            node.const_type = ConstantType::NotConstant;
        }
        ExpressionNode::Simple(Box::new_in(node, &self.allocator))
    }

    fn retained_expression(&self, fallback: &str, span: Span) -> Option<ExprRef<'a>> {
        if let Some(expression) = self.expressions.get(&(span.start, span.end)).copied() {
            return Some(expression);
        }
        let trimmed = fallback.trim();
        if trimmed.len() == fallback.len() {
            return None;
        }
        let leading = fallback.len().saturating_sub(fallback.trim_start().len()) as u32;
        let trailing = fallback.len().saturating_sub(fallback.trim_end().len()) as u32;
        let trimmed_span = Span::new(
            span.start.saturating_add(leading),
            span.end.saturating_sub(trailing),
        );
        self.expressions
            .get(&(trimmed_span.start, trimmed_span.end))
            .copied()
            .filter(|expression| expression.source() == trimmed)
    }

    pub(super) fn decoded_value(&self, value: &AttrValue<'a>) -> &'a str {
        let raw = value.content.text;
        if raw.contains('&') {
            self.allocator
                .alloc_str(decode_html_entities(raw, EntityContext::Attribute).as_str())
        } else {
            raw
        }
    }

    pub(super) fn decoded_text(&self, raw: &'a str) -> &'a str {
        if raw.contains('&') {
            self.allocator
                .alloc_str(decode_html_entities(raw, EntityContext::Text).as_str())
        } else {
            raw
        }
    }

    pub(super) fn attribute_span(&self, attribute: &Attribute<'a>) -> Span {
        let start = self.offset(attribute.name.text);
        let end = attribute.value.as_ref().map_or_else(
            || start + attribute.name.text.len() as u32,
            |value| {
                value.close_quote.as_ref().map_or_else(
                    || self.offset(value.content.text) + value.content.text.len() as u32,
                    |quote| self.offset(quote.text) + quote.text.len() as u32,
                )
            },
        );
        Span::new(start, end)
    }

    pub(super) fn token_loc(&self, token: &Token<'a>) -> SourceLocation {
        SourceLocation {
            span: self.token_span(token),
        }
    }

    pub(super) fn token_span(&self, token: &Token<'a>) -> Span {
        let start = self.offset(token.text);
        Span::new(start, start + token.text.len() as u32)
    }

    pub(super) fn offset(&self, slice: &str) -> u32 {
        let offset =
            (slice.as_ptr() as usize).saturating_sub(self.surface.source.as_ptr() as usize);
        debug_assert!(
            offset <= self.surface.source.len() && offset <= u32::MAX as usize,
            "L1 tokens borrow a source admitted by its u32 coordinate model"
        );
        offset as u32
    }
}

fn decode_html_entities(source: &str, context: EntityContext) -> vize_l0::String {
    let mut decoded = vize_l0::String::with_capacity(source.len());
    let mut rest = source;
    while let Some(candidate_offset) = memchr(b'&', rest.as_bytes()) {
        let (literal, candidate) = rest.split_at(candidate_offset);
        decoded.push_str(literal);

        if context == EntityContext::Text
            && let Some((character, consumed)) = decode_escaped_numeric_parenthesis(candidate)
        {
            decoded.push(character);
            rest = &candidate[consumed..];
            continue;
        }

        if let Some((entity, consumed)) = decode_one(candidate.as_bytes(), context) {
            entity.for_each(|character| decoded.push(character));
            rest = &candidate[consumed..];
            continue;
        }

        decoded.push('&');
        rest = &candidate[1..];
    }
    decoded.push_str(rest);
    decoded
}

fn decode_escaped_numeric_parenthesis(source: &str) -> Option<(char, usize)> {
    const ESCAPED_PARENTHESES: [(&str, char); 4] = [
        ("&amp;#40;", '('),
        ("&amp;#040;", '('),
        ("&amp;#41;", ')'),
        ("&amp;#041;", ')'),
    ];
    ESCAPED_PARENTHESES
        .into_iter()
        .find_map(|(spelling, character)| {
            source
                .starts_with(spelling)
                .then_some((character, spelling.len()))
        })
}

struct DecodedExpressionSpans<'a> {
    expression: &'a vize_l2::expr::JsExpr<'a>,
    offset: u32,
}

impl<'a> VisitMut<'a> for DecodedExpressionSpans<'a> {
    fn visit_span(&mut self, span: &mut oxc_span::Span) {
        if let Some(decoded) = self.expression.ast_span_to_source(*span) {
            *span = oxc_span::Span::new(
                decoded.start.saturating_add(self.offset),
                decoded.end.saturating_add(self.offset),
            );
        }
    }
}

pub(super) fn comment_body(text: &str) -> &str {
    text.strip_prefix("<!--")
        .and_then(|text| text.strip_suffix("-->"))
        .unwrap_or(text)
}

pub(super) fn surface_namespace(
    parent_namespace: Namespace,
    parent_tag: Option<&str>,
    tag: &str,
) -> Namespace {
    if vize_l0::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_l0::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }
    match parent_namespace {
        Namespace::Svg
            if !parent_tag.is_some_and(|tag| matches!(tag, "foreignObject" | "desc" | "title")) =>
        {
            Namespace::Svg
        }
        Namespace::MathMl
            if !parent_tag.is_some_and(|tag| {
                matches!(tag, "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext")
            }) =>
        {
            Namespace::MathMl
        }
        Namespace::Html | Namespace::Svg | Namespace::MathMl => Namespace::Html,
    }
}

pub(super) fn element_type(
    element: &Element<'_>,
    frozen: bool,
    namespace: Namespace,
) -> ElementType {
    let tag = element.tag();
    if frozen || namespace != Namespace::Html {
        return ElementType::Element;
    }
    if tag == "slot" {
        return ElementType::Slot;
    }
    if tag == "template"
        && element.open.attrs.iter().any(|attribute| {
            VueDirectives
                .decompose(attribute.name.text, 0)
                .ok()
                .flatten()
                .is_some_and(|directive| {
                    directive.prefix == DirectivePrefix::Slot
                        || (directive.prefix == DirectivePrefix::Full
                            && matches!(
                                directive.name.slice(attribute.name.text),
                                "if" | "else-if" | "else" | "for" | "slot"
                            ))
                })
        })
    {
        return ElementType::Template;
    }
    if vize_l1::dialect::vue3::is_lint_component(tag, true) {
        ElementType::Component
    } else {
        ElementType::Element
    }
}

pub(super) fn is_v_pre_head(name: &str) -> bool {
    VueDirectives
        .decompose(name, 0)
        .ok()
        .flatten()
        .is_some_and(|directive| {
            directive.prefix == DirectivePrefix::Full && directive.name.slice(name) == "pre"
        })
}
