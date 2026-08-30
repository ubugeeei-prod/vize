# P2-11 Installment 54 - Slot Text Facts

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5396](https://github.com/ubugeeei-prod/vize/pull/5396), merged
> 2026-08-30 at `f80bbb5d3`.

This installment keeps slot text facts aligned with the S2 DOM emitter. Slot
groups that participate in `v-once` and default slot realization now carry the
same text fact shape the shipped codegen observes.

The durable witnesses are:

- [`davinci_s2_text_facts.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_text_facts.rs)
  - compares slot text fact behavior against shipped output.
- [`vslot_pass.rs`](../../../../crates/vize_s1_to_s2/tests/vslot_pass.rs)
  - pins the slot pass consumption law.
- [`consume.rs`](../../../../crates/vize_s1_to_s2/src/pass/vslot/consume.rs)
  and [`group.rs`](../../../../crates/vize_s1_to_s2/src/pass/vslot/group.rs)
  - own the facts consumed by emission.

This installment does not tick P2-11. Slot text facts are aligned; the hydrated
corpus evidence and production-lane switch remain open.
