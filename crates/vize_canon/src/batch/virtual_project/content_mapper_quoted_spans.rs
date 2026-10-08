//! Retain exact quoted definition endpoints in the protocol consumer only.

use super::span_features::content_mapper_span_features;
use super::{ContentMapperSpanKind, SpanCandidate, VizeMapping, candidate};

pub(super) fn candidates(
    source: &str,
    generated: &str,
    mappings: &[VizeMapping],
) -> Vec<SpanCandidate> {
    let mut result = Vec::new();
    let mut mappings = mappings.iter().peekable();
    while let Some(mapping) = mappings.next() {
        if let Some(next) = mappings.peek()
            && let Some(quoted) = quoted_pair(source, generated, mapping, next)
        {
            result.push(quoted);
            let _ = mappings.next();
            continue;
        }
        if mapping.sub_spans.is_empty() {
            result.extend(candidate(
                source,
                generated,
                mapping.gen_range.clone(),
                mapping.src_range.clone(),
            ));
        } else {
            result.extend(mapping.sub_spans.iter().filter_map(|span| {
                candidate(
                    source,
                    generated,
                    span.gen_range.clone(),
                    span.src_range.clone(),
                )
            }));
        }
    }
    result
}

fn quoted_pair(
    source: &str,
    generated: &str,
    whole: &VizeMapping,
    interior: &VizeMapping,
) -> Option<SpanCandidate> {
    if !whole.sub_spans.is_empty()
        || !interior.sub_spans.is_empty()
        || whole.src_range.is_empty()
        || whole.src_range != interior.src_range
        || whole.gen_range.start.checked_add(1)? != interior.gen_range.start
        || whole.gen_range.end.checked_sub(1)? != interior.gen_range.end
    {
        return None;
    }
    let text = generated.get(whole.gen_range.clone())?;
    let quote = *text.as_bytes().first()?;
    if !matches!(quote, b'\'' | b'"') || text.as_bytes().last() != Some(&quote) {
        return None;
    }
    let authored = source.get(whole.src_range.clone())?;
    if authored.contains('\\') || text.get(1..text.len().checked_sub(1)?)? != authored {
        return None;
    }
    let original = whole.src_range.start.checked_sub(1)?..whole.src_range.end.checked_add(1)?;
    if source.get(original.clone())? != text {
        return None;
    }
    // VizeMapping has no feature field. Preserve the computed protocol
    // metadata of both ordinary candidates and the full replacement.
    let whole_candidate = candidate(
        source,
        generated,
        whole.gen_range.clone(),
        whole.src_range.clone(),
    )?;
    let interior_candidate = candidate(
        source,
        generated,
        interior.gen_range.clone(),
        interior.src_range.clone(),
    )?;
    if whole_candidate.generated != interior.gen_range
        || interior_candidate.generated != interior.gen_range
        || whole_candidate.kind != ContentMapperSpanKind::Verbatim
        || interior_candidate.kind != ContentMapperSpanKind::Verbatim
    {
        return None;
    }
    let features = content_mapper_span_features(
        generated,
        whole_candidate.generated.start,
        whole_candidate.kind,
    );
    if features
        != content_mapper_span_features(
            generated,
            interior_candidate.generated.start,
            interior_candidate.kind,
        )
        || features
            != content_mapper_span_features(
                generated,
                whole.gen_range.start,
                ContentMapperSpanKind::Verbatim,
            )
    {
        return None;
    }
    Some(SpanCandidate {
        generated: whole.gen_range.clone(),
        original,
        kind: ContentMapperSpanKind::Verbatim,
    })
}

#[cfg(test)]
#[path = "content_mapper_quoted_spans_tests.rs"]
mod tests;
