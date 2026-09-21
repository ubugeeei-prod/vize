//! A text assembled from copies of an origin plus synthesized pieces.

use vize_carton::String;

use super::runs::Runs;

/// Records the provenance of every copy of `origin` as the text grows, so a
/// pass that splices its input keeps token-level source-map provenance.
pub(crate) struct TracedText<'o> {
    origin: &'o str,
    text: String,
    runs: Runs,
}

impl<'o> TracedText<'o> {
    pub(crate) fn with_capacity(origin: &'o str, capacity: usize) -> Self {
        Self {
            origin,
            text: String::with_capacity(capacity),
            runs: Runs::default(),
        }
    }

    /// Append `origin[start..end]` as a copy.
    pub(crate) fn copy(&mut self, start: usize, end: usize) {
        self.runs.copy(self.text.len(), start, end - start);
        self.text.push_str(&self.origin[start..end]);
    }

    /// Append synthesized text.
    pub(crate) fn push_str(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// Anchor the next appended byte at `origin` offset `at`.
    pub(crate) fn mark(&mut self, at: usize) {
        self.runs.point(self.text.len(), at);
    }

    pub(crate) fn into_parts(self) -> (String, Runs) {
        (self.text, self.runs)
    }
}
