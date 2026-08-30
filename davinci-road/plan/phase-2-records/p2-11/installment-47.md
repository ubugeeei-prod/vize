# P2-11 Installment 47 - Handler Body And Slot Hardening

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5379](https://github.com/ubugeeei-prod/vize/pull/5379), merged
> 2026-08-30 at `481df679f`.

This installment hardens handler body emission and the slot surfaces that read
those bodies. Multi-statement and expression-like handler forms now keep their
shipped body shape through S2 DOM emission, and the refusal census is updated so
future unsupported cases stay named.

The durable witnesses are:

- [`emit_on_handler_body.rs`](../../../../crates/vize_s1_to_s2/tests/emit_on_handler_body.rs)
  - pins the handler body forms directly at the S2 emitter boundary.
- [`on_body.rs`](../../../../crates/vize_s1_to_s2/src/emit/on_body.rs)
  - centralizes the handler body rendering rules.
- [`davinci_s2_slots.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_slots.rs)
  - keeps slot-facing handler cases byte-identical to the shipped lane.

This installment does not tick P2-11. Handler-body residuals close here; the
hydrated corpus evidence and production-lane switch remain open.
