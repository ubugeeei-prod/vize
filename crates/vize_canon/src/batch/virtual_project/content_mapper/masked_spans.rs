//! Preserve exact fragments around byte-padded script keyword masks.

use super::{ContentMapperSpanKind, SpanCandidate};

pub(super) fn expand(candidates: &mut Vec<SpanCandidate>, source: &str, generated: &str) {
    let original_count = candidates.len();
    for index in 0..original_count {
        let Some(candidate) = candidates.get(index) else {
            break;
        };
        let Some(fragments) = split(candidate, source, generated) else {
            continue;
        };
        let mut fragments = fragments.into_iter();
        if let Some(first) = fragments.next()
            && let Some(candidate) = candidates.get_mut(index)
        {
            *candidate = first;
            candidates.extend(fragments);
        }
    }
}

fn split(candidate: &SpanCandidate, source: &str, generated: &str) -> Option<Vec<SpanCandidate>> {
    if candidate.kind != ContentMapperSpanKind::Atom {
        return None;
    }
    let original = source.get(candidate.original.clone())?;
    let projected = generated.get(candidate.generated.clone())?;
    if original == projected {
        return None;
    }
    if original.len() != projected.len() {
        return split_comment_elision(candidate, original, projected);
    }
    // The generator erases AST-owned module/export spans with ASCII spaces.
    // Other spelling or length rewrites retain their existing atom/alias rules.
    if original
        .bytes()
        .zip(projected.bytes())
        .any(|(before, after)| before != after && after != b' ')
    {
        return None;
    }
    let mut fragments = Vec::new();
    let mut run = None;
    let mut emit = |start: usize, end: usize, exact: bool| {
        fragments.push(SpanCandidate {
            generated: candidate.generated.start + start..candidate.generated.start + end,
            original: candidate.original.start + start..candidate.original.start + end,
            kind: if exact {
                ContentMapperSpanKind::Verbatim
            } else {
                ContentMapperSpanKind::Atom
            },
        });
    };
    for (offset, character) in original.char_indices() {
        let end = offset + character.len_utf8();
        let exact = original.get(offset..end) == projected.get(offset..end);
        if let Some((start, previous)) = run {
            if exact != previous {
                emit(start, offset, previous);
                run = Some((offset, exact));
            }
        } else {
            run = Some((offset, exact));
        }
    }
    if let Some((start, exact)) = run {
        emit(start, original.len(), exact);
    }
    (fragments.len() > 1).then_some(fragments)
}

/// Template expressions retain their authored comments, while the generated
/// TypeScript printer replaces those comments with a variable amount of
/// whitespace. The enclosing atom therefore has no single affine mapping.
/// Preserve only byte-identical pieces around comments and whitespace rewrites;
/// normalization can then split them safely around narrower navigation spans.
fn split_comment_elision(
    candidate: &SpanCandidate,
    original: &str,
    projected: &str,
) -> Option<Vec<SpanCandidate>> {
    if !original.contains("/*") {
        return None;
    }

    let original_bytes = original.as_bytes();
    let projected_bytes = projected.as_bytes();
    let mut original_at = 0;
    let mut projected_at = 0;
    let mut exact_run = None;
    let mut fragments = Vec::new();
    let mut skipped_comment = false;

    while original_at < original_bytes.len() && projected_at < projected_bytes.len() {
        if original_bytes.get(original_at) == projected_bytes.get(projected_at) {
            exact_run.get_or_insert((original_at, projected_at));
            original_at += 1;
            projected_at += 1;
            continue;
        }

        finish_exact_run(
            candidate,
            &mut fragments,
            &mut exact_run,
            original_at,
            projected_at,
        );
        if let Some(after_comment) = block_comment_end(original, original_at) {
            original_at = after_comment;
            skipped_comment = true;
            continue;
        }
        if skipped_comment {
            let next_original = skip_ascii_whitespace(original_bytes, original_at);
            let next_projected = skip_ascii_whitespace(projected_bytes, projected_at);
            if next_original != original_at || next_projected != projected_at {
                original_at = next_original;
                projected_at = next_projected;
                continue;
            }
        }
        return None;
    }
    finish_exact_run(
        candidate,
        &mut fragments,
        &mut exact_run,
        original_at,
        projected_at,
    );

    while original_at < original_bytes.len() {
        if let Some(after_comment) = block_comment_end(original, original_at) {
            original_at = after_comment;
            skipped_comment = true;
        } else if original_bytes
            .get(original_at)
            .is_some_and(u8::is_ascii_whitespace)
        {
            original_at += 1;
        } else {
            return None;
        }
    }
    projected_at = skip_ascii_whitespace(projected_bytes, projected_at);
    (skipped_comment && projected_at == projected_bytes.len() && !fragments.is_empty())
        .then_some(fragments)
}

