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
    if original.len() != projected.len() || original == projected {
        return None;
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
}
