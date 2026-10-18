use tower_lsp::lsp_types::Location;

/// Combine Corsa reference hits with the authored ones for this SFC.
///
/// Corsa only answers for the single virtual document the request opened, so a
/// script-side query reports the script occurrences but never the template
/// ones the same binding drives (nor the reverse). Folding the authored hits
/// back in keeps every block of the SFC represented, while Corsa keeps the
/// cross-file hits the authored sweep cannot see.
pub(crate) fn merge_authored_locations(
    corsa: Option<Vec<Location>>,
    authored: Option<Vec<Location>>,
) -> Option<Vec<Location>> {
    let mut locations = match (corsa, authored) {
        (Some(mut corsa), Some(authored)) => {
            corsa.extend(authored);
            corsa
        }
        (Some(locations), None) | (None, Some(locations)) => locations,
        (None, None) => return None,
    };

    if locations.is_empty() {
        return None;
    }

    locations.sort_by(|a, b| {
        a.range
            .start
            .line
            .cmp(&b.range.start.line)
            .then(a.range.start.character.cmp(&b.range.start.character))
            .then_with(|| a.uri.as_str().cmp(b.uri.as_str()))
            // Keep the existing position/URI order, but group identical full
            // ranges even when another hit starts at the same position.
            .then(a.range.end.line.cmp(&b.range.end.line))
            .then(a.range.end.character.cmp(&b.range.end.character))
    });
    locations.dedup_by(|a, b| a.uri == b.uri && a.range == b.range);

    Some(locations)
}

