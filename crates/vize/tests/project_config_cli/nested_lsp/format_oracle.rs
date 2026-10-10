//! Apply every public edit to its original buffer before comparing the oracle.
use super::Value;
const FORMATTED: &str = include_str!(
    "../../../../../tests/_fixtures/differential/config/nested-project-settings-8371/Formatted.vue.txt"
);

pub(super) fn expected(ty: &str, quote: &str) -> String {
    FORMATTED.replace("VALUE_TYPE", ty).replace("'", quote)
}

pub(super) fn apply(source: &str, edits: &Value) -> String {
    // This fixed corpus has only ASCII text: byte and UTF-16 positions agree.
    assert!(source.is_ascii());
    let mut edits = edits
        .as_array()
        .expect("enabled formatting returns edits")
        .iter()
        .map(|edit| {
            (
                offset(source, &edit["range"]["start"]),
                offset(source, &edit["range"]["end"]),
                edit["newText"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    edits.sort_by_key(|(start, end, _)| std::cmp::Reverse((*start, *end)));
    let mut result = source.to_owned();
    let mut previous = source.len();
    for (start, end, text) in edits {
        assert!(
            start <= end && end <= previous,
            "overlapping or invalid edits"
        );
        result.replace_range(start..end, text);
        previous = start;
    }
    result
}

fn offset(source: &str, position: &Value) -> usize {
    let line = position["line"].as_u64().unwrap() as usize;
    let character = position["character"].as_u64().unwrap() as usize;
    assert!(character <= source.split('\n').nth(line).unwrap().len());
    source
        .split('\n')
        .take(line)
        .map(|line| line.len() + 1)
        .sum::<usize>()
        + character
}
