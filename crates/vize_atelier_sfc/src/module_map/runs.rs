//! Byte-identity provenance between a derived text and its origin.

/// `len` bytes at `out` in the derived text are the bytes at `src` in the
/// origin text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Run {
    pub(crate) out: usize,
    pub(crate) src: usize,
    pub(crate) len: usize,
}

/// Where a derived text came from.
///
/// `runs` are verbatim copies, appended in derived-text order. `points` anchor
/// text that was regenerated from a known origin rather than copied (a
/// rewritten identifier, a rebuilt import, a synthesized macro binding): the
/// derived byte at `out` stands for the origin byte at `src`, and nothing is
/// claimed about the bytes after it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Runs {
    runs: Vec<Run>,
    points: Vec<(usize, usize)>,
}

impl Runs {
    /// The whole of a `len`-byte text copied from offset 0 of its origin.
    pub(crate) fn identity(len: usize) -> Self {
        let mut runs = Self::default();
        runs.copy(0, 0, len);
        runs
    }

    pub(crate) fn runs(&self) -> &[Run] {
        &self.runs
    }

    pub(crate) fn points(&self) -> &[(usize, usize)] {
        &self.points
    }

    /// Record that `len` bytes at `out` were copied from `src`.
    ///
    /// Calls must arrive in derived-text order; a run that continues the
    /// previous one in both texts extends it.
    pub(crate) fn copy(&mut self, out: usize, src: usize, len: usize) {
        if len == 0 {
            return;
        }
        if let Some(last) = self.runs.last_mut()
            && last.out + last.len == out
            && last.src + last.len == src
        {
            last.len += len;
            return;
        }
        debug_assert!(
            self.runs
                .last()
                .is_none_or(|last| last.out + last.len <= out),
            "runs must be appended in derived-text order"
        );
        self.runs.push(Run { out, src, len });
    }

    /// Record that the derived byte at `out` was regenerated from `src`.
    pub(crate) fn point(&mut self, out: usize, src: usize) {
        self.points.push((out, src));
    }

    /// Append `other`, whose offsets are relative to its own text, at `at`.
    pub(crate) fn append(&mut self, at: usize, other: &Runs) {
        for run in &other.runs {
            self.copy(at + run.out, run.src, run.len);
        }
        for &(out, src) in &other.points {
            self.point(at + out, src);
        }
    }

    /// The provenance of the derived bytes `start..start + len`, rebased so
    /// that `start` becomes offset 0.
    pub(crate) fn slice(&self, start: usize, len: usize) -> Runs {
        let end = start + len;
        let mut sliced = Runs::default();
        let first = self.runs.partition_point(|run| run.out + run.len <= start);
        let tail = self.runs.get(first..).unwrap_or_default();
        for run in tail.iter().take_while(|run| run.out < end) {
            let lo = run.out.max(start);
            let hi = (run.out + run.len).min(end);
            if lo < hi {
                sliced.copy(lo - start, run.src + (lo - run.out), hi - lo);
            }
        }
        for &(out, src) in &self.points {
            if (start..end).contains(&out) {
                sliced.point(out - start, src);
            }
        }
        sliced
    }

    /// Chain two stages: `self` maps this text into B, `lower` maps B into C;
    /// the result maps this text into C. Bytes `lower` does not account for
    /// drop out.
    pub(crate) fn compose(&self, lower: &Runs) -> Runs {
        let mut composed = Runs::default();
        for run in &self.runs {
            composed.append(run.out, &lower.slice(run.src, run.len));
        }
        for &(out, src) in &self.points {
            if let Some(origin) = lower.lookup(src) {
                composed.point(out, origin);
            }
        }
        composed
    }

    /// The origin offset of derived byte `pos`, when it is accounted for.
    pub(crate) fn lookup(&self, pos: usize) -> Option<usize> {
        let index = self.runs.partition_point(|run| run.out + run.len <= pos);
        if let Some(run) = self.runs.get(index)
            && run.out <= pos
        {
            return Some(run.src + (pos - run.out));
        }
        self.points
            .iter()
            .find(|&&(out, _)| out == pos)
            .map(|&(_, src)| src)
    }

