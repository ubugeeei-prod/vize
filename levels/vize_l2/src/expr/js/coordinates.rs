//! Neutral coordinates for a retained AST whose parser used a private wrapper.

use vize_l0::Span;

/// One complete decoded/source correspondence; entity interiors are indivisible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsSegment {
    pub decoded: Span,
    pub authored: Span,
    pub entity: bool,
}

/// An invalid coordinate bridge is refused before an expression enters L2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCoordinateError {
    SourceLimit,
    InvalidSpan,
    InvalidCoverage,
    InvalidIdentity,
    InvalidEntity,
    OutsideExpression,
}

/// Checked wrapper correction and decoded-to-authored projection.
///
/// This vocabulary is independent of L1 and HTML decoding. A producer transfers
/// the source preparation's correspondences; L2 never decodes or parses again.
#[derive(Debug, Clone, Copy)]
pub struct JsCoordinates<'a> {
    source: &'a str,
    authored: &'a str,
    span: Span,
    prefix: u32,
    segments: &'a [JsSegment],
}

impl<'a> JsCoordinates<'a> {
    /// Check complete UTF-8 coverage and literal segment identity once.
    /// An empty map denotes exact authored text, not unvalidated decoded text.
    pub fn checked(
        file: &'a str,
        source: &'a str,
        span: Span,
        prefix: u32,
        segments: &'a [JsSegment],
    ) -> Result<Self, JsCoordinateError> {
        u32::try_from(file.len()).map_err(|_| JsCoordinateError::SourceLimit)?;
        let length = u32::try_from(source.len()).map_err(|_| JsCoordinateError::SourceLimit)?;
        prefix
            .checked_add(length)
            .ok_or(JsCoordinateError::SourceLimit)?;
        let raw = file
            .get(span.start as usize..span.end as usize)
            .ok_or(JsCoordinateError::InvalidSpan)?;
        if segments.is_empty() {
            if source != raw {
                return Err(JsCoordinateError::InvalidIdentity);
            }
        } else {
            let (mut decoded, mut authored) = (0, span.start);
            for segment in segments {
                if segment.decoded.start != decoded
                    || segment.authored.start != authored
                    || segment.decoded.start >= segment.decoded.end
                    || segment.authored.start >= segment.authored.end
                    || segment.authored.end > span.end
                {
                    return Err(JsCoordinateError::InvalidCoverage);
                }
                let value = source
                    .get(segment.decoded.start as usize..segment.decoded.end as usize)
                    .ok_or(JsCoordinateError::InvalidSpan)?;
                let raw = file
                    .get(segment.authored.start as usize..segment.authored.end as usize)
                    .ok_or(JsCoordinateError::InvalidSpan)?;
                if segment.entity {
                    if !raw.starts_with('&') {
                        return Err(JsCoordinateError::InvalidEntity);
                    }
                } else if value != raw {
                    return Err(JsCoordinateError::InvalidIdentity);
                }
                (decoded, authored) = (segment.decoded.end, segment.authored.end);
            }
            if decoded != length || authored != span.end {
                return Err(JsCoordinateError::InvalidCoverage);
            }
        }
        Ok(Self {
            source,
            authored: raw,
            span,
            prefix,
            segments,
        })
    }

    pub(super) fn matches(&self, source: &str, span: Span) -> bool {
        self.source == source && self.span == span
    }

    pub(super) fn matches_authored_source(&self, file: &str) -> bool {
        file.get(self.span.start as usize..self.span.end as usize) == Some(self.authored)
    }

    pub(super) fn decoded_span(&self, span: oxc_span::Span) -> Option<Span> {
        let span = Span::new(
            span.start.checked_sub(self.prefix)?,
            span.end.checked_sub(self.prefix)?,
        );
        self.source.get(span.start as usize..span.end as usize)?;
        Some(span)
    }

    pub(super) fn authored_span(&self, span: Span) -> Option<Span> {
        self.source.get(span.start as usize..span.end as usize)?;
        if self.segments.is_empty() {
            return Some(Span::new(
                self.span.start.checked_add(span.start)?,
                self.span.start.checked_add(span.end)?,
            ));
        }
        Some(Span::new(self.offset(span.start)?, self.offset(span.end)?))
    }