fn finish_exact_run(
    candidate: &SpanCandidate,
    fragments: &mut Vec<SpanCandidate>,
    run: &mut Option<(usize, usize)>,
    original_end: usize,
    projected_end: usize,
) {
    let Some((original_start, projected_start)) = run.take() else {
        return;
    };
    if original_end - original_start != projected_end - projected_start {
        return;
    }
    fragments.push(SpanCandidate {
        generated: candidate.generated.start + projected_start
            ..candidate.generated.start + projected_end,
        original: candidate.original.start + original_start
            ..candidate.original.start + original_end,
        kind: ContentMapperSpanKind::Verbatim,
    });
}

fn block_comment_end(text: &str, start: usize) -> Option<usize> {
    let comment = text.get(start..)?.strip_prefix("/*")?;
    comment.find("*/").map(|length| start + 2 + length + 2)
}

fn skip_ascii_whitespace(bytes: &[u8], mut at: usize) -> usize {
    while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        at += 1;
    }
    at
}

#[cfg(test)]
mod tests {
    use super::{ContentMapperSpanKind, SpanCandidate, expand};

    #[test]
    fn exact_unicode_ranges_survive_only_space_padded_keyword_masks() {
        let source = "0123456789/* 😀 */ export const café = 1;";
        let generated = "01234567890123456789/* 😀 */        const café = 1;";
        let mut candidates = vec![SpanCandidate {
            generated: 20..54,
            original: 10..44,
            kind: ContentMapperSpanKind::Atom,
        }];
        expand(&mut candidates, source, generated);
        assert_eq!(
            candidates
                .iter()
                .map(|span| (span.generated.clone(), span.original.clone(), span.kind))
                .collect::<Vec<_>>(),
            vec![
                (20..31, 10..21, ContentMapperSpanKind::Verbatim),
                (31..37, 21..27, ContentMapperSpanKind::Atom),
                (37..54, 27..44, ContentMapperSpanKind::Verbatim),
            ]
        );
    }

    #[test]
    fn spelling_length_and_alias_rewrites_keep_their_original_contract() {
        for (original, generated, kind) in [
            ("field", "other", ContentMapperSpanKind::Atom),
            ("café", "cafe", ContentMapperSpanKind::Atom),
            ("field", "     ", ContentMapperSpanKind::Alias),
            ("😀", "    ", ContentMapperSpanKind::Atom),
        ] {
            let mut candidates = vec![SpanCandidate {
                generated: 0..generated.len(),
                original: 0..original.len(),
                kind,
            }];
            expand(&mut candidates, original, generated);
            assert_eq!(candidates.len(), 1);
            assert_eq!(candidates[0].generated, 0..generated.len());
            assert_eq!(candidates[0].original, 0..original.len());
            assert_eq!(candidates[0].kind, kind);
        }
    }

    #[test]
    fn exact_fragments_survive_comment_elision_with_different_lengths() {
        let original = "done && item.kind === 'a' ? 30 /* top */ - 20 /* bottom */ : undefined";
        let generated = "done && item.kind === 'a' ? 30   - 20   : undefined";
        let mut candidates = vec![SpanCandidate {
            generated: 0..generated.len(),
            original: 0..original.len(),
            kind: ContentMapperSpanKind::Atom,
        }];

        expand(&mut candidates, original, generated);

        let member = original.find("item.kind").expect("authored member");
        let projected_member = generated.find("item.kind").expect("generated member");
        assert!(candidates.iter().any(|span| {
            span.kind == ContentMapperSpanKind::Verbatim
                && span.original.start <= member
                && member + "item.kind".len() <= span.original.end
                && span.generated.start <= projected_member
                && projected_member + "item.kind".len() <= span.generated.end
        }));
        assert!(candidates.iter().all(|span| {
            original.get(span.original.clone()) == generated.get(span.generated.clone())
        }));
    }

    #[test]
    fn unexplained_rewrites_with_comments_keep_the_atom_contract() {
        let original = "/* note */ field";
        let generated = "other";
        let mut candidates = vec![SpanCandidate {
            generated: 0..generated.len(),
            original: 0..original.len(),
            kind: ContentMapperSpanKind::Atom,
        }];

        expand(&mut candidates, original, generated);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, ContentMapperSpanKind::Atom);
        assert_eq!(candidates[0].generated, 0..generated.len());
        assert_eq!(candidates[0].original, 0..original.len());
    }
}
