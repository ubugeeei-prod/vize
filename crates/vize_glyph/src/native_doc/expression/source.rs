//! Checked original source and typed-comment gaps, without a JS reparse.

use oxc_ast::ast::CommentKind;
use vize_l0::{Allocator, SourceBlock, Span, Vec};
use vize_l1::embed::{SourceError, syntax::RetainedExpression};

use super::{Doc, ExpressionRefusal};
use crate::native_doc::Line;

#[derive(Clone, Copy)]
pub(super) enum Gap {
    Empty,
    Space,
    Break,
}

pub(super) struct Context<'p, 'a> {
    pub original: &'p RetainedExpression<'a>,
    pub allocator: &'a Allocator,
    block: SourceBlock<'a>,
    comments: Vec<'a, Span>,
    comment_index: usize,
}

impl<'p, 'a> Context<'p, 'a> {
    pub fn new(
        original: &'p RetainedExpression<'a>,
        block: SourceBlock<'a>,
        allocator: &'a Allocator,
    ) -> Result<Self, ExpressionRefusal> {
        let source = original.source();
        if source.authored_root().len() != block.root_source().len()
            || source.authored_root().as_ptr() != block.root_source().as_ptr()
            || !block.contains_block_span(source.span())
        {
            return Err(ExpressionRefusal::SourceMismatch {
                span: source.span(),
            });
        }
        let mut context = Self {
            original,
            allocator,
            block,
            comments: Vec::new_in(&allocator),
            comment_index: 0,
        };
        let entire = Span::new(0, source.text().len() as u32);
        let projected = context.authored_span(entire)?;
        if projected != source.span() {
            return Err(ExpressionRefusal::SourceMismatch {
                span: source.span(),
            });
        }
        if source.decode_map().is_none()
            && context.authored(entire)?.as_ptr() != source.text().as_ptr()
        {
            return Err(ExpressionRefusal::SourceMismatch {
                span: source.span(),
            });
        }
        let mut end = 0;
        for comment in original.comments() {
            let span = comment
                .decoded_span()
                .map_err(|error| context.projection(error))?;
            let authored = comment
                .authored_span()
                .map_err(|error| context.projection(error))?;
            if span.start < end
                || span.start >= span.end
                || authored != context.authored_span(span)?
            {
                return Err(ExpressionRefusal::InvalidFraming { span });
            }
            let text = comment.text().map_err(|error| context.projection(error))?;
            if text.as_ptr() != context.decoded(span)?.as_ptr()
                || match comment.kind() {
                    CommentKind::Line => !text.starts_with("//"),
                    CommentKind::SingleLineBlock | CommentKind::MultiLineBlock => {
                        !text.starts_with("/*") || !text.ends_with("*/")
                    }
                }
            {
                return Err(ExpressionRefusal::InvalidFraming { span });
            }
            context.authored(span)?;
            context.comments.push(span);
            end = span.end;
        }
        Ok(context)
    }

    fn projection(&self, error: SourceError) -> ExpressionRefusal {
        ExpressionRefusal::Projection {
            offset: self.original.source().span().start,
            error,
        }
    }

    pub fn decoded_span(&self, span: oxc_span::Span) -> Result<Span, ExpressionRefusal> {
        self.original
            .decoded_span(span)
            .map_err(|error| self.projection(error))
    }

    pub fn authored_span(&self, span: Span) -> Result<Span, ExpressionRefusal> {
        let authored = self
            .original
            .source()
            .authored_span(span)
            .map_err(|error| self.projection(error))?;
        if !self.block.contains_block_span(authored) {
            return Err(ExpressionRefusal::SourceMismatch { span: authored });
        }
        Ok(authored)
    }

    fn decoded(&self, span: Span) -> Result<&'a str, ExpressionRefusal> {
        self.original
            .source()
            .text()
            .get(span.start as usize..span.end as usize)
            .ok_or(ExpressionRefusal::InvalidFraming { span })
    }

    fn authored(&self, span: Span) -> Result<&'a str, ExpressionRefusal> {
        let authored = self.authored_span(span)?;
        self.block
            .root_source()
            .get(authored.start as usize..authored.end as usize)
            .ok_or(ExpressionRefusal::SourceMismatch { span: authored })
    }

    pub fn text(&self, span: Span) -> Result<Doc<'a>, ExpressionRefusal> {
        if self
            .comments
            .get(self.comment_index)
            .is_some_and(|comment| comment.start < span.end)
        {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        Ok(Doc::text(self.authored(span)?))
    }

    fn whitespace(&self, span: Span) -> Result<(), ExpressionRefusal> {
        if !self
            .decoded(span)?
            .bytes()
            .all(|byte| byte.is_ascii_whitespace())
        {
            return Err(ExpressionRefusal::InvalidGap { span });
        }
        Ok(())
    }

    pub fn gap(&mut self, span: Span, style: Gap) -> Result<Doc<'a>, ExpressionRefusal> {
        let mut cursor = span.start;
        let initial = self.comment_index;
        while let Some(&comment) = self.comments.get(self.comment_index) {
            if comment.start >= span.end {
                break;
            }
            if comment.start < cursor || comment.end > span.end {
                return Err(ExpressionRefusal::InvalidFraming { span: comment });
            }
            self.whitespace(Span::new(cursor, comment.start))?;
            cursor = comment.end;
            self.comment_index += 1;
        }
        self.whitespace(Span::new(cursor, span.end))?;
        let authored = self.authored(span)?;
        if initial != self.comment_index || authored != self.decoded(span)? {
            return Ok(Doc::text(authored));
        }
        Ok(match style {
            Gap::Empty => Doc::text(""),
            Gap::Space => Doc::text(" "),
            Gap::Break => Doc::line(Line::Space),
        })
    }

    /// Locate only the operator/delimiter already identified by the AST.
    /// All other gap bytes must be ASCII whitespace or original typed comments.
    pub fn token(&self, span: Span, spelling: &str) -> Result<Span, ExpressionRefusal> {
        let text = self.original.source().text();
        self.decoded(span)?;
        let mut cursor = span.start;
        let mut comment_index = self.comment_index;
        let mut found = None;
        while cursor < span.end {
            if let Some(comment) = self.comments.get(comment_index)
                && comment.start == cursor
            {
                if comment.end > span.end {
                    return Err(ExpressionRefusal::InvalidFraming { span: *comment });
                }
                cursor = comment.end;
                comment_index += 1;
            } else if text
                .as_bytes()
                .get(cursor as usize)
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                cursor += 1;
            } else if found.is_none()
                && text
                    .get(cursor as usize..span.end as usize)
                    .is_some_and(|rest| rest.starts_with(spelling))
            {
                let end = cursor
                    .checked_add(spelling.len() as u32)
                    .ok_or(ExpressionRefusal::InvalidFraming { span })?;
                found = Some(Span::new(cursor, end));
                cursor = end;
            } else {
                return Err(ExpressionRefusal::InvalidGap { span });
            }
        }
        found.ok_or(ExpressionRefusal::InvalidFraming { span })
    }

    pub fn finish(&self) -> Result<(), ExpressionRefusal> {
        if let Some(&span) = self.comments.get(self.comment_index) {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        Ok(())
    }
}