/// Combine the canonical project answer with the block-local Corsa hits and
/// the authored ones for this SFC.
///
/// The canonical virtual document crosses SFC boundaries, but it only answers
/// for the regions it could map back, so the template and style occurrences
/// still ride in on the block-local and authored sweeps. An empty canonical
/// answer stays authoritative when nothing else matches, so an isolated
/// declaration keeps reporting no references instead of falling back.
pub(crate) fn merge_canonical_locations(
    canonical: Option<Vec<Location>>,
    corsa: Option<Vec<Location>>,
    authored: Option<Vec<Location>>,
) -> Option<Vec<Location>> {
    let canonical_answered_empty = matches!(canonical.as_deref(), Some([]));
    merge_authored_locations(merge_authored_locations(canonical, corsa), authored)
        .or_else(|| canonical_answered_empty.then(Vec::new))
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{Position, Range, Url};

    use super::*;

    fn location(uri: &str, line: u32, start: u32, end: u32) -> Location {
        Location {
            uri: Url::parse(uri).unwrap(),
            range: Range {
                start: Position::new(line, start),
                end: Position::new(line, end),
            },
        }
    }

    #[test]
    fn keeps_authored_hits_corsa_never_saw() {
        let sfc = "file:///app/App.vue";
        let merged = merge_authored_locations(
            Some(vec![location(sfc, 3, 6, 16), location(sfc, 4, 31, 41)]),
            Some(vec![
                location(sfc, 3, 6, 16),
                location(sfc, 4, 31, 41),
                location(sfc, 9, 18, 28),
            ]),
        )
        .unwrap();

        assert_eq!(
            merged,
            vec![
                location(sfc, 3, 6, 16),
                location(sfc, 4, 31, 41),
                location(sfc, 9, 18, 28),
            ]
        );
    }

    #[test]
    fn keeps_cross_file_hits_the_authored_sweep_cannot_see() {
        let sfc = "file:///app/App.vue";
        let other = "file:///app/other.ts";
        let merged = merge_authored_locations(
            Some(vec![location(other, 0, 6, 16)]),
            Some(vec![location(sfc, 3, 6, 16)]),
        )
        .unwrap();

        assert_eq!(
            merged,
            vec![location(other, 0, 6, 16), location(sfc, 3, 6, 16)]
        );
    }

    #[test]
    fn deduplicates_interleaved_ranges_with_the_same_start() {
        let sfc = "file:///app/App.vue";
        let short = location(sfc, 3, 6, 12);
        let long = location(sfc, 3, 6, 16);
        let mut multiline = location(sfc, 3, 6, 8);
        multiline.range.end.line = 4;
        let expected = vec![short.clone(), long.clone(), multiline.clone()];

        for hits in [
            vec![
                short.clone(),
                long.clone(),
                short.clone(),
                multiline.clone(),
            ],
            vec![long.clone(), short.clone(), long.clone(), multiline.clone()],
            vec![multiline.clone(), short.clone(), multiline, long],
        ] {
            assert_eq!(
                merge_authored_locations(Some(hits.clone()), None),
                Some(expected.clone())
            );
            assert_eq!(
                merge_authored_locations(None, Some(hits)),
                Some(expected.clone())
            );
        }
    }

    #[test]
    fn preserves_position_and_uri_order_while_merging_all_three_sources() {
        let sfc = "file:///app/App.vue";
        let other = "file:///app/Other.vue";
        let earlier = location(other, 2, 20, 24);
        let short = location(sfc, 3, 6, 12);
        let long = location(sfc, 3, 6, 16);
        let other_at_same_start = location(other, 3, 6, 8);
        let later_character = location(sfc, 3, 7, 8);
        let later_line = location(sfc, 4, 0, 1);
        let merged = merge_canonical_locations(
            Some(vec![long.clone(), earlier.clone(), short.clone()]),
            Some(vec![
                short.clone(),
                other_at_same_start.clone(),
                long.clone(),
            ]),
            Some(vec![
                later_line.clone(),
                short.clone(),
                later_character.clone(),
            ]),
        );

        assert_eq!(
            merged,
            Some(vec![
                earlier,
                short,
                long,
                other_at_same_start,
                later_character,
                later_line,
            ])
        );
    }

    #[test]
    fn falls_back_to_either_side_alone() {
        let sfc = "file:///app/App.vue";
        assert_eq!(
            merge_authored_locations(None, Some(vec![location(sfc, 3, 6, 16)])),
            Some(vec![location(sfc, 3, 6, 16)])
        );
        assert_eq!(
            merge_authored_locations(Some(vec![location(sfc, 3, 6, 16)]), None),
            Some(vec![location(sfc, 3, 6, 16)])
        );
        assert_eq!(merge_authored_locations(None, None), None);
    }

    #[test]
    fn canonical_hits_keep_the_blocks_the_project_document_missed() {
        let sfc = "file:///app/App.vue";
        let other = "file:///app/other.vue";
        let merged = merge_canonical_locations(
            Some(vec![location(sfc, 3, 6, 16), location(other, 1, 0, 6)]),
            Some(vec![location(sfc, 3, 6, 16), location(sfc, 4, 31, 41)]),
            Some(vec![location(sfc, 3, 6, 16), location(sfc, 9, 18, 28)]),
        )
        .unwrap();

        assert_eq!(
            merged,
            vec![
                location(other, 1, 0, 6),
                location(sfc, 3, 6, 16),
                location(sfc, 4, 31, 41),
                location(sfc, 9, 18, 28),
            ]
        );
    }

    #[test]
    fn an_empty_canonical_answer_stays_authoritative() {
        assert_eq!(
            merge_canonical_locations(Some(vec![]), None, None),
            Some(vec![])
        );
        assert_eq!(merge_canonical_locations(None, None, None), None);
    }

    #[test]
    fn an_empty_canonical_answer_still_keeps_authored_hits() {
        let sfc = "file:///app/App.vue";
        let merged = merge_canonical_locations(
            Some(vec![]),
            None,
            Some(vec![location(sfc, 3, 6, 16), location(sfc, 9, 18, 28)]),
        )
        .unwrap();

        assert_eq!(
            merged,
            vec![location(sfc, 3, 6, 16), location(sfc, 9, 18, 28)]
        );
    }
}
