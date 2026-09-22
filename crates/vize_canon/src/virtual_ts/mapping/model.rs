//! [`ProjectionMapping`]: the one span-link container every projection
//! consumer reads.

use std::ops::Range;

use super::super::semantic_links::VizeSemanticLink;
use super::rows::{ProjectionFeatures, ProjectionMeta, ProjectionRow, VizeMapping};

/// Span links between one generated virtual document and its authored source.
///
/// Rows keep producer order. Authored ranges are relative to
/// [`Self::authored_base`] (the authored block start for block-local
/// producers, `0` for whole-file producers); every lookup that returns an
/// authored offset adds the base, every lookup that takes one expects it
/// relative.
///
/// Two lookup families share the rows, because they answer different
/// questions:
///
/// - **caret lookups** ([`Self::to_generated`], [`Self::to_authored`] and
///   their `_for` variants) map one byte to one byte: the first row in row
///   order whose half-open range contains the offset, clamped to that row's
///   last byte;
/// - **diagnostic lookups** ([`Self::span_at_generated`],
///   [`Self::diagnostic_range_to_authored`]) map a checker range to authored
///   bytes: the narrowest row wins, an exact sub-span beats its row, and range
///   ends clamp to the authored end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionMapping {
    spans: Vec<VizeMapping>,
    /// Per-row metadata; empty while every row carries the default.
    meta: Vec<ProjectionMeta>,
    semantic_links: Vec<VizeSemanticLink>,
    authored_base: usize,
    /// Rows are sorted by authored start with pairwise disjoint authored
    /// ranges, so a binary search finds the unique containing row.
    authored_disjoint: bool,
}

impl Default for ProjectionMapping {
    fn default() -> Self {
        Self {
            spans: Vec::new(),
            meta: Vec::new(),
            semantic_links: Vec::new(),
            authored_base: 0,
            authored_disjoint: true,
        }
    }
}

impl ProjectionMapping {
    /// An empty mapping.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Rows with default metadata and no semantic links, in producer order.
    pub fn from_spans(spans: Vec<VizeMapping>) -> Self {
        Self::from_parts(spans, Vec::new())
    }

    /// Rows with default metadata plus the producer's semantic links.
    pub fn from_parts(spans: Vec<VizeMapping>, semantic_links: Vec<VizeSemanticLink>) -> Self {
        let authored_disjoint = authored_disjoint(&spans);
        Self {
            spans,
            meta: Vec::new(),
            semantic_links,
            authored_base: 0,
            authored_disjoint,
        }
    }

    /// Append one row with default metadata.
    pub fn push(&mut self, span: VizeMapping) {
        self.push_with(span, ProjectionMeta::default());
    }

    /// Append one row with explicit metadata.
    pub fn push_with(&mut self, span: VizeMapping, meta: ProjectionMeta) {
        self.authored_disjoint = self.authored_disjoint
            && self
                .spans
                .last()
                .is_none_or(|last| last.src_range.end <= span.src_range.start);
        // The metadata column is empty while every row is default, and
        // row-aligned with the spans once any row is not.
        if meta != ProjectionMeta::default() || !self.meta.is_empty() {
            self.meta
                .resize(self.spans.len(), ProjectionMeta::default());
            self.meta.push(meta);
        }
        self.spans.push(span);
    }

    /// Stable-sort rows by authored start, keeping each row's metadata.
    ///
    /// Producers that walk their input in order already emit sorted rows, so
    /// that case is a single comparison pass without allocation.
    pub fn sort_by_authored(&mut self) {
        if self.spans.is_sorted_by_key(|span| span.src_range.start) {
            self.authored_disjoint = authored_disjoint(&self.spans);
            return;
        }
        if self.meta.is_empty() {
            self.spans.sort_by_key(|span| span.src_range.start);
        } else {
            let mut rows: Vec<(VizeMapping, ProjectionMeta)> = core::mem::take(&mut self.spans)
                .into_iter()
                .zip(core::mem::take(&mut self.meta))
                .collect();
            rows.sort_by_key(|(span, _)| span.src_range.start);
            (self.spans, self.meta) = rows.into_iter().unzip();
        }
        self.authored_disjoint = authored_disjoint(&self.spans);
    }

    /// Set the authored offset every row's authored range is relative to.
    #[inline]
    pub fn set_authored_base(&mut self, base: usize) {
        self.authored_base = base;
    }

    /// The authored offset every row's authored range is relative to.
    #[inline]
    pub fn authored_base(&self) -> usize {
        self.authored_base
    }

    /// Rows in producer order.
    #[inline]
    pub fn spans(&self) -> &[VizeMapping] {
        &self.spans
    }

