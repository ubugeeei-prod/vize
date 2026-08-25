# P2-11 Installment 24 — Patch-flag matrix expansion (2026-08-25)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4924](https://github.com/ubugeeei-prod/vize/pull/4924) broadened the
patch-flag equivalence witness after the slot-outlet same-name fix landed. The
merge commit is `901cd282c36502e4d8e8dbd221d90571fc25e115`.

The S2-vs-shipped patch-flag battery now covers component class props,
component key listeners, native and component `v-model`, object-spread merge
flags with dynamic prop arrays, component object bind, and nested dynamic slots
inside a fragment. Each case still proves both full emitted-byte equality and
the exact per-node patch flag/comment plus dynamic-prop array extracted from
the generated render function.

The durable current witness is:

- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  — S2-vs-shipped byte-for-byte coverage plus explicit patch-site extraction.

This installment does not tick P2-11. Malformed slot-region guards, the
production-lane switch, full-corpus comparison count, remaining patch-flag
program, and DOM allocation budget remain task-level gates.
