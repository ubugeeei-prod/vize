//! Trivia a command-line check never reads, taken out of its projection.
//!
//! A check projection is about 15% comments and indentation: `@vize-map`
//! notes, section banners, the explanation after `@ts-ignore`, and the
//! indentation of generated lines. The host decodes every reply byte, on a
//! single thread in tsrs. A command-line check reads only diagnostics, and a
//! bind or check diagnostic starts at a token or at a directive comment,
//! never in other trivia. So the trivia between two tokens that no span
//! covers collapses to one line break where it had one, else one space where
//! it had one. Block comments and directive-like line comments (`///`, `#`,
//! `@…` other than `@vize-map`) stay, `@ts-ignore` keeps only its directive,
//! and automatic semicolon insertion and `@ts-ignore`'s next-line rule see
//! the same line breaks. Spans and directives move with the text.
//!
//! `trivia_scan` says where trivia is; a projection it is unsure of is left
//! as it is.

use vize_canon::ContentMapperTransform;

use super::trivia_scan::{Comment, trivia};

/// Original bytes copied as they are to `new_start` in the compacted text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Kept {
    start: usize,
    end: usize,
    new_start: usize,
}

/// `transform` for a geometry-only host: verbatim runs joined, and trivia
/// compacted when the host emits nothing (emitted JavaScript and declarations
/// keep comments).
pub(super) fn for_host(
    mut transform: ContentMapperTransform,
    settings: &super::super::project::ProjectSettings,
) -> ContentMapperTransform {
    if settings.transform_options.diagnostics_only() {
        if settings.no_emit {
            compact(&mut transform);
        }
        join_verbatim_runs(&mut transform.mappings);
    }
    transform
}

/// Verbatim kind in the span tuple.
const VERBATIM: usize = 0;

/// Joins each verbatim span with the next when both its generated and its
/// authored range continue there. A geometry-only host maps a position or a
/// range to the same authored range through one segment as through two, and
/// only tells a mapped range from a synthesized one; the editor projection
/// keeps the split, where features can differ.
pub(super) fn join_verbatim_runs(spans: &mut Vec<vize_canon::ContentMapperSpan>) {
    spans.dedup_by(|next, run| {
        let continues = run.0[4] == VERBATIM
            && next.0[4] == VERBATIM
            && next.0[0] == run.0[0] + run.0[1]
            && next.0[2] == run.0[2] + run.0[3];
        if continues {
            run.0[1] += next.0[1];
            run.0[3] += next.0[3];
        }
        continues
    });
}

/// Compact `transform`'s trivia in place; leave it unchanged when unsure.
pub(super) fn compact(transform: &mut ContentMapperTransform) {
    let Some((text, kept)) = compacted(&transform.text, &transform.mappings) else {
        return;
    };
    let map = |position: usize| remap(&kept, position, text.len());
    for span in &mut transform.mappings {
        span.0[0] = map(span.0[0]);
    }
    if let Some(table) = &mut transform.diagnostic_directives {
        for directive in &mut table.directives {
            directive.0[2] = map(directive.0[2]);
            directive.0[3] = map(directive.0[3]);
        }
    }
    for link in &mut transform.semantic_links {
        let source_end = map(link.source_start + link.source_length);
        link.source_start = map(link.source_start);
        link.source_length = source_end.saturating_sub(link.source_start);
        let target_end = map(link.target_start + link.target_length);
        link.target_start = map(link.target_start);
        link.target_length = target_end.saturating_sub(link.target_start);
    }
    transform.text = text;
}

/// The compacted text and where each kept run of the original went, or
/// `None` when nothing changes or the text is not certainly understood.
fn compacted(
    text: &str,
    spans: &[vize_canon::ContentMapperSpan],
) -> Option<(vize_l0::String, Vec<Kept>)> {
    let mut covered = covered_bytes(spans);
    let mut out = vize_l0::String::with_capacity(text.len());
    let mut kept = Vec::new();
    let mut pieces = Vec::new();
    let mut copied_to = 0;
    trivia(text, |start, end, comments| {
        let gap = start..end;
        // A lone space or line break is already as short as it gets.
        if comments.is_empty() && gap.len() == 1 || !covered.rewritable(gap.clone()) {
            return;
        }
        // Rewrites only drop or shorten, so the same length is the same text.
        if rewrite(text, gap.clone(), comments, &mut pieces) == gap.len() {
            return;
        }
        push_kept(&mut out, &mut kept, text, copied_to..gap.start);
        for piece in pieces.drain(..) {
            match piece {
                Piece::Separator(separator) => out.push_str(separator),
                Piece::Kept(range) => push_kept(&mut out, &mut kept, text, range),
            }
        }
        copied_to = gap.end;
    })?;
    if kept.is_empty() && copied_to == 0 {
        return None;
    }
    push_kept(&mut out, &mut kept, text, copied_to..text.len());
    Some((out, kept))
}

/// The spans' generated ranges, merged, and each zero-length span's position,
/// which no rewrite may move past; queried in text order.
struct Covered {
    ranges: Vec<(usize, usize)>,
    points: Vec<usize>,
    next_range: usize,
    next_point: usize,
}