    fn offset(&self, at: u32) -> Option<u32> {
        // Complete checked segments are sorted. Shared endpoints select the
        // prior segment and have the same authored offset in either neighbour.
        let index = self
            .segments
            .partition_point(|segment| segment.decoded.end < at);
        let segment = self.segments.get(index)?;
        if at < segment.decoded.start {
            return None;
        }
        if at == segment.decoded.start {
            return Some(segment.authored.start);
        }
        if at == segment.decoded.end {
            return Some(segment.authored.end);
        }
        (!segment.entity)
            .then(|| {
                segment
                    .authored
                    .start
                    .checked_add(at - segment.decoded.start)
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::{JsCoordinateError, JsCoordinates, JsSegment};
    use crate::expr::JsExpr;
    use oxc_span::{GetSpan, SourceType};
    use vize_l0::{Allocator, Span};

    #[test]
    fn retained_wrapper_corrects_ranges_without_copying_the_ast() {
        let a = Allocator::default();
        let expression = a.alloc(
            oxc_parser::Parser::new(a.as_oxc(), "(\n作者 + value\n)", SourceType::ts())
                .parse_expression()
                .unwrap(),
        );
        let oxc_ast::ast::Expression::ParenthesizedExpression(wrapper) = expression else {
            panic!("wrapper");
        };
        let ast = &wrapper.expression;
        let source = "作者 + value";
        let file = "--作者 + value--";
        let span = Span::new(2, 16);
        let coordinates = JsCoordinates::checked(file, source, span, 2, &[]).unwrap();
        let js = JsExpr::from_retained_in(&a, ast, source, span, coordinates).unwrap();
        assert!(core::ptr::eq(js.ast, ast));
        assert!(js.matches_authored_source(file));
        assert!(!js.matches_authored_source("--作者 + other--"));
        assert_eq!(js.ast_span_to_source(ast.span()), Some(Span::new(0, 14)));
        assert_eq!(js.authored_span(Span::new(0, 6)), Some(Span::new(2, 8)));
        assert_eq!(js.ast_span_to_source(oxc_span::Span::new(0, 1)), None);
        assert_eq!(js.ast_span_to_source(oxc_span::Span::new(2, 17)), None);
        assert_eq!(js.authored_span(Span::new(1, 2)), None);
    }

    #[test]
    fn equal_byte_length_entities_still_refuse_interior_edits() {
        let file = "x&acE;y";
        let source = "x∾\u{333}y";
        let segments = [
            JsSegment {
                decoded: Span::new(0, 1),
                authored: Span::new(0, 1),
                entity: false,
            },
            JsSegment {
                decoded: Span::new(1, 6),
                authored: Span::new(1, 6),
                entity: true,
            },
            JsSegment {
                decoded: Span::new(6, 7),
                authored: Span::new(6, 7),
                entity: false,
            },
        ];
        let coordinates =
            JsCoordinates::checked(file, source, Span::new(0, 7), 2, &segments).unwrap();
        assert_eq!(
            coordinates.authored_span(Span::new(1, 6)),
            Some(Span::new(1, 6))
        );
        assert_eq!(coordinates.authored_span(Span::new(1, 4)), None);
        assert_eq!(coordinates.authored_span(Span::new(4, 4)), None);
        assert_eq!(
            coordinates.authored_span(Span::new(1, 1)),
            Some(Span::new(1, 1))
        );
        assert_eq!(
            coordinates.authored_span(Span::new(6, 6)),
            Some(Span::new(6, 6))
        );
    }

    #[test]
    fn forged_or_incomplete_correspondences_do_not_enter_l2() {
        assert_eq!(
            JsCoordinates::checked("&amp;", "&", Span::new(0, 5), 0, &[]).unwrap_err(),
            JsCoordinateError::InvalidIdentity
        );
        let incomplete = [JsSegment {
            decoded: Span::new(0, 1),
            authored: Span::new(0, 1),
            entity: false,
        }];
        assert_eq!(
            JsCoordinates::checked("ab", "ab", Span::new(0, 2), 0, &incomplete).unwrap_err(),
            JsCoordinateError::InvalidCoverage
        );
        let non_entity = [JsSegment {
            decoded: Span::new(0, 1),
            authored: Span::new(0, 1),
            entity: true,
        }];
        assert_eq!(
            JsCoordinates::checked("a", "b", Span::new(0, 1), 0, &non_entity).unwrap_err(),
            JsCoordinateError::InvalidEntity
        );
        assert_eq!(
            JsCoordinates::checked("é", "é", Span::new(1, 2), 0, &[]).unwrap_err(),
            JsCoordinateError::InvalidSpan
        );
    }

    #[test]
    fn identity_load_path_keeps_original_text_and_spans() {
        let a = Allocator::default();
        let js = JsExpr::parse_in(&a, "value", Span::new(10, 15)).unwrap();
        assert!(js.coordinates.is_none());
        assert_eq!(js.ast_span_to_source(js.ast.span()), Some(Span::new(0, 5)));
        assert_eq!(js.authored_span(Span::new(0, 5)), Some(Span::new(10, 15)));
    }

    #[test]
    fn partial_expression_ast_cannot_claim_a_complete_source() {
        let a = Allocator::default();
        let ast = a.alloc(
            oxc_parser::Parser::new(a.as_oxc(), "a", SourceType::ts())
                .parse_expression()
                .unwrap(),
        );
        let coordinates =
            JsCoordinates::checked("a + b", "a + b", Span::new(0, 5), 0, &[]).unwrap();
        assert_eq!(
            JsExpr::from_retained_in(&a, ast, "a + b", Span::new(0, 5), coordinates).unwrap_err(),
            JsCoordinateError::OutsideExpression
        );
        let source = "a /* retained trivia */";
        let span = Span::new(0, source.len() as u32);
        let coordinates = JsCoordinates::checked(source, source, span, 0, &[]).unwrap();
        assert!(JsExpr::from_retained_in(&a, ast, source, span, coordinates).is_ok());
    }
}
