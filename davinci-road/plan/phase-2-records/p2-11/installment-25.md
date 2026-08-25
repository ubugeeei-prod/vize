# P2-11 Installment 25 — Dynamic component patch flags (2026-08-25)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4927](https://github.com/ubugeeei-prod/vize/pull/4927) extended the
patch-flag equivalence witness after installment 24's broader matrix landed.
The merge commit is `b72b28b2ea39733f361a2ee87de3b7704f90febc`.

The S2-vs-shipped patch-flag battery now covers dynamic component `:is`
exclusion from dynamic-prop arrays, dynamic component object bind falling back
to `FULL_PROPS`, named component `v-model` dynamic-prop arrays, and component
`v-model` modifier props preserving the same patch surface as the shipped lane.

The durable current witness is:

- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  — S2-vs-shipped byte-for-byte coverage plus explicit patch-site extraction.

This installment does not tick P2-11. Malformed slot-region guards, the
production-lane switch, full-corpus comparison count, remaining patch-flag
program, and DOM allocation budget remain task-level gates.
