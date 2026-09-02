# P2-11 Installment 74 - Directive Helper Priority

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5567](https://github.com/ubugeeei-prod/vize/pull/5567), merged
> 2026-09-01 at `8a420bc402`.

This installment keeps preferred rank-2 helpers ahead of body-order-only
helpers when ordering the Vue helper preamble. Custom directives combined with
event modifiers now have byte-for-byte parity coverage.

The durable witnesses are:

- [`emit_dirs.rs`](../../../../crates/vize_s1_to_s2/tests/emit_dirs.rs)
  - covers directive helper ordering in the S2 emit lane.
- [`davinci_dom_corpus.rs`](../../../../crates/vize_s1_to_s2/tests/davinci_dom_corpus.rs)
  - keeps the differential battery runnable by default.

This installment does not tick P2-11. The production-lane switch remains open.
