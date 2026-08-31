# P2-11 Installment 64 - Raw Handler Expressions

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5531](https://github.com/ubugeeei-prod/vize/pull/5531), merged
> 2026-08-31 at `589daf801`.

This installment keeps authored event handler expressions raw where the shipped
DOM lane does. Expression-arrow handlers still wrap, but direct arrow, null,
typed component and multiline block handlers retain their authored payloads.

The durable witnesses are:

- [`davinci_s2_handler_parity.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_handler_parity.rs)
  - compares reduced real-project handler cases byte-for-byte against the
    shipped DOM lane.
- [`wrapped.rs`](../../../../crates/vize_s1_to_s2/src/emit/on/wrapped.rs)
  - owns the direct-vs-wrapped handler classification.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
