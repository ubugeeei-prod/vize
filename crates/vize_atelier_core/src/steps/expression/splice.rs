//! Span-preserving emission for the identifier-prefix transform
//! (Davinci P1-9).
//!
//! The AST walk ([`super::collector::IdentifierCollector`], or the
//! standalone `_ctx.` collector in `prefix.rs`) decides WHAT to rewrite:
//! pure insertions (a prefix before an identifier span, a suffix after it)
//! or whole-span replacements, all in byte positions of the walked text.
//! This module owns turning those decisions into output bytes: one forward
//! pass splices the edits into the ORIGINAL text, so every byte the walk
//! did not name survives verbatim — never an oxc_codegen re-print, whose
//! formatting would not be byte-stable.
//!
//! Byte-parity contract: these splicers replace the retired end-to-start
//! `insert_str`/`replace_range` loops and must reproduce them exactly.
//! Three retired behaviors are load-bearing:
//!
//! - **Out-of-range drop.** The retired loops checked each position against
//!   the (growing) result length while processing end-to-start, which is
//!   provably the same as dropping every edit past the end of the original
//!   text: an in-range edit can never lift a beyond-the-end edit into
//!   range, because positions only decrease along the loop. This is the
//!   wrapper-paren overshoot class P1-7 documented (an assignment-target
//!   paren scan running through the legacy `(content)` wrapper).
//! - **Same-position order.** `insert_str` at an already-edited position
//!   displaces the earlier insertion, so the *last-processed* edit lands
//!   *first* in the output. Forward emission therefore walks the
//!   stable-descending-sorted edit list in reverse: ascending positions,
//!   and reversed construction order within a position.
//! - **Prefix-before-suffix.** One edit's suffix was inserted first and its
//!   prefix then displaced it at the same position, so an edit emits as
//!   `prefix` then `suffix`.

use vize_carton::{FxHashSet, String};

