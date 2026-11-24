//! Match retained recovery records to sorted original byte positions.
use super::TemplateRecovery;

mod window;
use core::{
    iter,
    ops::{ControlFlow, Range},
};
use vize_l0::{
    Span,
    diag::{Diagnostic, DiagnosticPart, WitnessLink},
};
use vize_l1_to_l2::native::NativeHole;
use window::matching_positions;

/// Each value borrows the original lowering, including the parent of a related site.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryRecord<'db> {
    Hole(&'db NativeHole),
    Diagnostic(&'db Diagnostic),
    Part {
        diagnostic: &'db Diagnostic,
        part: &'db DiagnosticPart,
    },
    Witness {
        diagnostic: &'db Diagnostic,
        link: &'db WitnessLink,
    },
}

impl<'db> RecoveryRecord<'db> {
    fn for_diagnostic(diagnostic: &'db Diagnostic) -> impl Iterator<Item = Self> {
        let parts = diagnostic
            .parts
            .iter()
            .map(move |part| Self::Part { diagnostic, part });
        let links = diagnostic
            .witness_chain()
            .into_iter()
            .flat_map(|chain| chain.links())
            .map(move |link| Self::Witness { diagnostic, link });
        iter::once(Self::Diagnostic(diagnostic))
            .chain(parts)
            .chain(links)
    }

    /// Range relative to the observed template's original UTF-8 text.
    #[must_use]
    pub fn relative_span(self) -> Span {
        match self {
            Self::Hole(hole) => hole.span,
            Self::Diagnostic(diagnostic) => diagnostic.span,
            Self::Part { part, .. } => part.span,
            Self::Witness { link, .. } => link.span,
        }
    }
}

/// One source record and the range of indices into the caller's cursor list it covers.
pub struct RecoveryMatch<'db> {
    pub record: RecoveryRecord<'db>,
    pub authored: Span,
    pub text: &'db str,
    pub positions: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryPositionError {
    OutsideSource(u32),
    OutOfOrder { previous: u32, next: u32 },
    InvalidRecord(Span),
}

impl<'db> TemplateRecovery<'db> {
    /// Visit holes, then each diagnostic's primary span, parts and witness links,
    /// in original producer order. Offsets are file-absolute UTF-8 boundaries
    /// in this block. Nonempty ranges are half-open and empty ranges match
    /// their insertion position, including EOF. Duplicate cursors are retained.
    /// All input positions are validated before the first callback.
    pub fn visit_positions<B>(
        &self,
        positions: &[u32],
        visit: impl FnMut(RecoveryMatch<'db>) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, RecoveryPositionError> {
        let Some((&first, rest)) = positions.split_first() else {
            return Ok(ControlFlow::Continue(()));
        };
        self.validate_position(first)?;
        if rest.is_empty() {
            return self.visit_matching(
                |span| window::matching_position(first, span.start, span.end),
                visit,
            );
        }
        let mut previous = first;
        for &next in rest {
            if next == previous {
                continue;
            }
            self.validate_position(next)?;
            if next < previous {
                return Err(RecoveryPositionError::OutOfOrder { previous, next });
            }
            previous = next;
        }
        match *positions {
            [a, b] => self.visit_short([a, b], visit),
            [a, b, c] => self.visit_short([a, b, c], visit),
            [a, b, c, d] => self.visit_short([a, b, c, d], visit),
            _ => self.visit_matching(
                |span| matching_positions(positions, span.start, span.end),
                visit,
            ),
        }
    }

    #[inline]
    fn visit_short<const N: usize, B>(
        &self,
        points: [u32; N],
        visit: impl FnMut(RecoveryMatch<'db>) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, RecoveryPositionError> {
        self.visit_matching(
            |span| window::matching_short(&points, span.start, span.end),
            visit,
        )
    }

    #[inline]
    fn validate_position(&self, position: u32) -> Result<(), RecoveryPositionError> {
        if position
            .checked_sub(self.start)
            .is_some_and(|local| self.text.is_char_boundary(local as usize))
        {
            Ok(())
        } else {
            Err(RecoveryPositionError::OutsideSource(position))
        }
    }

    #[inline]
    fn visit_matching<B>(
        &self,
        matches: impl Fn(Span) -> Range<usize>,
        mut visit: impl FnMut(RecoveryMatch<'db>) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, RecoveryPositionError> {
        let holes = self.holes.iter().map(RecoveryRecord::Hole);
        let diagnostics = self
            .diagnostics
            .iter()
            .flat_map(RecoveryRecord::for_diagnostic);
        for record in holes.chain(diagnostics) {
            let relative = record.relative_span();
            let (authored, text) = self
                .source_site(relative)
                .map_err(|_| RecoveryPositionError::InvalidRecord(relative))?;
            let hits = matches(authored);
            if !hits.is_empty()
                && let ControlFlow::Break(value) = visit(RecoveryMatch {
                    record,
                    authored,
                    text,
                    positions: hits,
                })
            {
                return Ok(ControlFlow::Break(value));
            }
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[cfg(test)]
mod tests;
