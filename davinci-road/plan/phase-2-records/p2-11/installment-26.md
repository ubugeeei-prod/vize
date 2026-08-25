# P2-11 Installment 26 — Model listener patch order (2026-08-25)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4929](https://github.com/ubugeeei-prod/vize/pull/4929) extended the
patch-flag equivalence witness after installment 25's dynamic-component cases
landed. The merge commit is `a1d7208bd56cc3f23d7839155ee9f8b90c1eba41`.

The S2-vs-shipped patch-flag battery now covers component `v-model` paired
with an explicit `@update:modelValue` listener in both source orders. The
witness pins not only the `PROPS` flag, but also the shipped dynamic-prop array
order: `v-model` before listener keeps `modelValue` first, while listener
before `v-model` keeps `onUpdate:modelValue` first.

The durable current witness is:

- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  — S2-vs-shipped byte-for-byte coverage plus explicit patch-site extraction.
- [`support/mod.rs`](../../../../crates/vize_atelier_dom/tests/support/mod.rs)
  — shared patch-site extraction used by the DOM-lane differential harness.

This installment does not tick P2-11. Malformed slot-region guards, the
production-lane switch, full-corpus comparison count, remaining patch-flag
program, and DOM allocation budget remain task-level gates.
