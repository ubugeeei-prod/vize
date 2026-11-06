//! Lazy enrichment of an already selected native HTML hover.

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupKind};

use super::elements::{self, ElementDocs};

const DESCRIPTION: &str = "\n\n";
const VOID_SYNTAX: &str =
    "\n\n**HTML syntax**\n\nVoid elements do not have child content or an end tag.";
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

/// Keep the exact DOM-library type and add Vue template behavior around it.
pub(super) fn enrich_native_attribute(mut hover: Hover, tag: &str, attribute: &str) -> Hover {
    let Some(info) = crate::ide::corsa_support::native_dom_attribute_info(tag, attribute) else {
        return hover;
    };
    let HoverContents::Markup(content) = &mut hover.contents else {
        return hover;
    };
    if content.kind != MarkupKind::Markdown {
        return hover;
    }

    let runtime = if info.is_boolean {
        "Boolean HTML attribute"
    } else {
        "DOM reflected attribute"
    };
    let example = super::native_attribute_example(tag, attribute, info.is_boolean);
    let pieces = [
        "\n\n**Vue template runtime**\n\nVize recognizes this as native markup. Vue patches the platform element instead of resolving a component prop.",
        "\n\n**Runtime surface**\n\n",
        runtime,
        "\n\n**Editor behavior**\n- The type above comes from the active TypeScript DOM library.\n- Component prop lookup is skipped because the enclosing tag is a native element.",
        "\n\n**Example**\n\n```vue\n",
        example.as_str(),
        "\n```\n\n**Docs**\n\n[MDN reference](",
        info.documentation_url.as_str(),
        ")",
    ];
    content
        .value
        .reserve(pieces.iter().map(|piece| piece.len()).sum());
    for piece in pieces {
        content.value.push_str(piece);
    }
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
        if vize_l0::is_void_tag(docs.tag) {
            VOID_SYNTAX
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
