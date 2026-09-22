//! Projection rows from the S4 emission document (P3-9, consumed by P4-5b).
//!
//! Every S4 target writes an [`EmitDocument`] whose [`SpanLink`]s pair
//! generated ranges with authored ranges. The same links serialize as Source
//! Map v3 segments (in `vize_atelier_core`) and, here, as
//! [`ProjectionMapping`] rows, so a projection emitted through the document
//! needs no mapping model of its own.

use vize_atelier_core::codegen::document::{EmitDocument, SpanLink};

use super::{ProjectionMapping, VizeMapping, VizeSubSpan};

impl ProjectionMapping {
    /// Rows for the range links of an S4 emission document, in emission
    /// order. A link that spans generated and authored bytes becomes a row,
    /// unless it lies inside the preceding row's generated range (a rewritten
    /// identifier inside its expression), in which case it refines that row
    /// as a sub-span. Unit anchors carry no authored extent and yield no row.
    pub fn from_emit_links(links: &[SpanLink]) -> Self {
        let mut spans: Vec<VizeMapping> = Vec::new();
        for link in links {
            let (generated, authored) = (link.generated, link.authored);
            if generated.start >= generated.end || authored.start >= authored.end {
                continue;
            }
            let gen_range = generated.start as usize..generated.end as usize;
            let src_range = authored.start as usize..authored.end as usize;
            match spans.last_mut() {
                Some(row)
                    if row.gen_range.start <= gen_range.start
                        && gen_range.end <= row.gen_range.end =>
                {
                    row.sub_spans.push(VizeSubSpan {
                        gen_range,
                        src_range,
                    });
                }
                _ => spans.push(VizeMapping::new(gen_range, src_range)),
            }
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
}
