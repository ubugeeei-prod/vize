//! Range membership for the mapper's ordered prop-echo admission pass.

use std::collections::BTreeMap;

use super::ContentMapperDiagnosticDirective;

/// Keep short SFCs on their allocation-free linear membership path.
pub(super) const INDEX_THRESHOLD: usize = 256;

/// Existing directives may overlap. Their union is disjoint before new
/// echoes arrive; every admitted echo is disjoint from it and earlier echoes.
/// Touching intervals stay separate so an empty probe at their shared boundary
/// keeps the exact strict-inequality behavior of the original predicate.
pub(super) struct EchoRanges(Option<BTreeMap<usize, usize>>);

impl EchoRanges {
    pub(super) fn new(existing: &[ContentMapperDiagnosticDirective], echoes: usize) -> Self {
        if existing.len().saturating_add(echoes) <= INDEX_THRESHOLD {
            return Self(None);
        }
        let mut sorted = existing
            .iter()
            .map(|directive| (directive.0[2], directive.0[3]))
            .collect::<Vec<_>>();
        sorted.sort_unstable();
        let mut union: Vec<(usize, usize)> = Vec::with_capacity(sorted.len());
        for (start, end) in sorted {
            if let Some(previous) = union.last_mut()
                && (start < previous.1 || start == previous.0)
            {
                previous.1 = previous.1.max(end);
            } else {
                union.push((start, end));
            }
        }
        Self(Some(union.into_iter().collect()))
    }

    pub(super) fn overlaps(
        &self,
        existing: &[ContentMapperDiagnosticDirective],
        start: usize,
        end: usize,
    ) -> bool {
        match &self.0 {
            Some(index) => index
                .range(..end)
                .next_back()
                .is_some_and(|(_, existing_end)| start < *existing_end),
            None => existing.iter().any(|directive| {
                let [_, _, existing_start, existing_end, _, _] = directive.0;
                start < existing_end && existing_start < end
            }),
        }
    }

    /// Call only after `overlaps` rejected every existing interval.
    pub(super) fn insert(&mut self, start: usize, end: usize) {
        if let Some(index) = &mut self.0 {
            index.insert(start, end);
        }
    }
}

#[cfg(test)]
mod tests;
