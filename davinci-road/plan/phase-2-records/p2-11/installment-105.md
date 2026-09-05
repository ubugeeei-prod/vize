# P2-11 Installment 105 - Shared DOM Battery For Production Options

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: _pending - the number, merge date and squash SHA are filled in at
> merge, as every prior installment's line was._

This installment makes the curated S2 DOM differential battery reusable by
moving it under the shared test support tree, then feeds that same battery into
the production-option combination matrix. The regular DOM witness remains a
thin byte-for-byte release ratchet, while the production matrix now exercises
the broad DOM surface under the option pairs that real SFCs use together.

The matrix keeps its focused production-layout cases and adds scoped
cached/static child layouts where a dynamic setup component forces a static
sibling through the cached/hoisted props paths with `scope_id` and handler
caching enabled.

## Evidence

`crates/vize_atelier_dom/tests/support/battery/dom.rs` owns the shared curated
DOM battery. `crates/vize_atelier_dom/tests/davinci_s2_dom.rs` and
`crates/vize_atelier_dom/tests/davinci_s2_option_matrix.rs` both consume it.

Focused gates:

```sh
cargo test -p vize_atelier_dom --test davinci_s2_dom --test davinci_s2_option_matrix
node --test tests/tooling/davinci-phase2-ledger.test.ts tests/tooling/source-file-lengths.test.ts
```

This installment does not tick P2-11. The production-lane switch remains open,
and the old DOM lane is still the shipped non-profiled compile path.