fn covered_bytes(spans: &[vize_canon::ContentMapperSpan]) -> Covered {
    let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(spans.len());
    let mut points = Vec::new();
    for span in spans {
        let (start, length) = (span.0[0], span.0[1]);
        if length == 0 {
            points.push(start);
        } else {
            ranges.push((start, start + length));
        }
    }
    ranges.sort_unstable();
    ranges.dedup_by(|next, run| {
        let overlaps = next.0 <= run.1;
        if overlaps {
            run.1 = run.1.max(next.1);
        }
        overlaps
    });
    points.sort_unstable();
    Covered {
        ranges,
        points,
        next_range: 0,
        next_point: 0,
    }
}

impl Covered {
    /// No span byte inside `gap`, and no zero-length span strictly inside.
    /// Gaps come in text order.
    fn rewritable(&mut self, gap: std::ops::Range<usize>) -> bool {
        while self
            .ranges
            .get(self.next_range)
            .is_some_and(|range| range.1 <= gap.start)
        {
            self.next_range += 1;
        }
        while self
            .points
            .get(self.next_point)
            .is_some_and(|point| *point <= gap.start)
        {
            self.next_point += 1;
        }
        self.ranges
            .get(self.next_range)
            .is_none_or(|range| range.0 >= gap.end)
            && self
                .points
                .get(self.next_point)
                .is_none_or(|point| *point >= gap.end)
    }
}

/// One piece of a token gap's replacement.
enum Piece {
    Separator(&'static str),
    Kept(std::ops::Range<usize>),
}

/// The gap's kept comments and separators, into `pieces`, and their length.
/// Nothing needs separating from the start of the file.
fn rewrite(
    text: &str,
    gap: std::ops::Range<usize>,
    comments: &[Comment],
    pieces: &mut Vec<Piece>,
) -> usize {
    pieces.clear();
    let mut len = 0;
    let mut separator = Separator::None;
    let mut at = gap.start;
    let mut preceded = gap.start > 0;
    for comment in comments {
        let (start, end) = (comment.start, comment.end);
        separator = separator.widen(text, at..start);
        at = end;
        let Some(keep) = kept_comment(text, comment.line, start..end) else {
            continue;
        };
        if let Some(separator) = separator.take().filter(|_| preceded) {
            pieces.push(Piece::Separator(separator));
            len += separator.len();
        }
        len += keep.len();
        pieces.push(Piece::Kept(keep));
        preceded = true;
    }
    if let Some(separator) = separator
        .widen(text, at..gap.end)
        .take()
        .filter(|_| preceded)
    {
        pieces.push(Piece::Separator(separator));
        len += separator.len();
    }
    len
}

/// The widest whitespace seen since the last kept piece.
#[derive(Clone, Copy)]
enum Separator {
    None,
    Space,
    Line,
}

impl Separator {
    fn widen(self, text: &str, range: std::ops::Range<usize>) -> Self {
        let bytes = text.as_bytes().get(range).unwrap_or_default();
        if bytes.iter().any(|byte| matches!(byte, b'\n' | b'\r')) {
            Self::Line
        } else if !bytes.is_empty() && !matches!(self, Self::Line) {
            Self::Space
        } else {
            self
        }
    }

    fn take(&mut self) -> Option<&'static str> {
        let separator = match self {
            Self::None => None,
            Self::Space => Some(" "),
            Self::Line => Some("\n"),
        };
        *self = Self::None;
        separator
    }
}

/// The part of a comment that stays: block comments and directive-like line
/// comments whole, `@ts-ignore` up to its directive, nothing of the rest.
fn kept_comment(
    text: &str,
    line: bool,
    range: std::ops::Range<usize>,
) -> Option<std::ops::Range<usize>> {
    if !line {
        return Some(range);
    }
    let body = text.get(range.start + 2..range.end)?;
    if body.starts_with(['/', '#']) {
        return Some(range);
    }
    let directive = body.trim_start();
    if !directive.starts_with('@') || directive.starts_with("@vize-map") {
        return None;
    }
    const IGNORE: &str = "@ts-ignore";
    if let Some(reason) = directive.strip_prefix(IGNORE)
        && reason.starts_with([' ', '\t'])
    {
        return Some(range.start..range.end - reason.len());
    }
    Some(range)
}

fn push_kept(
    out: &mut vize_l0::String,
    kept: &mut Vec<Kept>,
    text: &str,
    range: std::ops::Range<usize>,
) {
    if range.is_empty() {
        return;
    }
    kept.push(Kept {
        start: range.start,
        end: range.end,
        new_start: out.len(),
    });
    out.push_str(text.get(range).unwrap_or_default());
}

/// Where `position` lands: its own place in a kept run, or the next kept
/// byte's place when its own byte was dropped.
fn remap(kept: &[Kept], position: usize, new_len: usize) -> usize {
    let index = kept.partition_point(|run| run.end <= position);
    match kept.get(index) {
        Some(run) if run.start <= position => run.new_start + (position - run.start),
        Some(run) => run.new_start,
        None => new_len,
    }
}
