# Handoff: perf/l1-to-l2-cold-instances (variant B), 2026-11-19

Delete this `.handoff/` directory before anything from this branch merges.

## State

The branch is variant B (cold recording lowering and recorded passes compiled once), rebased onto
main 0cced76570 as 76af26d29a. There is NO PR, because it regresses instruction rows. The LSP CSS
catalog compaction landed separately as #13374 (−1,000,768 B).

## Screen (run 44463928317, scratch driver `perf/pb-driver`, base 0cced76570)

The subsets were built as temporary commits on branches `perf/pb-cold-b1` and `perf/pb-cold-b2`.

| Variant | ci-opt bytes vs base | Instruction rows moved / up |
| --- | ---: | --- |
| B (both halves) | −325,312 | 24 / 14 up (dom compile up to +0.31%, ssr small +0.16%, vapor stress-interp +0.09%, s1_to_s2 +0.16%) |
| B1: `pass::recorded` wrappers only (pass.rs, pass/recorded.rs) | −186,240 | 24 / 14 up (same ssr/vapor/s1_to_s2 rows as B, plus dom) |
| B2: `lower_recorded_with_caps_and_comment_policy` + capture.rs only | −137,216 | 6 / 6 up, **only** `atelier_dom_compile_*` (+0.01..+0.21%) |

What the screen shows:
- B2 adds a root to `vize_l1_to_l2` without moving any SSR, Vapor or s1_to_s2 row. Its 6 rows
  come from `vize_atelier_dom` losing its copy of the recording lowering, which re-partitions
  that crate's 16 CGUs.
- B1's new `pass/recorded.rs` module re-partitions `vize_l1_to_l2` itself (the SSR, Vapor and
  s1_to_s2 rows move exactly as in B), and it also drops the pass copies from `vize_l4`.

## Next steps (if pursued)

1. B2 is the cheapest candidate. Look for a form of the dom side that keeps `vize_atelier_dom`'s
   hot CGUs intact. You can check the partition locally: the parent agent's
   `-Zprint-mono-items` / `-Zdump-mono-stats` scripts compare the CGU grouping base vs head
   in about 40 s per try, and the partition is target-independent. Then screen on CI with
   `gh workflow run level-instruction-counts.yml --ref perf/pb-driver -f roles='["base=<sha>","v=<sha>",...]'`.
2. B1: place the wrappers so that `vize_l1_to_l2`'s partition stays identical to main (check it
   locally with the same tooling before dispatching).
3. Never open a PR while any row is up (#13345 was closed for 38 rows).
