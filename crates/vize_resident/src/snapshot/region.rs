//! The S2 region joint: a template block's root children split into regions
//! that lower independently, and the lowering of one region.
//!
//! A template is **decomposable** when its root children are elements
//! separated only by whitespace runs that contain a newline — the runs Vue's
//! whitespace condensing drops at the root. A `v-else` / `v-else-if` element
//! joins the region of the element before it, so an `if` chain is one region.
//! Anything else at the root (text, an interpolation, a comment, a
//! same-line whitespace run, recovered bytes) makes the template
//! non-decomposable and the block lowers whole. For a decomposable template,
//! lowering each region against the block frame and concatenating the
//! results equals lowering the whole block — the equality TS-42 checks after
//! every step (`EquivalenceReport` runs the snapshot path too).

use vize_davinci::diagnostic::{Diagnostic, Stage};
use vize_s0::{Allocator, SourceRoot, String};
use vize_s1::{SurfaceChild, Token};
use vize_s1_to_s2::{LegacyCaps, lower_source_block_with_caps};
use vize_s2::folio::{FolioOp, S2Folio};

/// One region's syntax: its bytes and block-relative start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionSyntax {
    /// Block-relative byte offset of the region.
    pub start: u32,
    /// The region's bytes.
    pub text: String,
}

/// One region's lowering, with block-relative spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionLowering {
    /// The region's root ops.
    pub ops: Vec<FolioOp>,
    /// Tokenizer findings (`Stage::Surface`), in report order.
    pub surface: Vec<Diagnostic>,
    /// The lowering's own diagnostics, in decision order.
    pub semantic: Vec<Diagnostic>,
}

/// The template's regions, or `None` when it is not decomposable.
#[must_use]
pub fn split_regions(block: &str) -> Option<Vec<RegionSyntax>> {
    let allocator = Allocator::default();
    let (tree, _) = vize_s1::parse(&allocator, block);
    let base = block.as_ptr() as usize;
    let starts: Vec<usize> = tree
        .children
        .iter()
        .map(|child| first_token(child).leading.as_ptr() as usize - base)
        .collect();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (index, (child, &start)) in tree.children.iter().zip(&starts).enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(block.len());
        match child {
            SurfaceChild::Text(token) if is_dropped_whitespace(token) => {}
            SurfaceChild::Element(element) if element.open.lt_name.leading.is_empty() => {
                if continues_chain(child) {
                    ranges.last_mut()?.1 = end;
                } else {
                    ranges.push((start, end));
                }
            }
            _ => return None,
        }
    }
    ranges
        .into_iter()
        .map(|(start, end)| {
            Some(RegionSyntax {
                start: u32::try_from(start).ok()?,
                text: String::from(block.get(start..end)?),
            })
        })
        .collect()
}

/// Lower one region of `block` against the block frame (block-relative spans),
/// or `None` when `region` is not a slice of `block`.
#[must_use]
pub fn lower_region(
    block: &str,
    region: &RegionSyntax,
    caps: LegacyCaps,
) -> Option<RegionLowering> {
    let start = region.start as usize;
    let slice = block.get(start..start + region.text.len())?;
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, slice);
    let frame = SourceRoot::new(block)
        .and_then(|root| root.block(slice, region.start))
        .ok()?;
    let lowered = lower_source_block_with_caps(&allocator, &tree, &errors, frame, caps);
    let (surface, semantic) = lowered
        .diagnostics
        .into_iter()
        .partition(|diagnostic| diagnostic.stage == Stage::Surface);
    Some(RegionLowering {
        ops: S2Folio::of(&lowered.root.ops).ops,
        surface,
        semantic,
    })
}

/// Concatenate region lowerings in document order: every region's ops, then
/// every tokenizer finding, then every lowering diagnostic — the order the
/// whole-block lowering reports them in.
#[must_use]
pub fn assemble<'r>(
    regions: impl Iterator<Item = &'r RegionLowering> + Clone,
) -> (S2Folio, Vec<Diagnostic>) {
    let ops = regions
        .clone()
        .flat_map(|region| region.ops.iter().cloned())
        .collect();
    let diagnostics = regions
        .clone()
        .flat_map(|region| region.surface.iter().cloned())
        .chain(regions.flat_map(|region| region.semantic.iter().cloned()))
        .collect();
    (S2Folio { ops }, diagnostics)
}

fn first_token<'a>(child: &SurfaceChild<'a>) -> Token<'a> {
    match child {
        SurfaceChild::Element(element) => element.open.lt_name,
        SurfaceChild::Interpolation(node) => node.open,
        SurfaceChild::Text(token)
        | SurfaceChild::Comment(token)
        | SurfaceChild::Cdata(token)
        | SurfaceChild::ProcessingInstruction(token)
        | SurfaceChild::Unexpected(token) => *token,
    }
}

fn is_dropped_whitespace(token: &Token<'_>) -> bool {
    token.leading.is_empty() && token.text.trim().is_empty() && token.text.contains('\n')
}

fn continues_chain(child: &SurfaceChild<'_>) -> bool {
    let SurfaceChild::Element(element) = child else {
        return false;
    };
    element
        .open
        .attrs
        .iter()
        .any(|attr| matches!(attr.name.text, "v-else" | "v-else-if"))
}
