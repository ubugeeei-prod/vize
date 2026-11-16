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
//! oxc's tokens say where trivia is. A projection it does not parse cleanly,
//! or whose token gaps hold anything but whitespace and comments, is left as
//! it is.

use oxc_allocator::Allocator;
use oxc_ast::ast::CommentKind;
use oxc_parser::Parser;
use oxc_parser::config::TokensParserConfig;
use oxc_span::SourceType;
use vize_canon::ContentMapperTransform;

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
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::tsx())
        .with_config(TokensParserConfig)
        .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let covered = covered_bytes(text.len(), spans);
    let comments = &parsed.program.comments;
    let mut next_comment = 0;
    let mut out = vize_l0::String::with_capacity(text.len());
    let mut kept = Vec::new();
    let mut copied_to = 0;
    let mut previous_end = 0;
    let gaps = parsed
        .tokens
        .iter()
        .map(|token| (token.start() as usize, token.end() as usize))
        .chain(std::iter::once((text.len(), text.len())));
    for (token_start, token_end) in gaps {
        if token_start < previous_end || token_end < token_start {
            return None;
        }
        let gap = previous_end..token_start;
        previous_end = token_end;
        let first = next_comment;
        while comments
            .get(next_comment)
            .is_some_and(|comment| (comment.span.start as usize) < gap.end)
        {
            next_comment += 1;
        }
        let gap_comments = comments.get(first..next_comment).unwrap_or_default();
        if !is_trivia(text, gap.clone(), gap_comments) {
            return None;
        }
        if gap.is_empty() || !rewritable(&covered, gap.clone()) {
            continue;
        }
        let rewritten = rewrite(text, gap.clone(), gap_comments);
        if rewritten.len() == gap.len() && rewritten.text == text.get(gap.clone())? {
            continue;
        }
        push_kept(&mut out, &mut kept, text, copied_to..gap.start);
        rewritten.emit(text, &mut out, &mut kept);
        copied_to = gap.end;
    }
    if kept.is_empty() && copied_to == 0 {
        return None;
    }
    push_kept(&mut out, &mut kept, text, copied_to..text.len());
    Some((out, kept))
}

/// One byte flag per text byte inside a span, plus each zero-length span's
/// position, which no rewrite may move past.
struct Covered {
    bytes: Vec<bool>,
    points: Vec<usize>,
}

fn covered_bytes(len: usize, spans: &[vize_canon::ContentMapperSpan]) -> Covered {
    let mut bytes = vec![false; len];
    let mut points = Vec::new();
    for span in spans {
        let (start, length) = (span.0[0], span.0[1]);
        if length == 0 {
            points.push(start);
        }
        for flag in bytes.iter_mut().skip(start).take(length) {
            *flag = true;
        }
    }
    points.sort_unstable();
    Covered { bytes, points }
}

/// No span byte inside, and no zero-length span strictly inside.
fn rewritable(covered: &Covered, gap: std::ops::Range<usize>) -> bool {
    let free = covered
        .bytes
        .get(gap.clone())
        .is_some_and(|bytes| !bytes.contains(&true));
    let first_point = covered.points.partition_point(|point| *point <= gap.start);
    free && covered
        .points
        .get(first_point)
        .is_none_or(|point| *point >= gap.end)
}

/// Every byte of a token gap is ASCII whitespace or inside one of its
/// comments, and every comment lies inside the gap.
fn is_trivia(text: &str, gap: std::ops::Range<usize>, comments: &[oxc_ast::Comment]) -> bool {
    let mut at = gap.start;
    for comment in comments {
        let (start, end) = (comment.span.start as usize, comment.span.end as usize);
        if start < at || end > gap.end || !is_whitespace(text, at..start) {
            return false;
        }
        at = end;
    }
    is_whitespace(text, at..gap.end)
}

fn is_whitespace(text: &str, range: std::ops::Range<usize>) -> bool {
    text.as_bytes().get(range).is_some_and(|bytes| {
        bytes
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
    })
}

/// A token gap's replacement: kept comment runs and the separators between.
struct Rewrite {
    pieces: Vec<Piece>,
    text: vize_l0::String,
}

enum Piece {
    Separator(&'static str),
    Kept(std::ops::Range<usize>),
}

impl Rewrite {
    fn len(&self) -> usize {
        self.text.len()
    }

    fn emit(self, text: &str, out: &mut vize_l0::String, kept: &mut Vec<Kept>) {
        for piece in self.pieces {
            match piece {
                Piece::Separator(separator) => out.push_str(separator),
                Piece::Kept(range) => push_kept(out, kept, text, range),
            }
        }
    }
}

/// The gap's kept comments and separators. Nothing needs separating from the
/// start of the file.
fn rewrite(text: &str, gap: std::ops::Range<usize>, comments: &[oxc_ast::Comment]) -> Rewrite {
    let mut pieces = Vec::new();
    let mut rendered = vize_l0::String::default();
    let mut separator = Separator::None;
    let mut at = gap.start;
    let mut preceded = gap.start > 0;
    for comment in comments {
        let (start, end) = (comment.span.start as usize, comment.span.end as usize);
        separator = separator.widen(text, at..start);
        at = end;
        let Some(keep) = kept_comment(text, comment.kind, start..end) else {
            continue;
        };
        if let Some(separator) = separator.take().filter(|_| preceded) {
            pieces.push(Piece::Separator(separator));
            rendered.push_str(separator);
        }
        rendered.push_str(text.get(keep.clone()).unwrap_or_default());
        pieces.push(Piece::Kept(keep));
        preceded = true;
    }
    if let Some(separator) = separator
        .widen(text, at..gap.end)
        .take()
        .filter(|_| preceded)
    {
        pieces.push(Piece::Separator(separator));
        rendered.push_str(separator);
    }
    Rewrite {
        pieces,
        text: rendered,
    }
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
    kind: CommentKind,
    range: std::ops::Range<usize>,
) -> Option<std::ops::Range<usize>> {
    if kind != CommentKind::Line {
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
