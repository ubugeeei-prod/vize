# P2-11 Installment 39 - Recent Patch-Flag Witness

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5212](https://github.com/ubugeeei-prod/vize/pull/5212), merged
> 2026-08-29 at `22674520f`.
> Issue: [#5211](https://github.com/ubugeeei-prod/vize/issues/5211).

This installment expands the patch-flag equivalence witness for the late P2-11
directive and object-spread increments. The S2 DOM lane already matched the
shipped byte output for these surfaces; this pins the exact per-node patch-site
lists so later emitter changes cannot silently preserve bytes in one fixture
while drifting the flag program.

The durable witnesses are:

- [`davinci_s2_recent_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_recent_patch_flags.rs)
  - S2-vs-shipped byte comparison plus exact patch-site extraction for
    `v-show`, `v-html`, `v-text`, `v-cloak`, object `v-bind` modifiers and
    object `v-on` modifiers.
- [`davinci_s2_patch_flags.rs`](../../../../crates/vize_atelier_dom/tests/davinci_s2_patch_flags.rs)
  - the earlier broad matrix remains below the source-length budget; this
    installment keeps the late-surface witness split rather than growing it
    past the enforced limit.
- [`source-file-lengths.test.ts`](../../../../tests/tooling/source-file-lengths.test.ts)
  - guards the split so the witness expansion cannot cross the 350-line
    threshold that originally forced the P2 plan file split.

This installment does not tick P2-11. The production-lane switch, hydrated
full-corpus comparison count, remaining patch-flag program and DOM allocation
budget remain task-level gates.
