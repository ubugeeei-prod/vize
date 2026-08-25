# P2-11 Installment 22 — Vue 2 filter helper order (2026-08-25)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4919](https://github.com/ubugeeei-prod/vize/pull/4919) closed the helper
ordering gap left after the Vue 2 pipe-filter landing. The merge commit is
`59b76d9a0b6be0e98ab033d1315a809c958c2bbc`.

The legacy filter pass now records when a component's own binding surface needs
`_resolveFilter` before `_resolveComponent`, matching the shipped helper
destructure order for component props such as `<Foo :value="1 | formatId" />`.
The patch-flag witness also covers Vue 2 filters across text, native props,
component props, default slots, and slot outlets so the helper order and patch
sites are pinned together.

The durable current witnesses are:

- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  — S2-vs-shipped byte-for-byte coverage for Vue 2 filter patch sites.
- [`emit_filters.rs`](../../../../crates/vize_ricalco/tests/emit_filters.rs)
  — direct S2 emission snapshots for filter assets and wrapped calls.

This installment does not tick P2-11. Malformed slot-region guards, the
production-lane switch, full-corpus comparison count, patch-flag equivalence
program, and DOM allocation budget remain task-level gates.
