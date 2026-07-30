//! Resolve which part of an authored SFC is safe to scan as markup.
//!
//! Shared by the providers that read the authored text directly rather than the
//! virtual TypeScript projection (selection ranges, linked editing). Getting the
//! region wrong is not cosmetic: scanning a `<script>` body as markup makes
//! `a < b` look like a tag, which would silently attach editor features to
//! nonsense spans.

use crate::utils::is_standalone_html_path;

/// The SFC block layout around a cursor offset.
pub(crate) struct SfcRegion {
    /// `(tag_start, tag_end)` and `(content_start, content_end)` of every block
    /// whose tags enclose the cursor, in discovery order.
    pub(crate) block_spans: Vec<(usize, usize)>,
    /// The region that may be scanned as markup, or `None` when the cursor sits
    /// inside a raw-text block (`<script>`, `<style>`).
    pub(crate) markup: Option<(usize, usize)>,
}

/// Resolve the block layout for `offset`.
///
/// Standalone petite-vue HTML and `.art.vue` documents are not SFCs, and an SFC
/// that fails to parse (mid-edit) has no trustworthy block boundaries; both scan
/// the whole file, which is the behaviour that degrades most gracefully.
pub(crate) fn resolve(content: &str, filename: &str, offset: usize) -> SfcRegion {
    let whole_file = SfcRegion {
        block_spans: Vec::new(),
        markup: Some((0, content.len())),
    };

    if is_standalone_html_path(filename) || filename.ends_with(".art.vue") {
        return whole_file;
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return whole_file;
    };

    let template_loc = descriptor.template.as_ref().map(|block| &block.loc);
    let raw_text_locs = descriptor
        .script
        .as_ref()
        .map(|block| &block.loc)
        .into_iter()
        .chain(descriptor.script_setup.as_ref().map(|block| &block.loc))
        .chain(descriptor.styles.iter().map(|block| &block.loc));

    let mut block_spans = Vec::new();
    for loc in template_loc.into_iter().chain(raw_text_locs.clone()) {
        if loc.tag_start <= offset && offset <= loc.tag_end {
            block_spans.push((loc.tag_start, loc.tag_end));
            block_spans.push((loc.start, loc.end));
        }
    }

    // A raw-text block is off-limits including its own tags: a cursor on
    // `<script>` must not make the scanner walk the script body looking for
    // elements.
    if raw_text_locs
        .into_iter()
        .any(|loc| loc.tag_start <= offset && offset <= loc.tag_end)
    {
        return SfcRegion {
            block_spans,
            markup: None,
        };
    }

    let markup = template_loc
        // The template block's own tags are markup too, so a cursor anywhere
        // from `<template>` to `</template>` scans the whole block.
        .filter(|loc| loc.tag_start <= offset && offset <= loc.tag_end)
        .map(|loc| (loc.tag_start, loc.tag_end))
        // Outside every block (for example on a custom block's tag) the whole
        // file is safe: raw-text blocks were already excluded above.
        .unwrap_or((0, content.len()));

    SfcRegion {
        block_spans,
        markup: Some(markup),
    }
}
