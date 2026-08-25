# P2-11 Installment 23 — Slot outlet same-name names (2026-08-25)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4921](https://github.com/ubugeeei-prod/vize/pull/4921) moved slot outlet
same-name name spellings out of the S2 DOM emitter's local divergence surface.
The merge commit is `891e631710c5c007ebb615c22693c3bb9ac4b627`.

Value-less and blank `<slot :name>` / `<slot v-bind:name>` now lower to the
runtime `name` expression, matching the shipped parser's Vue 3.4 same-name
shorthand behavior before slot-outlet name resolution. The byte-differential
outlet battery now covers bare names, bracket-literal names, member
expressions, same-name shorthand, longhand shorthand, and blank dynamic names.
The storage ratchet was reduced in the same PR for the reviewed
`vize_s0::String` bound-use drop in `lower/slot.rs`.

The durable current witness is:

- [`davinci_s2_outlets.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_outlets.rs)
  — S2-vs-shipped byte-for-byte coverage for slot outlet name and prop shapes.

This installment does not tick P2-11. Malformed slot-region guards, the
production-lane switch, full-corpus comparison count, remaining patch-flag
program, and DOM allocation budget remain task-level gates.
