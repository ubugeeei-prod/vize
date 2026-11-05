//! The SFC itself, as the target of navigation into its component.

use super::span_features::CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET;
use super::{ContentMapperSpan, ContentMapperSpanKind};
use crate::virtual_ts::{ProjectionMeta, ProjectionSpanKind, VizeMapping};

/// Insert each component-export row as a navigation-only atom, keeping
/// `spans` ordered by generated start.
pub(super) fn insert_spans(
    spans: &mut Vec<ContentMapperSpan>,
    mappings: &[VizeMapping],
    metadata: &[ProjectionMeta],
) {
    for (row, _) in mappings
        .iter()
        .zip(metadata)
        .filter(|(_, meta)| meta.kind == ProjectionSpanKind::ComponentExport)
    {
        let at = spans.partition_point(|span| span.0[0] < row.gen_range.start);
        spans.insert(
            at,
            ContentMapperSpan([
                row.gen_range.start,
                row.gen_range.len(),
                row.src_range.start,
                row.src_range.len(),
                ContentMapperSpanKind::Atom as usize,
                CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET,
            ]),
        );
    }
}
