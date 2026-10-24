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
    type_only: bool,
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
            type_only: candidate.is_type,
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

/// What a captured `export * from './X.vue'` may be replaced by.
#[derive(Debug, Clone)]
pub(in crate::batch) enum VueStarTarget<'a> {
    /// The component has no named module exports: re-export nothing.
    Empty,
    /// The component's authored named exports, which exclude the declarations
    /// the virtual module generates for it (`Props`, `Emits`, `Slots`, ...).
    Names(&'a [vize_croquis::script_parser::AuthoredModuleExport]),
}

/// Replay only captured original star bytes onto their original literal baseline.
/// This never parses, reconstructs candidates, or reruns a specifier policy.
#[cfg(test)]
pub(in crate::batch) fn apply_captured_empty_vue_stars<E>(
    base_code: &str,
    base_map: &ImportSourceMap,
    stars: &[EmptyVueStar],
    empty_target: E,
) -> RewriteResult
where
    E: Fn(&str, &str, crate::PackageResolutionMode) -> bool,
{
    apply_captured_vue_stars(base_code, base_map, stars, |authored, projected, mode| {
        empty_target(authored, projected, mode).then_some(VueStarTarget::Empty)
    })
}

/// Replace each certified star with an explicit list of the component's
/// authored exports. Two stars of the same module that would re-export the
/// same name keep their original `*`, so an authored collision is still
/// reported exactly as TypeScript reports it for star exports.
pub(in crate::batch) fn apply_captured_vue_stars<'t, E>(
    base_code: &str,
    base_map: &ImportSourceMap,
    stars: &[EmptyVueStar],
    target: E,
) -> RewriteResult
where
    E: Fn(&str, &str, crate::PackageResolutionMode) -> Option<VueStarTarget<'t>>,
{
    let mut candidates = Vec::new();
    for star in stars {
        let Some(projected) = star.projected_literal(base_code, base_map) else {
            continue;
        };
        let Some(target) = target(star.specifier(), projected, star.mode()) else {
            continue;
        };
        let start = base_map.get_virtual_offset(star.source_start);
        let end = base_map.get_virtual_offset(star.source_end);
        if end == start + 1
            && base_code.get(start as usize..end as usize) == Some("*")
            && !candidates
                .iter()
                .any(|(offset, _, _, _): &(u32, u32, VueStarTarget<'t>, bool)| *offset == start)
        {
            candidates.push((start, star.source_start, target, star.type_only));
        }
    }
    let mut name_counts: vize_carton::FxHashMap<&str, usize> = vize_carton::FxHashMap::default();
    for (_, _, target, _) in &candidates {
        if let VueStarTarget::Names(names) = target {
            for export in names.iter() {
                *name_counts.entry(export.name.as_str()).or_default() += 1;
            }
        }
    }
    let mut edits: Vec<(u32, u32, String)> = candidates
        .iter()
        .filter_map(|(start, original, target, type_only)| {
            let text = match target {
                VueStarTarget::Empty => String::from("{}"),
                VueStarTarget::Names(names) => {
                    // A local export shadows a star silently but conflicts
                    // with an explicit list, so any mention keeps the star.
                    if names.iter().any(|export| {
                        name_counts.get(export.name.as_str()) > Some(&1)
                            || mentions_identifier(base_code, export.name.as_str())
                    }) {
                        return None;
                    }
                    named_star_replacement(names, *type_only)
                }
            };
            Some((*start, *original, text))
        })
        .collect();
    edits.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));
    let mut code = base_code.to_compact_string();
    for (start, _, text) in &edits {
        code.replace_range(*start as usize..*start as usize + 1, text);
    }
    let replacements: Vec<_> = edits
        .into_iter()
        .map(|(_, original, text)| (original, text.len() as u32))
        .collect();
    RewriteResult {
        code,
        source_map: base_map.with_added_star_replacements(&replacements),
    }
}

fn mentions_identifier(code: &str, name: &str) -> bool {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_' || c == '$';
    code.match_indices(name).any(|(start, _)| {
        let before = code.get(..start).and_then(|text| text.chars().next_back());
        let after = code
            .get(start + name.len()..)
            .and_then(|text| text.chars().next());
        !before.is_some_and(is_ident) && !after.is_some_and(is_ident)
    })
}

/// `{ value, type Props }` for an ordinary star; an `export type *` star makes
/// every name type-only already, so its list carries no modifiers.
fn named_star_replacement(
    names: &[vize_croquis::script_parser::AuthoredModuleExport],
    type_star: bool,
) -> String {
    if names.is_empty() {
        return String::from("{}");
    }
    let mut text = String::from("{ ");
    for (index, export) in names.iter().enumerate() {
        if index > 0 {
            text.push_str(", ");
        }
        if export.type_only && !type_star {
            text.push_str("type ");
        }
        text.push_str(export.name.as_str());
    }
    text.push_str(" }");
    text
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
