//! Projection rows from the S4 emission document (P3-9, consumed by P4-5b).
//!
//! Every S4 target writes an [`EmitDocument`] whose [`SpanLink`]s pair
//! generated ranges with authored ranges. The same links serialize as Source
//! Map v3 segments (in `vize_atelier_core`) and, here, as
//! [`ProjectionMapping`] rows, so a projection emitted through the document
//! needs no mapping model of its own.
//!
//! [`virtual_ts_document`] is the checker's virtual TypeScript module in that
//! same document. The text is the generator's code, moved unchanged.

use std::ops::Range;

use vize_atelier_core::codegen::document::{EmitDocument, SpanLink};
use vize_s0::Span;

use super::{ProjectionMapping, VizeMapping, VizeSubSpan};

/// The checker's virtual TypeScript module as an S4 emission document.
///
/// `code` moves into the document unchanged, so snapshots and the
/// content-mapper protocol keep the generator's bytes. Each non-empty row
/// and sub-span becomes a link. [`ProjectionMapping::from_emit_document`]
/// nests a link inside the previous row when its generated range sits inside
/// that row. The generator also emits those overlaps as their own rows, so
/// the checker keeps that mapping and does not replace it with the nested
/// reading.
pub fn virtual_ts_document(code: vize_s0::String, spans: &[VizeMapping]) -> EmitDocument {
    let mut links = Vec::new();
    for span in spans {
        push_link(&mut links, &span.gen_range, &span.src_range);
        for sub in &span.sub_spans {
            push_link(&mut links, &sub.gen_range, &sub.src_range);
        }
    }
    EmitDocument::from_parts(code, links)
}

/// Move `output`'s code through [`virtual_ts_document`] and return it.
///
/// The mapping stays the generator's. Nested generated ranges are links on
/// the document, and folding those links would merge rows the generator
/// keeps separate.
pub(crate) fn publish_virtual_ts(
    mut output: super::super::types::VirtualTsOutput,
) -> super::super::types::VirtualTsOutput {
    let code = std::mem::take(&mut output.code);
    output.code = virtual_ts_document(code, output.mapping.spans()).into_string();
    output
}

fn push_link(links: &mut Vec<SpanLink>, generated: &Range<usize>, authored: &Range<usize>) {
    let (Ok(generated_start), Ok(generated_end), Ok(authored_start), Ok(authored_end)) = (
        u32::try_from(generated.start),
        u32::try_from(generated.end),
        u32::try_from(authored.start),
        u32::try_from(authored.end),
    ) else {
        return;
    };
    if generated_start >= generated_end || authored_start >= authored_end {
        return;
    }
    links.push(SpanLink {
        generated: Span::new(generated_start, generated_end),
        authored: Span::new(authored_start, authored_end),
        name: None,
        segment: true,
    });
}

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
    use super::{ProjectionMapping, VizeMapping, VizeSubSpan, virtual_ts_document};
    use vize_atelier_core::codegen::document::EmitDocument;
    use vize_s0::Span;

    #[test]
    fn virtual_ts_document_keeps_the_code_and_the_rows() {
        let code = "return _ctx.a + _ctx.b;";
        let mut expression = VizeMapping::new(7..22, 13..18);
        expression.sub_spans = vec![
            VizeSubSpan {
                gen_range: 7..13,
                src_range: 13..14,
            },
            VizeSubSpan {
                gen_range: 16..22,
                src_range: 17..18,
            },
        ];
        let spans = vec![VizeMapping::new(0..6, 0..6), expression];
        let document = virtual_ts_document(vize_s0::String::new(code), &spans);
        assert_eq!(document.as_str(), code);
        assert_eq!(
            ProjectionMapping::from_emit_document(&document).spans(),
            spans.as_slice()
        );
    }

    #[test]
    fn checker_document_matches_the_generator_on_an_sfc() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const message = ref('Hello')
</script>
<template>
  <p>{{ message }}</p>
  <button @click="message = 'x'">ok</button>
</template>
"#;
        let descriptor =
            vize_atelier_sfc::parse_sfc(source, vize_atelier_sfc::SfcParseOptions::default())
                .expect("sfc");
        let template = descriptor.template.as_ref().expect("template");
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, &template.content);
        let summary = vize_atelier_sfc::croquis::analyze_sfc_descriptor(
            &descriptor,
            Some(&root),
            vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
        );
        let script = descriptor
            .script_setup
            .as_ref()
            .map(|block| block.content.as_ref());
        let output = crate::virtual_ts::generate_virtual_ts(
            &summary,
            script,
            Some(&root),
            template.loc.start as u32,
        );
        let document = virtual_ts_document(output.code.clone(), output.mapping.spans());
        assert_eq!(document.as_str(), output.code.as_str());
        // This fixture's generated ranges do not nest, so the document's
        // reading matches the generator row for row. Overlapping rows stay
        // on the generator mapping the checker stores.
        assert_eq!(
            ProjectionMapping::from_emit_document(&document).spans(),
            output.mapping.spans()
        );
        assert!(!output.code.is_empty());
        assert!(!output.mapping.is_empty());
    }

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
