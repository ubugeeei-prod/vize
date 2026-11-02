//! Text and coordinate movement for one TSX syntax rewrite.
//!
//! The new text is a list of copied old runs and inserted bytes. A range inside
//! one run moves exactly; a range across an edit covers the rewritten region.

use std::ops::Range;

use oxc_span::Span;
use vize_carton::String as CompactString;

use super::{Coordinates, Edit};
use crate::virtual_ts::{ProjectionFeatures, ProjectionMeta, VizeMapping, VizeSubSpan};

/// Copied run of old text and its new start.
#[derive(Clone, Debug)]
struct Piece {
    old: Range<usize>,
    new_start: usize,
}

impl Piece {
    fn place(&self, range: &Range<usize>) -> Range<usize> {
        let start = self.new_start + (range.start - self.old.start);
        start..start + range.len()
    }
}

/// Old and new extent of one outermost edit.
#[derive(Clone, Debug)]
struct Region {
    old: Range<usize>,
    new: Range<usize>,
}

pub(super) struct Rewrite {
    text: CompactString,
    /// Disjoint, sorted by old start.
    pieces: Vec<Piece>,
    /// Outermost edits, sorted by old start.
    regions: Vec<Region>,
    /// A dropped `<` and the new `(` that starts its assertion node.
    anchors: Vec<(usize, usize)>,
}

struct Builder<'s> {
    code: &'s str,
    text: CompactString,
    pieces: Vec<Piece>,
    regions: Vec<Region>,
    anchors: Vec<(usize, usize)>,
}

impl Builder<'_> {
    fn copy(&mut self, old: Range<usize>) -> Option<()> {
        let text = self.code.get(old.clone())?;
        if !old.is_empty() {
            self.pieces.push(Piece {
                old,
                new_start: self.text.len(),
            });
            self.text.push_str(text);
        }
        Some(())
    }

    fn emit(&mut self, range: Range<usize>, edits: &[Edit], outermost: bool) -> Option<()> {
        let mut cursor = range.start;
        let mut index = 0;
        while let Some(&edit) = edits.get(index) {
            let new_start;
            let old;
            match edit {
                Edit::Comma(at) => {
                    let at = at as usize;
                    self.copy(cursor..at)?;
                    new_start = self.text.len();
                    self.text.push(',');
                    old = at..at;
                    index += 1;
                }
                Edit::Assertion {
                    whole,
                    ty,
                    expression,
                } => {
                    let whole = span_range(whole);
                    let nested = edits
                        .get(index + 1..)?
                        .iter()
                        .take_while(|edit| (edit.start() as usize) < whole.end)
                        .count();
                    let inner = edits.get(index + 1..index + 1 + nested)?;
                    self.copy(cursor..whole.start)?;
                    new_start = self.text.len();
                    self.text.push('(');
                    self.anchors.push((whole.start, self.text.len()));
                    self.text.push('(');
                    let expression = span_range(expression);
                    self.emit(expression.clone(), &within(inner, &expression), false)?;
                    self.text.push_str(") as ");
                    let ty = span_range(ty);
                    self.emit(ty.clone(), &within(inner, &ty), false)?;
                    self.text.push(')');
                    old = whole;
                    index += 1 + nested;
                }
            }
            cursor = old.end;
            if outermost {
                self.regions.push(Region {
                    old,
                    new: new_start..self.text.len(),
                });
            }
        }
        self.copy(cursor..range.end)
    }
}

fn span_range(span: Span) -> Range<usize> {
    span.start as usize..span.end as usize
}

fn within(edits: &[Edit], range: &Range<usize>) -> Vec<Edit> {
    edits
        .iter()
        .copied()
        .filter(|edit| range.contains(&(edit.start() as usize)))
        .collect()
}

fn is_linear(generated: &Range<usize>, source: &Range<usize>) -> bool {
    !generated.is_empty() && generated.len() == source.len()
}

impl Rewrite {
    pub(super) fn build(code: &str, edits: &[Edit]) -> Option<Self> {
        let mut builder = Builder {
            code,
            text: CompactString::with_capacity(code.len() + edits.len() * 8),
            pieces: Vec::new(),
            regions: Vec::new(),
            anchors: Vec::new(),
        };
        builder.emit(0..code.len(), edits, true)?;
        let Builder {
            text,
            mut pieces,
            regions,
            mut anchors,
            ..
        } = builder;
        pieces.sort_by_key(|piece| piece.old.start);
        anchors.sort_unstable();
        Some(Self {
            text,
            pieces,
            regions,
            anchors,
        })
    }

    pub(super) fn apply(self, code: &mut CompactString, coordinates: Coordinates<'_>) {
        let Coordinates {
            mappings,
            mapping_meta,
            semantic_links,
            prop_default_key_links,
            diagnostic_ownership,
        } = coordinates;
        let mut rows = Vec::with_capacity(mappings.len() + self.regions.len() * 5);
        let mut meta = Vec::with_capacity(rows.capacity());
        for (index, row) in std::mem::take(mappings).into_iter().enumerate() {
            let row_meta = mapping_meta.get(index).copied().unwrap_or_default();
            self.move_row(row, row_meta, &mut rows, &mut meta);
        }
        if mapping_meta.is_empty() && meta.iter().all(|meta| *meta == ProjectionMeta::default()) {
            meta.clear();
        }
        *mappings = rows;
        *mapping_meta = meta;
        for link in semantic_links.iter_mut().chain(prop_default_key_links) {
            link.source_range = self.range(&link.source_range).0;
            link.target_range = self.range(&link.target_range).0;
        }
        if let Some(collector) = diagnostic_ownership {
            collector
                .borrow_mut()
                .note_generated_rewrite(|range| self.range(range));
        }
        *code = self.text;
    }

