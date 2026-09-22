//! Projection rows from the S4 emission document (P3-9, consumed by P4-5b).
//!
//! Every S4 target writes an [`EmitDocument`] whose [`SpanLink`]s pair
//! generated ranges with authored ranges. The same links serialize as Source
//! Map v3 segments (in `vize_atelier_core`) and, here, as
//! [`ProjectionMapping`] rows, so a projection emitted through the document
//! needs no mapping model of its own.

use std::ops::Range;

use vize_atelier_core::codegen::document::{EmitDocument, SpanLink};

use super::{ProjectionMapping, VizeMapping, VizeSubSpan};

impl ProjectionMapping {
    /// Rows for the range links of an S4 emission document. Links nest by
    /// containment, whatever order they were recorded in: an outermost link
    /// (by generated range) is a row, and every link inside it refines that
    /// row as a sub-span, narrowest first so the most exact authored bytes
    /// win. Rows follow generated order. Unit anchors carry no authored
    /// extent and yield no row.
    pub fn from_emit_links(links: &[SpanLink]) -> Self {
        let mut ranged: Vec<(Range<usize>, Range<usize>)> = links
            .iter()
            .filter(|link| {
                link.generated.start < link.generated.end && link.authored.start < link.authored.end
            })
            .map(|link| {
                (
                    link.generated.start as usize..link.generated.end as usize,
                    link.authored.start as usize..link.authored.end as usize,
                )
            })
            .collect();
        // Stable: equal generated ranges keep recording order.
        ranged.sort_by(|left, right| {
            left.0
                .start
                .cmp(&right.0.start)
                .then(right.0.end.cmp(&left.0.end))
        });
        let mut spans: Vec<VizeMapping> = Vec::new();
        for (gen_range, src_range) in ranged {
            match spans.last_mut() {
                Some(row) if gen_range.end <= row.gen_range.end => {
                    row.sub_spans.push(VizeSubSpan {
                        gen_range,
                        src_range,
                    });
                }
                _ => spans.push(VizeMapping::new(gen_range, src_range)),
            }
        }
        for row in &mut spans {
            row.sub_spans
                .sort_by_key(|span| (span.gen_range.len(), span.gen_range.start));
        }
        Self::from_spans(spans)
    }

    /// Rows for every range link of `document`.
    pub fn from_emit_document(document: &EmitDocument) -> Self {
        Self::from_emit_links(document.links())
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectionMapping, VizeMapping, VizeSubSpan};
    use vize_atelier_core::codegen::document::EmitDocument;
    use vize_s0::Span;

    #[test]
    fn document_links_become_rows_with_identifier_sub_spans() {
        let source = "<p id=\"x\">{{ a + b }}</p>";
        let mut doc = EmitDocument::new(true);
        doc.anchor(0);
        doc.push_str("return ");
        doc.push_mapped("_openBlock(", 0);
        doc.push_named("id", Span::new(3, 5), "id");
        doc.push_str(": ");
        doc.push_expression("_ctx.a + _ctx.b", Span::new(13, 18), source);

        let mapping = ProjectionMapping::from_emit_document(&doc);
        let mut expression = VizeMapping::new(22..37, 13..18);
        expression.sub_spans = vec![
            VizeSubSpan {
                gen_range: 22..28,
                src_range: 13..14,
            },
            VizeSubSpan {
                gen_range: 31..37,
                src_range: 17..18,
            },
        ];
        assert_eq!(
            mapping.spans(),
            [VizeMapping::new(18..20, 3..5), expression]
        );
        assert_eq!(mapping.diagnostic_range_to_authored(22, 28), Some((13, 14)));
        assert_eq!(mapping.diagnostic_range_to_authored(31, 37), Some((17, 18)));
    }

    /// A statement linked after its pieces (`link_since`) still owns them:
    /// nesting follows containment, not recording order, and the narrowest
    /// sub-span answers first.
    #[test]
    fn an_enclosing_link_recorded_after_its_children_nests_them() {
        let source = "<p :title=\"a + b\" />";
        let mut doc = EmitDocument::new(true);
        doc.push_str("{\n");
        let statement = doc.len() as u32;
        doc.push_str("const __t = (");
        doc.push_expression("_ctx.a + _ctx.b", Span::new(11, 16), source);
        doc.push_str(");");
        doc.link_since(statement, Span::new(3, 17));
        doc.push_str("\n}");

        let mut row = VizeMapping::new(2..32, 3..17);
        row.sub_spans = vec![
            VizeSubSpan {
                gen_range: 15..21,
                src_range: 11..12,
            },
            VizeSubSpan {
                gen_range: 24..30,
                src_range: 15..16,
            },
            VizeSubSpan {
                gen_range: 15..30,
                src_range: 11..16,
            },
        ];
        let mapping = ProjectionMapping::from_emit_document(&doc);
        assert_eq!(mapping.spans(), [row]);
        assert_eq!(mapping.diagnostic_range_to_authored(24, 30), Some((15, 16)));
        assert_eq!(mapping.diagnostic_range_to_authored(2, 3), Some((3, 4)));
    }
}
