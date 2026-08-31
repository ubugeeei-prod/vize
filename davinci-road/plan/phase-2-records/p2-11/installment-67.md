# P2-11 Installment 67 - Component Class Binds

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5543](https://github.com/ubugeeei-prod/vize/pull/5543), merged
> 2026-08-31 at `da97fe2d70`.

This installment keeps component `class` binds on the inline props path so the
shipped DOM lane's `_normalizeClass` surface is preserved. Static class-array
component props inside slots now have a reduced S2-vs-shipped witness, and the
component static-props hoist decision treats `class` as an inline-only key.

The durable witnesses are:

- [`davinci_s2_hoist_order.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_hoist_order.rs)
  - compares the reduced static class-array component case byte-for-byte
    against the shipped DOM lane.
- [`props_static.rs`](../../../../crates/vize_s1_to_s2/src/emit/props_static.rs)
  - owns the component static-props hoist key filter that keeps `class`
    bindings inline.

This installment does not tick P2-11. The hydrated corpus evidence and
production-lane switch remain open.
