//! Selection range provider (`textDocument/selectionRange`).
//!
//! `@vue/language-server` advertises `selectionRangeProvider: true`, and
//! "expand/shrink selection" is bound out of the box in every editor Vize
//! packages (VS Code `Shift+Alt+Right`/`Left`, Zed `ctrl-shift-right`, Helix
//! `Alt-o`, Neovim/Emacs through their LSP clients). Maestro advertised nothing,
//! so the command silently did nothing inside `.vue` files — the worst class of
//! divergence, because the editor shows no error.
//!
//! Every range is computed on the **authored** document, never on the virtual
//! TypeScript projection, so the chain an editor receives always addresses real
//! `.vue` byte offsets.
//!
//! # Levels
//!
//! From innermost outward, the chain can contain:
//!
//! 1. the identifier/token under the cursor;
//! 2. the trimmed expression inside an interpolation (`{{ ␣count␣ }}`);
//! 3. the whole interpolation including its delimiters (`{{ count }}`);
//! 4. an attribute value inside its quotes (`:title="␣label␣"`);
//! 5. the quoted attribute value including the quotes (`:title=␣"label"␣`);
//! 6. the whole attribute (`:title="label"`);
//! 7. the start tag (`<Child :title="label" />`);
//! 8. for each enclosing element, innermost first: its inner content, then the
//!    element including both tags;
//! 9. an HTML comment (`<!-- … -->`);
//! 10. the content of the enclosing SFC block;
//! 11. the enclosing SFC block including its own tags;
//! 12. the whole document.
//!
//! Levels 2-9 are markup levels and are produced only inside the template block
//! (or, for standalone petite-vue HTML and `.art.vue` documents, over the whole
//! file).
//!
//! # Measured differences from `@vue/language-server` 3.3.8
//!
//! Recorded on the same fixture and positions (see
//! `tests/tooling/lsp-selection-range.test.ts`, which pins both sides):
//!
//! - Vize adds levels Vue LS omits: the identifier token, the start tag as a
//!   whole (`<div …>`), the SFC block including its own tags, and the whole
//!   document. Vue LS's chain stops at the block *content*.
//! - Vue LS offers `div class="wrap"` (the start tag interior, without `<` and
//!   `>`) where Vize offers `<div class="wrap">` (the start tag as a unit).
//! - Vue LS returns a statement level inside `<script>` (`const count = 1`)
//!   because it walks the TypeScript AST. Vize does not yet, so a script cursor
//!   gets levels 1, 10, 11, 12. Tracked separately; the authored-range levels
//!   below are unaffected.
//! - Vue LS returns `null` for positions inside `<style>`; Vize still returns
//!   the token/block/document chain there.

mod markup;

use tower_lsp::lsp_types::{Position, Range, SelectionRange};

use self::markup::markup_spans;
use super::{offset_to_position, token_span_at_offset};
use crate::utils::is_standalone_html_path;

pub struct SelectionRangeService;

impl SelectionRangeService {
    /// Build the selection-range chain for `offset` in `content`.
    ///
    /// `filename` is only used to parse the SFC (block boundaries) and to detect
    /// standalone HTML documents; no server state is consulted, which keeps the
    /// provider a pure function of the authored text.
    pub fn selection_range(content: &str, filename: &str, offset: usize) -> Option<SelectionRange> {
        let offset = offset.min(content.len());
        let mut spans = vec![(0, content.len())];

        if let Some((start, end)) = token_span_at_offset(content, offset, is_token_char)
            && start <= offset
            && offset <= end
        {
            spans.push((start, end));
        }

        let markup_region = collect_block_spans(content, filename, offset, &mut spans);
        if let Some(region) = markup_region {
            markup_spans(content, region, offset, &mut spans);
        }

        build_chain(content, spans)
    }
}

/// Push SFC block levels and return the region that should be scanned as markup.
///
/// Returns `None` when the cursor sits in a raw-text block (`<script>`,
/// `<style>`): scanning those as markup would treat `a < b` as a tag.
fn collect_block_spans(
    content: &str,
    filename: &str,
    offset: usize,
    spans: &mut Vec<(usize, usize)>,
) -> Option<(usize, usize)> {
    if is_standalone_html_path(filename) || filename.ends_with(".art.vue") {
        return Some((0, content.len()));
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return Some((0, content.len()));
    };

    let mut markup_region = None;
    let template_loc = descriptor.template.as_ref().map(|block| &block.loc);
    let raw_text_locs = descriptor
        .script
        .as_ref()
        .map(|block| &block.loc)
        .into_iter()
        .chain(descriptor.script_setup.as_ref().map(|block| &block.loc))
        .chain(descriptor.styles.iter().map(|block| &block.loc));

    let mut in_raw_text = false;
    for loc in template_loc.into_iter().chain(raw_text_locs.clone()) {
        if loc.tag_start <= offset && offset <= loc.tag_end {
            spans.push((loc.tag_start, loc.tag_end));
            spans.push((loc.start, loc.end));
        }
    }
    for loc in raw_text_locs {
        if loc.start <= offset && offset <= loc.end {
            in_raw_text = true;
        }
    }
    if let Some(loc) = template_loc
        && loc.start <= offset
        && offset <= loc.end
    {
        markup_region = Some((loc.start, loc.end));
    }

    if in_raw_text {
        return None;
    }
    // Outside every block (for example on a custom block's tag) the whole file
    // is safe to scan: raw-text interiors were already excluded above.
    Some(markup_region.unwrap_or((0, content.len())))
}

#[inline]
fn is_token_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

/// Turn candidate spans into a strictly nested innermost-first chain.
fn build_chain(content: &str, spans: Vec<(usize, usize)>) -> Option<SelectionRange> {
    let mut sorted: Vec<(usize, usize)> = spans
        .into_iter()
        .filter(|(start, end)| start < end)
        .collect();
    sorted.sort_by_key(|(start, end)| (end - start, *start));
    sorted.dedup();

    let mut chain: Vec<(usize, usize)> = Vec::with_capacity(sorted.len());
    for span in sorted {
        match chain.last() {
            None => chain.push(span),
            Some(&(previous_start, previous_end)) => {
                let contains = span.0 <= previous_start && span.1 >= previous_end;
                let strictly_larger = span.0 < previous_start || span.1 > previous_end;
                if contains && strictly_larger {
                    chain.push(span);
                }
            }
        }
    }

    let mut node: Option<Box<SelectionRange>> = None;
    for span in chain.iter().rev() {
        node = Some(Box::new(SelectionRange {
            range: to_range(content, *span),
            parent: node,
        }));
    }
    node.map(|boxed| *boxed)
}

fn to_range(content: &str, span: (usize, usize)) -> Range {
    let (start_line, start_character) = offset_to_position(content, span.0);
    let (end_line, end_character) = offset_to_position(content, span.1);
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}

#[cfg(test)]
mod tests;
