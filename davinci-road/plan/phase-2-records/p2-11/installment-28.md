# P2-11 Installment 28 — SFC style carriers are DOM-inert (2026-08-26)

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5009](https://github.com/ubugeeei-prod/vize/pull/5009), awaiting the
> auto-merge commit.

This installment makes the P2-10 style-block carrier explicit in the DOM
emitter: `vue.css-bind` facts stay in the S2 artifact for phase-4 consumers,
but they do not become DOM `<style>` VNodes. The old SFC style compile path is
still `vize_atelier_sfc`; DOM realization ignores this analysis carrier.

The change is deliberately narrow:

- `Lowered::push_style_block*` skips style blocks with no CSS `v-bind()` facts,
  so an empty carrier is not appended to the root artifact.
- Root-fragment selection ignores non-empty style carriers whose bindings are
  all `vue.css-bind`.
- DOM emission still re-derives page-order ids through the skipped carrier and
  its bindings, so later facts keep their existing ids.
- Authored template `<style>` elements remain ordinary `ui.element style` and
  still emit normally.

The durable witnesses are:

- [`css_bind_append.rs`](../../../../crates/vize_ricalco/tests/css_bind_append.rs)
  — style blocks without CSS `v-bind()` do not append a carrier.
- [`emit_sfc_style_carrier.rs`](../../../../crates/vize_ricalco/tests/emit_sfc_style_carrier.rs)
  — single-root, multi-root, empty-template and real-template-style DOM output
  are pinned exactly.

This installment removes the P2-10 style-carrier item from the current named
remainder. It does not tick P2-11: malformed slot-region guards, the
production-lane switch, full-corpus comparison count, patch-flag program, and
DOM allocation budget remain task-level gates.
