# P2-11 Installment 27 — Dynamic component model arguments (2026-08-26)

> Part of the [P2-11 series record](../p2-11.md), split per installment.

[#4933](https://github.com/ubugeeei-prod/vize/pull/4933) made component
`v-model:[arg]` part of the S2 DOM lane instead of leaving it behind the old
emitter. The merge commit is `f9a56d1fc0c96b7c399913788df164a5aaede676`.

`ui.model` now carries the model argument as a `DynamicName`, and Folio
round-trips that argument with a `name=` field while still accepting the older
argument-less spelling. The DOM emitter realizes static and JavaScript dynamic
model names into the same prop family as the shipped lane: model value key,
`onUpdate:*` listener key, and `*Modifiers` key. Dynamic arguments set
`FULL_PROPS`; static arguments keep `PROPS` and dynamic-prop arrays.

The durable current witness is:

- [`emit_model.rs`](../../../../crates/vize_s1_to_s2/tests/emit_model.rs)
  — direct S2 emitter pins for dynamic component model arguments and modifiers.
- [`davinci_s2_model.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_model.rs)
  — S2-vs-shipped byte-for-byte fixtures for component dynamic model args.
- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  — patch-flag fixtures proving dynamic model args use `FULL_PROPS`.
- [`davinci-storage-policy.test.ts`](../../../../tests/tooling/davinci-storage-policy.test.ts)
  — reviewed owned-storage inventory for the new model-key helpers.

This installment also narrows the P2-9 surface/hoist comparator by removing
the dynamic-model-argument skip class; the S2 representation can now compare
that surface directly.

This installment does not tick P2-11. Malformed slot-region guards, the
production-lane switch, full-corpus comparison count, remaining patch-flag
program, and DOM allocation budget remain task-level gates.
