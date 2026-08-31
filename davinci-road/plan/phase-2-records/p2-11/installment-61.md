# P2-11 Installment 61 - Nested Interactive Recovery Comparison

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5467](https://github.com/ubugeeei-prod/vize/pull/5467), merged
> 2026-08-31 at `86f4b794f`.

This installment makes recoverable nested `<a>` and `<button>` tree-construction
cases reach the S2-vs-shipped DOM comparison lane instead of being skipped as
hard old-lane errors. Unrelated invalid end tags stay visible as skip evidence.

The durable witnesses are:

- [`davinci_dom_corpus.rs`](../../../../crates/vize_s1_to_s2/tests/davinci_dom_corpus.rs)
  - asserts nested interactive recoveries are compared, while unrelated invalid
    end tags still block comparison and are reported.
- [`nested_interactive_recovery.rs`](../../../../crates/vize_atelier_dom/tests/nested_interactive_recovery.rs)
  - pins the shipped DOM lane's recoverable-diagnostic behavior.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
