# P2-11 Installment 48 - Static Class Patch Flags

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5380](https://github.com/ubugeeei-prod/vize/pull/5380), merged
> 2026-08-30 at `b67020bde`.

This installment aligns S2 DOM static class emission with the shipped patch-flag
program. Static class props now preserve the shipped no-flag and dynamic-prop
decisions for the covered native and component cases.

The durable witnesses are:

- [`davinci_s2_static_class_patch.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_static_class_patch.rs)
  - records the S2-vs-shipped byte and patch-site behavior for static classes.
- [`props.rs`](../../../../crates/vize_s1_to_s2/src/emit/props.rs)
  - owns the prop emission path whose static-class behavior changed.
- [`consumer-migration-surfaces.md`](../../consumer-migration-surfaces.md)
  - records the affected S2 DOM surface inventory.

This installment does not tick P2-11. Static class patch flags are pinned, while
the hydrated corpus evidence and production-lane switch remain open.
