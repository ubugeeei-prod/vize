//! Multiple virtual declarations can map to one authored definition.

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location};
use vize_l0::FxHashSet;

pub(super) fn convert(mut locations: Vec<Location>) -> Option<GotoDefinitionResponse> {
    if locations.len() < 2 {
        return locations.pop().map(GotoDefinitionResponse::Scalar);
    }
    let mut seen = FxHashSet::default();
    locations.retain(|location| {
        seen.insert((
            location.uri.clone(),
            location.range.start.line,
            location.range.start.character,
            location.range.end.line,
            location.range.end.character,
        ))
    });
    match locations.as_slice() {
        [] => None,
        [location] => Some(GotoDefinitionResponse::Scalar(location.clone())),
        _ => Some(GotoDefinitionResponse::Array(locations)),
    }
}

#[cfg(test)]
mod tests {
    use super::convert;
    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

    fn location(file: &str, start: (u32, u32), end: (u32, u32)) -> Location {
        Location {
            uri: Url::parse(file).unwrap(),
            range: Range::new(Position::new(start.0, start.1), Position::new(end.0, end.1)),
        }
    }

    #[test]
    fn empty_and_repeated_single_targets_keep_null_and_scalar_shapes() {
        assert_eq!(convert(vec![]), None);
        let target = location("file:///Canvas.vue", (102, 4), (102, 20));
        for count in [1, 2, 4] {
            assert_eq!(
                convert(vec![target.clone(); count]),
                Some(GotoDefinitionResponse::Scalar(target.clone()))
            );
        }
    }

    #[test]
    fn full_authored_identity_deduplicates_stably_without_losing_distinct_targets() {
        let first = location("file:///Canvas.vue", (102, 4), (102, 20));
        let different_end = location("file:///Canvas.vue", (102, 4), (103, 20));
        let different_start = location("file:///Canvas.vue", (102, 5), (102, 20));
        let different_start_line = location("file:///Canvas.vue", (101, 4), (102, 20));
        let different_end_character = location("file:///Canvas.vue", (102, 4), (102, 21));
        let different_uri = location("file:///Other.vue", (102, 4), (102, 20));
        assert_eq!(
            convert(vec![
                first.clone(),
                different_end.clone(),
                first.clone(),
                different_uri.clone(),
                different_start.clone(),
                different_start_line.clone(),
                different_end_character.clone(),
                different_end.clone(),
                different_uri.clone(),
            ]),
            Some(GotoDefinitionResponse::Array(vec![
                first,
                different_end,
                different_uri,
                different_start,
                different_start_line,
                different_end_character,
            ]))
        );
    }
}
