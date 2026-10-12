//! Prove the original star byte from a same-parse top-level AST and trivia.

use oxc_ast::ast::Comment;
use oxc_span::Span;
use vize_carton::{String, ToCompactString};

use super::collect::BareStarExport;
use super::{ImportSourceMap, RewriteResult};

/// Same-owner original parse evidence; fields cannot be constructed by callers.
#[derive(Debug, Clone)]
pub(in crate::batch) struct EmptyVueStar {
    source_start: u32,
    source_end: u32,
    literal: Span,
    quote: u8,
    specifier: String,
    mode: crate::PackageResolutionMode,
}

impl EmptyVueStar {
    pub(super) fn capture(
        source: &str,
        comments: &[Comment],
        candidate: BareStarExport,
    ) -> Option<Self> {
        let star = star_offset(source, comments, &candidate)?;
        let quote_start = candidate.literal.start.checked_sub(1)?;
        let quote = *source.as_bytes().get(quote_start as usize)?;
        if quote_start != candidate.header.end
            || candidate.literal.start > candidate.literal.end
            || !matches!(quote, b'\'' | b'"')
            || source.as_bytes().get(candidate.literal.end as usize) != Some(&quote)
        {
            return None;
        }
        source.get(candidate.literal.start as usize..candidate.literal.end as usize)?;
        Some(Self {
            source_start: star,
            source_end: star + 1,
            literal: candidate.literal,
            quote,
            specifier: candidate.specifier,
            mode: candidate.mode,
        })
    }

    pub(in crate::batch) fn specifier(&self) -> &str {
        self.specifier.as_str()
    }

    pub(in crate::batch) fn mode(&self) -> crate::PackageResolutionMode {
        self.mode
    }

    fn projected_literal<'a>(&self, code: &'a str, map: &ImportSourceMap) -> Option<&'a str> {
        // The legacy content-start mapping already includes this literal's
        // own delta. Its opening quote precedes that adjustment, so anchor there.
        let quote_start = map.get_virtual_offset(self.literal.start.checked_sub(1)?);
        let start = quote_start.checked_add(1)?;
        let end = map.get_virtual_offset(self.literal.end);
        if start > end
            || code.as_bytes().get(quote_start as usize) != Some(&self.quote)
            || code.as_bytes().get(end as usize) != Some(&self.quote)
        {
            return None;
        }
        let literal = code.get(start as usize..end as usize)?;
        // Escaped or injected delimiter text has no current decoded receipt.
        // Preserve its baseline instead of inferring a target without lexing.
        (!literal
            .bytes()
            .any(|byte| byte == self.quote || matches!(byte, b'\\' | b'\r' | b'\n')))
        .then_some(literal)
    }
}

/// Replay only captured original star bytes onto their original literal baseline.
/// This never parses, reconstructs candidates, or reruns a specifier policy.
pub(in crate::batch) fn apply_captured_empty_vue_stars<E>(
    base_code: &str,
    base_map: &ImportSourceMap,
    stars: &[EmptyVueStar],
    empty_target: E,
) -> RewriteResult
where
    E: Fn(&str, &str, crate::PackageResolutionMode) -> bool,
{
    let mut edits = Vec::new();
    for star in stars {
        let Some(projected) = star.projected_literal(base_code, base_map) else {
            continue;
        };
        if !empty_target(star.specifier(), projected, star.mode()) {
            continue;
        }
        let start = base_map.get_virtual_offset(star.source_start);
        let end = base_map.get_virtual_offset(star.source_end);
        if end == start + 1
            && base_code.get(start as usize..end as usize) == Some("*")
            && !edits.iter().any(|(offset, _)| *offset == start)
        {
            edits.push((start, star.source_start));
        }
    }
    edits.sort_by_key(|(start, _)| std::cmp::Reverse(*start));
    let mut code = base_code.to_compact_string();
    for (start, _) in &edits {
        code.replace_range(*start as usize..*start as usize + 1, "{}");
    }
    let original_starts: Vec<_> = edits.into_iter().map(|(_, original)| original).collect();
    RewriteResult {
        code,
        source_map: base_map.with_added_star_replacements(&original_starts),
    }
}

pub(super) fn star_offset(
    source: &str,
    comments: &[Comment],
    candidate: &BareStarExport,
) -> Option<u32> {
    let start = candidate.header.start as usize;
    let end = candidate.header.end as usize;
    source.get(start..end)?;
    let mut cursor = HeaderCursor {
        source,
        comments,
        comment: comments.partition_point(|comment| comment.span.end <= candidate.header.start),
        position: start,
        end,
    };
    cursor.take("export")?;
    let separated = cursor.trivia()?;
    if candidate.is_type {
        if !separated {
            return None;
        }
        cursor.take("type")?;
        cursor.trivia()?;
    }
    let star = cursor.position;
    cursor.take("*")?;
    cursor.trivia()?;
    cursor.take("from")?;
    cursor.trivia()?;
    (cursor.position == end).then_some(star as u32)
}

struct HeaderCursor<'a> {
    source: &'a str,
    comments: &'a [Comment],
    comment: usize,
    position: usize,
    end: usize,
}

impl HeaderCursor<'_> {
    fn take(&mut self, text: &str) -> Option<()> {
        let end = self.position.checked_add(text.len())?;
        if end > self.end || self.source.get(self.position..end)? != text {
            return None;
        }
        self.position = end;
        Some(())
    }

    fn trivia(&mut self) -> Option<bool> {
        let initial = self.position;
        while self.position < self.end {
            if let Some(comment) = self.comments.get(self.comment) {
                let start = comment.span.start as usize;
                let end = comment.span.end as usize;
                if start < self.position {
                    return None;
                }
                if start == self.position {
                    if end <= start || end > self.end {
                        return None;
                    }
                    self.source.get(start..end)?;
                    self.position = end;
                    self.comment += 1;
                    continue;
                }
            }
            let character = self.source.get(self.position..self.end)?.chars().next()?;
            if !matches!(
                character,
                '\t' | '\u{b}' | '\u{c}' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
                    ..='\u{200a}'
                        | '\u{202f}'
                        | '\u{205f}'
                        | '\u{3000}'
                        | '\u{feff}'
                        | '\n'
                        | '\r'
                        | '\u{2028}'
                        | '\u{2029}'
            ) {
                break;
            }
            self.position += character.len_utf8();
        }
        Some(self.position != initial)
    }
}
