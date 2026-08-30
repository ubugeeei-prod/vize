# P2-11 Installment 56 - Line Comment Expression Edges

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5399](https://github.com/ubugeeei-prod/vize/pull/5399), merged
> 2026-08-30 at `4c8a27cae`.

This installment admits line-comment expression edges through S2 DOM emission.
Line comments in child, prop, outlet, template and branch positions now render
through the same shipped expression output path instead of falling into local
unsupported buckets.

The durable witnesses are:

- [`davinci_s2_expression_residuals.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_expression_residuals.rs)
  - records the DOM byte witness for line-comment expression cases.
- [`js.rs`](../../../../crates/vize_s1_to_s2/src/emit/js.rs)
  - owns the emitted JS expression rendering.
- [`emit_unsupported_census.rs`](../../../../crates/vize_s1_to_s2/tests/emit_unsupported_census.rs)
  - keeps the retired unsupported class from silently returning.

This installment does not tick P2-11. It closes the line-comment expression
edge; the hydrated corpus evidence and production-lane switch remain open.