    /// A row moved as one run shifts. A linear row is split into its copied
    /// runs, and a row with sub-spans splits each linear sub-span; dropped
    /// `<`/`>` bytes and inserted text stay unmapped except for
    /// diagnostic-only anchors. Any other row keeps its authored range.
    fn move_row(
        &self,
        row: VizeMapping,
        row_meta: ProjectionMeta,
        rows: &mut Vec<VizeMapping>,
        meta: &mut Vec<ProjectionMeta>,
    ) {
        let linear = is_linear(&row.gen_range, &row.src_range);
        let mut anchors = Vec::new();
        if self.exact(&row.gen_range).is_some() || (!linear && row.sub_spans.is_empty()) {
            rows.push(VizeMapping {
                gen_range: self.range(&row.gen_range).0,
                sub_spans: row
                    .sub_spans
                    .iter()
                    .filter_map(|span| self.exact_sub_span(span))
                    .collect(),
                src_range: row.src_range,
            });
            meta.push(row_meta);
        } else if row.sub_spans.is_empty() {
            self.anchor_rows(&row.gen_range, row.src_range.start, &mut anchors);
            let pieces = self.split(&row.gen_range, row.src_range.start);
            meta.resize(meta.len() + pieces.len(), row_meta);
            rows.extend(
                pieces
                    .into_iter()
                    .map(|(generated, source)| VizeMapping::new(generated, source)),
            );
        } else {
            let mut sub_spans = Vec::with_capacity(row.sub_spans.len() + 2);
            for span in &row.sub_spans {
                if is_linear(&span.gen_range, &span.src_range) {
                    self.anchor_rows(&span.gen_range, span.src_range.start, &mut anchors);
                    sub_spans.extend(
                        self.split(&span.gen_range, span.src_range.start)
                            .into_iter()
                            .map(|(gen_range, src_range)| VizeSubSpan {
                                gen_range,
                                src_range,
                            }),
                    );
                } else {
                    sub_spans.extend(self.exact_sub_span(span));
                }
            }
            rows.push(VizeMapping {
                gen_range: self.range(&row.gen_range).0,
                src_range: row.src_range,
                sub_spans,
            });
            meta.push(row_meta);
        }
        let diagnostics = ProjectionMeta {
            features: ProjectionFeatures::DIAGNOSTICS,
            ..row_meta
        };
        meta.resize(meta.len() + anchors.len(), diagnostics);
        rows.extend(anchors);
    }

    /// The copied runs of a linear old range, with their authored bytes.
    fn split(&self, old: &Range<usize>, source: usize) -> Vec<(Range<usize>, Range<usize>)> {
        let first = self
            .pieces
            .partition_point(|piece| piece.old.end <= old.start);
        self.pieces
            .iter()
            .skip(first)
            .take_while(|piece| piece.old.start < old.end)
            .map(|piece| {
                let part = piece.old.start.max(old.start)..piece.old.end.min(old.end);
                let authored = source + (part.start - old.start);
                (piece.place(&part), authored..authored + part.len())
            })
            .collect()
    }

    /// One-byte rows from each assertion's new `(` to its authored `<`.
    fn anchor_rows(&self, old: &Range<usize>, source: usize, rows: &mut Vec<VizeMapping>) {
        let first = self
            .anchors
            .partition_point(|(anchor, _)| *anchor < old.start);
        for &(anchor, new) in self
            .anchors
            .iter()
            .skip(first)
            .take_while(|(anchor, _)| *anchor < old.end)
        {
            let authored = source + (anchor - old.start);
            rows.push(VizeMapping::new(new..new + 1, authored..authored + 1));
        }
    }

    fn exact_sub_span(&self, span: &VizeSubSpan) -> Option<VizeSubSpan> {
        Some(VizeSubSpan {
            gen_range: self.exact(&span.gen_range)?,
            src_range: span.src_range.clone(),
        })
    }

    /// The new range of a range copied as one run.
    fn exact(&self, range: &Range<usize>) -> Option<Range<usize>> {
        let piece = self.piece_at(range.start)?;
        (range.end <= piece.old.end).then(|| piece.place(range))
    }

    /// The new range, and whether it moved as one copied run. A range across
    /// an edit covers the whole rewritten region.
    fn range(&self, range: &Range<usize>) -> (Range<usize>, bool) {
        if let Some(moved) = self.exact(range) {
            return (moved, true);
        }
        let start = self.start(range.start);
        let end = if range.is_empty() {
            start
        } else {
            self.end(range.end).max(start)
        };
        (start..end, false)
    }

    fn piece_at(&self, offset: usize) -> Option<&Piece> {
        let index = self
            .pieces
            .partition_point(|piece| piece.old.start <= offset);
        index
            .checked_sub(1)
            .and_then(|index| self.pieces.get(index))
            .filter(|piece| offset < piece.old.end)
    }

    fn region(&self, contains: impl Fn(&Region) -> bool) -> Option<&Region> {
        self.regions.iter().find(|region| contains(region))
    }

    fn start(&self, offset: usize) -> usize {
        if let Some(region) =
            self.region(|region| region.old.start <= offset && offset < region.old.end)
        {
            return region.new.start;
        }
        self.piece_at(offset).map_or(self.text.len(), |piece| {
            piece.place(&(offset..offset)).start
        })
    }

    fn end(&self, offset: usize) -> usize {
        if let Some(region) =
            self.region(|region| region.old.start < offset && offset <= region.old.end)
        {
            return region.new.end;
        }
        let index = self
            .pieces
            .partition_point(|piece| piece.old.start < offset);
        index
            .checked_sub(1)
            .and_then(|index| self.pieces.get(index))
            .filter(|piece| offset <= piece.old.end)
            .map_or(0, |piece| piece.place(&(offset..offset)).start)
    }
}
