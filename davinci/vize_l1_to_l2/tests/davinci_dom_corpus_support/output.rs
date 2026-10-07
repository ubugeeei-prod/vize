//! Existing whole-output mismatch rendering for corpus receipts.
fn preview(source: &str) -> String {
    source
        .lines()
        .take(4)
        .collect::<Vec<_>>()
        .join("\\n")
        .chars()
        .take(320)
        .collect()
}

pub(super) fn first_diff(left: &str, right: &str) -> usize {
    left.as_bytes()
        .iter()
        .zip(right.as_bytes())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| left.len().min(right.len()))
}

pub(super) fn mismatch_window(source: &str, other: &str) -> String {
    let diff = first_diff(source, other);
    let start = source[..diff]
        .char_indices()
        .rev()
        .nth(80)
        .map_or(0, |(index, _)| index);
    let end = source[diff..]
        .char_indices()
        .nth(180)
        .map_or(source.len(), |(index, _)| diff + index);
    preview(&source[start..end])
}
