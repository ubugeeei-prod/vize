# P2-11 Installment 45 - Event Model Slot Residuals

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5373](https://github.com/ubugeeei-prod/vize/pull/5373), merged
> 2026-08-30 at `ee8f222cb`.

This installment closes the event/model/slot residuals that still forced local
S2 DOM refusals. The emitter now preserves the shipped key and helper ordering
for model listeners, dynamic `v-on` keys, slot props and slot bodies while
keeping the already-pinned object spread and hoist shapes intact.

The durable witnesses are:

- [`davinci_s2_colon.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_colon.rs)
  and [`davinci_s2_model.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_model.rs)
  - keep the S2 DOM output byte-identical to the shipped lane for colon and
    model residuals.
- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - covers the slot-body side of the same residual program.
- [`emit_model.rs`](../../../../crates/vize_s1_to_s2/tests/emit_model.rs) and
  [`emit_on.rs`](../../../../crates/vize_s1_to_s2/tests/emit_on.rs)
  - pin the direct emitter behavior before the atelier byte comparison.

This installment does not tick P2-11. It removes a named residual class, while
the hydrated corpus evidence and production-lane switch remain open.