/// Splice pure insertions into `original` in one forward pass.
///
/// `rewrites` are `(position, prefix)` pairs and `suffix_rewrites` are
/// `(position, suffix)` pairs, exactly as the collector produced them.
/// `wrapper_offset` is subtracted from every position (1 on the legacy
/// wrapped-parse path whose spans count the synthetic `(`, 0 for
/// content-relative spans on the retained and program paths).
pub(super) fn splice_insertions(
    original: &str,
    rewrites: FxHashSet<(usize, String)>,
    suffix_rewrites: Vec<(usize, String)>,
    wrapper_offset: usize,
) -> String {
    let mut all_rewrites: Vec<(usize, String, String)> = rewrites
        .into_iter()
        .map(|(pos, prefix)| (pos, prefix, String::default()))
        .collect();
    for (pos, suffix) in suffix_rewrites {
        all_rewrites.push((pos, String::default(), suffix));
    }
    // Stable descending sort, then emitted back-to-front: reversed
    // construction order within a position is the retired loop's
    // displacement order (see the module docs).
    all_rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

    let inserted: usize = all_rewrites
        .iter()
        .map(|(_, prefix, suffix)| prefix.len() + suffix.len())
        .sum();
    let mut result = String::with_capacity(original.len() + inserted);
    let mut cursor = 0usize;
    for (pos, prefix, suffix) in all_rewrites.iter().rev() {
        let pos = pos.saturating_sub(wrapper_offset);
        // Out-of-range drop (module docs); dropped edits sort last, so the
        // cursor never moves past a kept one.
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

/// Splice whole-span replacements into `original` in one forward pass.
///
/// `rewrites` are `(start, end, replacement)` triples in content
/// coordinates; spans come from one AST walk and never overlap. Spans not
/// fully inside `original` are dropped — the retired
/// `start < len && end <= len` guard.
pub(super) fn splice_replacements(
    original: &str,
    mut rewrites: Vec<(usize, usize, String)>,
) -> String {
    rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
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
    use vize_carton::{FxHashSet, String};

    /// The retired string-rewriting loop, kept verbatim as the test oracle:
    /// stable descending sort, then end-to-start `insert_str` with the
    /// grow-aware bounds check.
    fn retired_insert_loop(
        original: &str,
        rewrites: &FxHashSet<(usize, String)>,
        suffix_rewrites: &[(usize, String)],
        wrapper_offset: usize,
    ) -> String {
        let mut all_rewrites: Vec<(usize, String, String)> = rewrites
            .iter()
            .cloned()
            .map(|(pos, prefix)| (pos, prefix, String::default()))
            .collect();
        for (pos, suffix) in suffix_rewrites {
            all_rewrites.push((*pos, String::default(), suffix.clone()));
        }
        all_rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
        let mut result = String::new(original);
        for (pos, prefix, suffix) in all_rewrites {
            let adjusted_pos = pos.saturating_sub(wrapper_offset);
            if adjusted_pos <= result.len() {
                if !suffix.is_empty() {
                    result.insert_str(adjusted_pos, &suffix);
                }
                if !prefix.is_empty() {
                    result.insert_str(adjusted_pos, &prefix);
                }
            }
        }
        result
    }

    fn prefix_set(entries: &[(usize, &str)]) -> FxHashSet<(usize, String)> {
        entries
            .iter()
            .map(|(pos, prefix)| (*pos, String::new(*prefix)))
            .collect()
    }

    fn suffixes(entries: &[(usize, &str)]) -> Vec<(usize, String)> {
        entries
            .iter()
            .map(|(pos, suffix)| (*pos, String::new(*suffix)))
            .collect()
    }

    #[test]
    fn insertions_prefix_and_suffix_mix_exact() {
        // `a + b` with `_ctx.` on both identifiers and `.value` after `b`,
        // content-relative spans.
        let rewrites = prefix_set(&[(0, "_ctx."), (4, "_ctx.")]);
        let suffix_rewrites = suffixes(&[(5, ".value")]);
        let spliced = splice_insertions("a + b", rewrites.clone(), suffix_rewrites.clone(), 0);
        assert_eq!(spliced.as_str(), "_ctx.a + _ctx.b.value");
        assert_eq!(
            spliced,
            retired_insert_loop("a + b", &rewrites, &suffix_rewrites, 0)
        );
    }

    #[test]
    fn insertions_wrapper_offset_shifts_spans_exact() {
        // Wrapped-parse coordinates: `(a)` puts `a` at span 1.
        let rewrites = prefix_set(&[(1, "_ctx.")]);
        let spliced = splice_insertions("a", rewrites.clone(), Vec::new(), 1);
        assert_eq!(spliced.as_str(), "_ctx.a");
        assert_eq!(spliced, retired_insert_loop("a", &rewrites, &[], 1));
    }

    #[test]
    fn insertions_drop_out_of_range_edit_exact() {
        // The wrapper-paren overshoot class: a suffix whose paren scan ran
        // past the end of the content is dropped, exactly as the retired
        // bounds check dropped it.
        let suffix_rewrites = suffixes(&[(8, ".value")]);
        let spliced =
            splice_insertions("(a) = b", FxHashSet::default(), suffix_rewrites.clone(), 0);
        assert_eq!(spliced.as_str(), "(a) = b");
        assert_eq!(
            spliced,
            retired_insert_loop("(a) = b", &FxHashSet::default(), &suffix_rewrites, 0)
        );
    }

    #[test]
    fn insertions_at_text_end_append_exact() {
        // Position == length is in range (`<=` in the retired loop).
        let suffix_rewrites = suffixes(&[(1, ".value")]);
        let spliced = splice_insertions("a", FxHashSet::default(), suffix_rewrites.clone(), 0);
        assert_eq!(spliced.as_str(), "a.value");
        assert_eq!(
            spliced,
            retired_insert_loop("a", &FxHashSet::default(), &suffix_rewrites, 0)
        );
    }

    #[test]
    fn insertions_same_position_order_matches_retired_loop() {
        // A prefix and two suffixes colliding on one position: the retired
        // loop's displacement order is the contract, whatever it is.
        let rewrites = prefix_set(&[(2, "_ctx.")]);
        let suffix_rewrites = suffixes(&[(2, ")"), (2, ".value")]);
        let spliced = splice_insertions("ab", rewrites.clone(), suffix_rewrites.clone(), 0);
        assert_eq!(spliced.as_str(), "ab.value)_ctx.");
        assert_eq!(
            spliced,
            retired_insert_loop("ab", &rewrites, &suffix_rewrites, 0)
        );
    }

    #[test]
    fn replacements_splice_in_source_order_exact() {
        let rewrites = vec![
            (0usize, 3usize, String::new("_ctx.foo")),
            (6usize, 9usize, String::new("_ctx.bar")),
        ];
        let spliced = splice_replacements("foo + bar", rewrites);
        assert_eq!(spliced.as_str(), "_ctx.foo + _ctx.bar");
    }

    #[test]
    fn replacements_drop_out_of_range_span_exact() {
        let rewrites = vec![
            (0usize, 1usize, String::new("_ctx.a")),
            (7usize, 8usize, String::new("_ctx.x")),
        ];
        let spliced = splice_replacements("a + b", rewrites);
        assert_eq!(spliced.as_str(), "_ctx.a + b");
    }
}
