# P2-11 Installment 102 - Static V-For Key Fragment Flags

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: _pending - the number, merge date and squash SHA are filled in at
> merge, as every prior installment's line was._

This installment closes a reduced S2 DOM residual from the committed fixture
tree: a `v-for` item with static `key="..."` must still emit the static key on
the item props, but it must not make the wrapping fragment keyed. The shipped
DOM lane only treats dynamic `:key` bindings as the fragment-key signal for
plain `v-for` items. Static keys belong to item props, so the surrounding
fragment keeps `UNKEYED_FRAGMENT`.

The same rule also applies to `v-memo` item reuse guards. A static key is not
read back as `_cached.key === ...`; only a dynamic item key participates in
that guard. The S2 DOM emitter now shares that dynamic-key decision instead of
letting the prop key path and the fragment-key path drift.

## Evidence

`crates/vize_atelier_dom/tests/davinci_s2_corpus_residuals.rs` promotes the
reduced corpus witness from `vif/chain-keys.vue`, where the `v-else-if`
template contains `<img v-for="src in images" key="img">` between statically
keyed conditional siblings.

`crates/vize_atelier_dom/tests/davinci_s2_for.rs` pins the plain native
`v-for` static-key case byte-for-byte against the shipped lane, while
`crates/vize_atelier_dom/tests/davinci_s2_memo.rs` pins both byte output and
the per-node patch-site expectation for a static-keyed `v-memo` loop.

The focused smoke corpus probe over `crates/vize_s1_to_s2/tests/fixtures`
now compares the committed DOM-output samples with zero divergences. This
installment does not tick P2-11. The production-lane switch remains open, and
the old DOM lane is still the shipped compile path.
