//! Revalidate local primitive initializer judgments before an incomplete edit.
//! A complete, explicitly terminated top-level declaration fixes the lexical
//! context at its right boundary. The entire preceding source must still agree.
use super::{Assignment, CheckedAssignments, GetSpan, SourceBlock, Span};
use alloc::vec::Vec;
use oxc_span::SourceType;
mod frame;

#[derive(Clone, Copy)]
struct Entry {
    /// End of the complete original variable statement relative to the script.
    end: u32,
    row: Assignment,
}

pub struct LiteralPrefixIndex<'old> {
    original: SourceBlock<'old>,
    grammar: SourceType,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixRefusal {
    ForeignSource,
    Grammar,
    DiagnosticDirective,
    InvalidBoundary,
}

impl<'old> LiteralPrefixIndex<'old> {
    /// Reborrow the source independently of the parser arena. Only the original
    /// checked buffer can keep the index after that arena has been released.
    pub fn collect(
        original: SourceBlock<'old>,
        checked: &CheckedAssignments<'_, '_, '_>,
    ) -> Result<Self, PrefixRefusal> {
        let owner = checked.source();
        if !core::ptr::eq(original.root_source(), owner.root_source())
            || original.span() != owner.span()
            || !core::ptr::eq(original.source(), owner.source())
        {
            return Err(PrefixRefusal::ForeignSource);
        }
        let mut entries = Vec::new();
        let mut rows = checked.assignments().peekable();
        let mut sealed = 0;
        let text = original.source().as_bytes();
        for stmt in &checked.original().parsed().program.body {
            let end = stmt.span().end;
            let past = original.start() + end;
            let terminated = end.checked_sub(1).and_then(|at| text.get(at as usize)) == Some(&b';');
            while let Some(row) = rows.next_if(|row| row.binding.start < past) {
                entries.push(Entry { end, row: *row });
            }
            if terminated {
                // A later, unchanged statement fixes the original ASI boundary
                // of any preceding declarations, including their next token.
                for pending in entries.iter_mut().skip(sealed) {
                    pending.end = end;
                }
                sealed = entries.len();
            }
        }
        entries.truncate(sealed);
        Ok(Self {
            original,
            grammar: checked.original().source_type(),
            entries,
        })
    }

    /// Reuse all local bindings up to the last identical, terminated original
    /// declaration. A current suffix can be arbitrarily malformed. This view
    /// only asserts the recovered individual judgments.
    #[expect(
        clippy::indexing_slicing,
        reason = "The length is bounded by both source blocks. Partitioning produces a valid prefix length and its final statement lies in that prefix."
    )]
    pub fn revalidate<'idx, 'new>(
        &'idx self,
        current: SourceBlock<'new>,
        grammar: SourceType,
    ) -> Result<UnchangedLiterals<'idx, 'new>, PrefixRefusal> {
        if grammar != self.grammar {
            return Err(PrefixRefusal::Grammar);
        }
        let last = self.entries.last().map_or(0, |e| e.end as usize);
        let length = last.min(current.source().len());
        let old = &self.original.source().as_bytes()[..length];
        let new = &current.source().as_bytes()[..length];
        let equal = common_bytes(old, new);
        let count = self.entries.partition_point(|e| e.end as usize <= equal);
        let entries = &self.entries[..count];
        let covered = entries.last().map_or(0, |e| e.end);
        // A positive fence ends just after an ASCII semicolon. A directive
        // marker cannot overlap that boundary, even when the edit itself cuts
        // through the marker. Search the unparsed suffix for its first byte.
        let suffix = current
            .source()
            .get(covered as usize..)
            .ok_or(PrefixRefusal::InvalidBoundary)?;
        if suffix.match_indices('@').any(|(at, _)| {
            suffix.as_bytes()[at..]
                .get(..4)
                .is_some_and(|word| word.eq_ignore_ascii_case(b"@ts-"))
        }) {
            return Err(PrefixRefusal::DiagnosticDirective);
        }
        Ok(UnchangedLiterals {
            entries,
            original_start: self.original.start(),
            current,
            covered,
        })
    }
}

fn common_bytes(a: &[u8], b: &[u8]) -> usize {
    let mut offset = 0;
    for (left, right) in a.chunks(64).zip(b.chunks(64)) {
        if left != right {
            return offset + left.iter().zip(right).take_while(|(x, y)| x == y).count();
        }
        offset += left.len();
    }
    offset
}

/// An immutable view tied to the exact new source buffer and the index that
/// verified its prefix. Relative byte extents survive movement of the SFC tag.
pub struct UnchangedLiterals<'idx, 'new> {
    entries: &'idx [Entry],
    original_start: u32,
    current: SourceBlock<'new>,
    covered: u32,
}
impl<'new> UnchangedLiterals<'_, 'new> {
    pub fn current(&self) -> SourceBlock<'new> {
        self.current
    }
    pub fn covered(&self) -> Span {
        Span::new(self.current.start(), self.current.start() + self.covered)
    }
    pub fn assignments(&self) -> impl ExactSizeIterator<Item = Assignment> + '_ {
        self.entries.iter().map(|entry| self.remap(entry.row))
    }
    /// Look up a binding at a UTF-8 cursor boundary in logarithmic time.
    pub fn binding_at(&self, offset: u32) -> Option<Assignment> {
        let relative = offset.checked_sub(self.current.start())?;
        if relative >= self.covered || !self.current.source().is_char_boundary(relative as usize) {
            return None;
        }
        let old = self.original_start + relative;
        let index = self.entries.partition_point(|e| e.row.binding.end <= old);
        let row = self.entries.get(index)?.row;
        (row.binding.start <= old).then(|| self.remap(row))
    }
    fn remap(&self, row: Assignment) -> Assignment {
        let translate = |s: Span| {
            Span::new(
                frame::rebase(s.start, self.original_start, self.current.start()),
                frame::rebase(s.end, self.original_start, self.current.start()),
            )
        };
        Assignment {
            binding: translate(row.binding),
            annotation: translate(row.annotation),
            initializer: translate(row.initializer),
            ..row
        }
    }
}
#[cfg(test)]
mod tests;
