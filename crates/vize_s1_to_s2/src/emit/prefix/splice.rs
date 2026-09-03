//! Span-preserving emission for the identifier-prefix rewrite — the
//! verbatim port of `vize_atelier_core::steps::expression::splice`
//! (Davinci P1-9), whose three retired-loop behaviors are load-bearing
//! for byte parity: out-of-range edits drop, same-position edits emit in
//! reversed construction order, and one edit emits `prefix` then
//! `suffix`.

use alloc::vec::Vec as StdVec;

use vize_s0::String;

/// Splice pure insertions into `original` in one forward pass.
///
/// `rewrites` are `(position, prefix)` pairs and `suffix_rewrites` are
/// `(position, suffix)` pairs in construction order. `wrapper_offset` is
/// subtracted from every position (1 on the legacy wrapped-parse path
/// whose spans count the synthetic `(`, 0 for content-relative spans).
pub(super) fn splice_insertions(
    original: &str,
    rewrites: StdVec<(usize, String)>,
    suffix_rewrites: StdVec<(usize, String)>,
    wrapper_offset: usize,
) -> String {
    let mut all_rewrites: StdVec<(usize, String, String)> = rewrites
        .into_iter()
        .map(|(pos, prefix)| (pos, prefix, String::default()))
        .collect();
    for (pos, suffix) in suffix_rewrites {
        all_rewrites.push((pos, String::default(), suffix));
    }
    all_rewrites.sort_by_key(|rewrite| core::cmp::Reverse(rewrite.0));

    let inserted: usize = all_rewrites
        .iter()
        .map(|(_, prefix, suffix)| prefix.len() + suffix.len())
        .sum();
    let mut result = String::with_capacity(original.len() + inserted);
    let mut cursor = 0usize;
    for (pos, prefix, suffix) in all_rewrites.iter().rev() {
        let pos = pos.saturating_sub(wrapper_offset);
        if pos > original.len() {
            continue;
        }
        result.push_str(&original[cursor..pos]);
        result.push_str(prefix);
        result.push_str(suffix);
        cursor = pos;
    }
    result.push_str(&original[cursor..]);
    result
}

/// Splice whole-span replacements into `original` in one forward pass;
/// spans not fully inside `original` are dropped (the retired
/// `start < len && end <= len` guard of the codegen visitor's
/// `apply_rewrites`).
pub(super) fn splice_replacements(
    original: &str,
    mut rewrites: StdVec<(usize, usize, String)>,
) -> String {
    if rewrites.is_empty() {
        return String::from(original);
    }
    rewrites.sort_by_key(|rewrite| core::cmp::Reverse(rewrite.0));
    let grown: usize = rewrites
        .iter()
        .map(|(_, _, replacement)| replacement.len())
        .sum();
    let mut result = String::with_capacity(original.len() + grown);
    let mut cursor = 0usize;
    for (start, end, replacement) in rewrites.iter().rev() {
        if *start >= original.len() || *end > original.len() {
            continue;
        }
        result.push_str(&original[cursor..*start]);
        result.push_str(replacement);
        cursor = *end;
    }
    result.push_str(&original[cursor..]);
    result
}

#[cfg(test)]
mod tests {
    use super::{splice_insertions, splice_replacements};
    use alloc::vec;
    use vize_s0::String;

    #[test]
    fn insertions_prefix_and_suffix_mix_exact() {
        let spliced = splice_insertions(
            "a + b",
            vec![(0, String::from("_ctx.")), (4, String::from("_ctx."))],
            vec![(5, String::from(".value"))],
            0,
        );
        assert_eq!(spliced.as_str(), "_ctx.a + _ctx.b.value");
    }

    #[test]
    fn insertions_wrapper_offset_shifts_spans_exact() {
        let spliced = splice_insertions("a", vec![(1, String::from("_ctx."))], vec![], 1);
        assert_eq!(spliced.as_str(), "_ctx.a");
    }

    #[test]
    fn insertions_drop_out_of_range_edit_exact() {
        let spliced = splice_insertions("(a) = b", vec![], vec![(8, String::from(".value"))], 0);
        assert_eq!(spliced.as_str(), "(a) = b");
    }

    #[test]
    fn insertions_same_position_order_matches_retired_loop() {
        let spliced = splice_insertions(
            "ab",
            vec![(2, String::from("_ctx."))],
            vec![(2, String::from(")")), (2, String::from(".value"))],
            0,
        );
        assert_eq!(spliced.as_str(), "ab.value)_ctx.");
    }

    #[test]
    fn replacements_splice_in_source_order_exact() {
        let spliced = splice_replacements(
            "foo + bar",
            vec![
                (0, 3, String::from("_ctx.foo")),
                (6, 9, String::from("_ctx.bar")),
            ],
        );
        assert_eq!(spliced.as_str(), "_ctx.foo + _ctx.bar");
    }

    #[test]
    fn replacements_drop_out_of_range_span_exact() {
        let spliced = splice_replacements(
            "a + b",
            vec![
                (0, 1, String::from("_ctx.a")),
                (7, 8, String::from("_ctx.x")),
            ],
        );
        assert_eq!(spliced.as_str(), "_ctx.a + b");
    }
}
