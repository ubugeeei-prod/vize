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

    /// A linear row is split into its copied runs; dropped `<`/`>` bytes and
    /// inserted text stay unmapped except for diagnostic-only anchors. Other
    /// rows keep their authored range.
    fn move_row(
        &self,
        row: VizeMapping,
        row_meta: ProjectionMeta,
        rows: &mut Vec<VizeMapping>,
        meta: &mut Vec<ProjectionMeta>,
    ) {
        let linear = !row.gen_range.is_empty() && row.gen_range.len() == row.src_range.len();
        if !linear || self.exact(&row.gen_range).is_some() {
            rows.push(VizeMapping {
                gen_range: self.range(&row.gen_range).0,
                sub_spans: self.sub_spans(&row, &row.gen_range),
                src_range: row.src_range,
            });
            meta.push(row_meta);
            return;
        }
        let first = rows.len();
        let anchors = self
            .anchors
            .iter()
            .skip(
                self.anchors
                    .partition_point(|(old, _)| *old < row.gen_range.start),
            )
            .take_while(|(old, _)| *old < row.gen_range.end);
        for &(old, new) in anchors {
            let source = row.src_range.start + (old - row.gen_range.start);
            rows.push(VizeMapping::new(new..new + 1, source..source + 1));
        }
        let anchored = rows.len() - first;
        let start = self
            .pieces
            .partition_point(|piece| piece.old.end <= row.gen_range.start);
        for piece in self.pieces.iter().skip(start) {
            if piece.old.start >= row.gen_range.end {
                break;
            }
            let part =
                piece.old.start.max(row.gen_range.start)..piece.old.end.min(row.gen_range.end);
            let source = row.src_range.start + (part.start - row.gen_range.start);
            rows.push(VizeMapping {
                gen_range: piece.place(&part),
                src_range: source..source + part.len(),
                sub_spans: self.sub_spans(&row, &part),
            });
        }
        let diagnostics = ProjectionMeta {
            features: ProjectionFeatures::DIAGNOSTICS,
            ..row_meta
        };
        meta.resize(meta.len() + anchored, diagnostics);
        meta.resize(meta.len() + (rows.len() - first - anchored), row_meta);
    }

    fn sub_spans(&self, row: &VizeMapping, part: &Range<usize>) -> Vec<VizeSubSpan> {
        row.sub_spans
            .iter()
            .filter(|span| part.start <= span.gen_range.start && span.gen_range.end <= part.end)
            .filter_map(|span| {
                Some(VizeSubSpan {
                    gen_range: self.exact(&span.gen_range)?,
                    src_range: span.src_range.clone(),
                })
            })
            .collect()
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
