//! Report placement for the core-rule visitor: offsets, operators and holes.

use super::{compatibility_help, tokens::find_token, visitor::CoreVisitor};
use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement};
use oxc_span::{GetSpan, Span};

impl CoreVisitor<'_, '_> {
    pub(super) fn report(&mut self, span: Span) {
        if self.silent {
            return;
        }
        let span = Span::new(
            span.start.saturating_add(self.parse_offset),
            span.end.saturating_add(self.parse_offset),
        );
        if let Some(range) = self.origin.range(span) {
            self.ctx
                .warn_at_with_help(self.kind.message(), range, compatibility_help());
        }
    }

    pub(super) fn set_parse_offset(&mut self, parse_offset: u32) {
        if self.parse_offset != parse_offset {
            self.parse_offset = parse_offset;
            self.covered_spans.clear();
        }
    }

    /// The first `token` between two parse-relative offsets, outside comments.
    pub(super) fn token_between(&self, start: u32, end: u32, token: &str) -> Option<Span> {
        let offset = self.parse_offset;
        let span = find_token(
            self.source,
            start.checked_add(offset)?,
            end.checked_add(offset)?,
            token,
        )?;
        Some(Span::new(
            span.start.checked_sub(offset)?,
            span.end.checked_sub(offset)?,
        ))
    }

    pub(super) fn report_operator(&mut self, left: Span, right: Span, operator: &str) {
        let span = self
            .token_between(left.end, right.start, operator)
            .unwrap_or(Span::new(left.start, right.end));
        self.report(span);
    }

    /// ESLint reports each hole at the comma that closes it.
    pub(super) fn report_sparse_array_holes(&mut self, array: &ArrayExpression<'_>) {
        let mut cursor = array.span.start.saturating_add(1);
        let last = array.elements.len().saturating_sub(1);
        for (index, element) in array.elements.iter().enumerate() {
            let hole = matches!(element, ArrayExpressionElement::Elision(_));
            if index == last && !hole {
                return;
            }
            if !hole {
                cursor = element.span().end;
            }
            let Some(comma) = self.token_between(cursor, array.span.end, ",") else {
                return;
            };
            cursor = comma.end;
            if hole {
                self.report(comma);
            }
        }
    }

    pub(super) fn expressions_share_line(&self, left: Span, right: Span) -> bool {
        let adjusted = |span: Span| {
            Span::new(
                span.start.saturating_add(self.parse_offset),
                span.end.saturating_add(self.parse_offset),
            )
        };
        let Some(left) = self.origin.range(adjusted(left)) else {
            return false;
        };
        let Some(right) = self.origin.range(adjusted(right)) else {
            return false;
        };
        self.ctx
            .source
            .get(left.end as usize..right.start as usize)
            .is_some_and(|between| !between.contains(['\n', '\r']))
    }
}