    /// Semantic links between generated ranges.
    #[inline]
    pub fn semantic_links(&self) -> &[VizeSemanticLink] {
        &self.semantic_links
    }

    /// Metadata of the row at `index`.
    #[inline]
    pub fn meta(&self, index: usize) -> ProjectionMeta {
        self.meta.get(index).copied().unwrap_or_default()
    }

    /// Rows with their metadata, in producer order.
    pub fn rows(&self) -> impl ExactSizeIterator<Item = ProjectionRow<'_>> + '_ {
        self.spans
            .iter()
            .enumerate()
            .map(|(index, span)| ProjectionRow {
                span,
                meta: self.meta(index),
            })
    }

    /// Rows whose half-open authored range contains `authored`.
    pub fn rows_containing_authored(
        &self,
        authored: usize,
    ) -> impl Iterator<Item = ProjectionRow<'_>> + '_ {
        self.rows()
            .filter(move |row| row.span.src_range.contains(&authored))
    }

    /// Number of rows.
    #[inline]
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Whether the mapping has no rows.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// The rows and semantic links, dropping metadata and base.
    pub fn into_parts(self) -> (Vec<VizeMapping>, Vec<VizeSemanticLink>) {
        (self.spans, self.semantic_links)
    }

    /// Caret lookup: the generated byte for authored byte `authored`.
    pub fn to_generated(&self, authored: usize) -> Option<usize> {
        let span = if self.authored_disjoint {
            let index = self
                .spans
                .binary_search_by(|span| {
                    if span.src_range.end <= authored {
                        core::cmp::Ordering::Less
                    } else if span.src_range.start > authored {
                        core::cmp::Ordering::Greater
                    } else {
                        core::cmp::Ordering::Equal
                    }
                })
                .ok()?;
            &self.spans[index]
        } else {
            self.spans
                .iter()
                .find(|span| span.src_range.contains(&authored))?
        };
        Some(clamped_byte(&span.src_range, &span.gen_range, authored))
    }

    /// Caret lookup restricted to rows serving `feature`.
    pub fn to_generated_for(&self, authored: usize, feature: ProjectionFeatures) -> Option<usize> {
        if self.meta.is_empty() {
            return self.to_generated(authored);
        }
        let row = self.rows().find(|row| {
            row.span.src_range.contains(&authored) && row.meta.features.contains(feature)
        })?;
        Some(clamped_byte(
            &row.span.src_range,
            &row.span.gen_range,
            authored,
        ))
    }

    /// Caret lookup: the authored byte (base included) for generated byte
    /// `generated`.
    pub fn to_authored(&self, generated: usize) -> Option<usize> {
        let span = self
            .spans
            .iter()
            .find(|span| span.gen_range.contains(&generated))?;
        Some(clamped_byte(&span.gen_range, &span.src_range, generated) + self.authored_base)
    }

    /// Caret lookup restricted to rows serving `feature`.
    pub fn to_authored_for(&self, generated: usize, feature: ProjectionFeatures) -> Option<usize> {
        let row = self.rows().find(|row| {
            row.span.gen_range.contains(&generated) && row.meta.features.contains(feature)
        })?;
        Some(clamped_byte(&row.span.gen_range, &row.span.src_range, generated) + self.authored_base)
    }

    /// Caret lookup of a generated range: first and last byte mapped
    /// independently, the end kept exclusive.
    pub fn generated_range_to_authored(&self, generated: Range<usize>) -> Option<Range<usize>> {
        let start = self.to_authored(generated.start)?;
        let end = self.to_authored(generated.end.saturating_sub(1))? + 1;
        Some(start..end)
    }

    /// Diagnostic lookup: the narrowest row whose generated range contains
    /// `generated`.
    #[inline]
    pub fn span_at_generated(&self, generated: usize) -> Option<&VizeMapping> {
        super::mapping_for_generated_offset(&self.spans, generated)
    }

    /// Diagnostic lookup of a checker range (base included); see
    /// [`super::map_generated_range_to_source`].
    pub fn diagnostic_range_to_authored(&self, start: usize, end: usize) -> Option<(usize, usize)> {
        let (start, end) = super::map_generated_range_to_source(&self.spans, start, end)?;
        Some((start + self.authored_base, end + self.authored_base))
    }
}

/// `offset` in `from` moved to the same relative byte of `to`, clamped to
/// `to`'s last byte.
#[inline]
fn clamped_byte(from: &Range<usize>, to: &Range<usize>, offset: usize) -> usize {
    let relative = offset - from.start;
    to.start + relative.min(to.end.saturating_sub(to.start).saturating_sub(1))
}

fn authored_disjoint(spans: &[VizeMapping]) -> bool {
    spans
        .windows(2)
        .all(|pair| pair[0].src_range.end <= pair[1].src_range.start)
}