    /// Move every origin offset by `base` (a block's position in its file).
    pub(crate) fn offset_origin(&self, base: usize) -> Runs {
        let mut moved = Runs::default();
        for run in &self.runs {
            moved.copy(run.out, run.src + base, run.len);
        }
        for &(out, src) in &self.points {
            moved.point(out, src + base);
        }
        moved
    }

    /// Keep only the runs whose bytes really are identical in both texts and
    /// the points that land inside both. Every recorded run is identical by
    /// construction; this makes a construction defect drop a mapping instead of
    /// publishing a wrong one.
    pub(crate) fn verified(self, derived: &str, origin: &str) -> Runs {
        let mut kept = Runs::default();
        for run in self.runs {
            let copied = derived.get(run.out..run.out + run.len);
            if copied.is_some() && copied == origin.get(run.src..run.src + run.len) {
                kept.copy(run.out, run.src, run.len);
            }
        }
        for (out, src) in self.points {
            if derived.is_char_boundary(out)
                && out < derived.len()
                && origin.is_char_boundary(src)
                && src < origin.len()
            {
                kept.point(out, src);
            }
        }
        kept
    }
}

/// `text.replace(from, to)` with the provenance of the result: every byte
/// outside a replacement is a copy, and each replacement is anchored at the
/// text it replaced.
pub(crate) fn replace_traced(text: &str, from: &str, to: &str) -> (vize_carton::String, Runs) {
    let mut replaced = vize_carton::String::with_capacity(text.len());
    let mut runs = Runs::default();
    let mut cursor = 0;
    for (start, _) in text.match_indices(from) {
        runs.copy(replaced.len(), cursor, start - cursor);
        replaced.push_str(text.get(cursor..start).unwrap_or_default());
        runs.point(replaced.len(), start);
        replaced.push_str(to);
        cursor = start + from.len();
    }
    runs.copy(replaced.len(), cursor, text.len() - cursor);
    replaced.push_str(text.get(cursor..).unwrap_or_default());
    (replaced, runs)
}

/// The provenance of `derived`, which was built by replacing `edits`
/// (`start..end` of `origin`, each replaced by `len` bytes; sorted, disjoint)
/// in `origin`.
pub(crate) fn edit_runs(origin_len: usize, edits: &[(usize, usize, usize)]) -> Runs {
    let mut runs = Runs::default();
    let (mut cursor, mut out) = (0, 0);
    for &(start, end, len) in edits {
        runs.copy(out, cursor, start - cursor);
        out += start - cursor;
        if len > 0 {
            runs.point(out, start);
        }
        out += len;
        cursor = end;
    }
    runs.copy(out, cursor, origin_len - cursor);
    runs
}

/// Replace sorted `edits` (`start..end` of `origin` by the text) in `origin`,
/// skipping an edit that starts inside an earlier one, and return the result
/// with its provenance: untouched text is copied and each non-empty
/// replacement is anchored at the text it replaced.
pub(crate) fn apply_edits<'e>(
    origin: &str,
    edits: impl IntoIterator<Item = (usize, usize, &'e str)>,
) -> (vize_carton::String, Runs) {
    let mut out = vize_carton::String::with_capacity(origin.len());
    let mut runs = Runs::default();
    let mut cursor = 0;
    for (start, end, replacement) in edits {
        if start < cursor {
            continue;
        }
        runs.copy(out.len(), cursor, start - cursor);
        out.push_str(origin.get(cursor..start).unwrap_or_default());
        if !replacement.is_empty() {
            runs.point(out.len(), start);
        }
        out.push_str(replacement);
        cursor = end;
    }
    runs.copy(out.len(), cursor, origin.len() - cursor);
    out.push_str(origin.get(cursor..).unwrap_or_default());
    (out, runs)
}
