//! Lazy enrichment of an already selected native HTML hover.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupKind};

use super::elements::{self, ElementDocs};

const DESCRIPTION: &str = "\n\n";
const MDN: &str = "\n\n**Docs**\n\n[MDN reference](";
const STANDARD: &str = ")\n\n[HTML Living Standard](";
const END: &str = ")";

/// Keep the complete converted native type text and its existing range.
pub(super) fn enrich_native_html(mut hover: Hover, tag: &str, category: &str) -> Hover {
    // Distinct SVG/MathML-classified tags bypass the HTML table. Overlapping
    // spellings retain the existing provider's HTML-first classification;
    // this function does not introduce authored-namespace analysis.
    if category != "HTML element" {
        return hover;
    }
    let Some(docs) = elements::lookup(tag) else {
        return hover;
    };
    append(&mut hover, docs, true);
    hover
}

pub(super) fn append(hover: &mut Hover, docs: &ElementDocs, include_description: bool) {
    // convert_lsp_hover always supplies Markdown; do not reinterpret any other
    // content shape if its contract changes in the future.
    let HoverContents::Markup(content) = &mut hover.contents else {
        return;
    };
    if content.kind != MarkupKind::Markdown {
        return;
    }
    let pieces = [
        if include_description { DESCRIPTION } else { "" },
        if include_description {
            docs.description
        } else {
            ""
        },
        MDN,
        docs.mdn_url,
        STANDARD,
        docs.standard_url,
        END,
    ];
    // Grow the existing output once; no temporary Markdown or descriptor copies.
    content
        .value
        .reserve(pieces.iter().map(|piece| piece.len()).sum());
    for piece in pieces {
        content.value.push_str(piece);
    }
}
