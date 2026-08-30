# P2-11 Installment 51 - DOM Corpus Lane

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5387](https://github.com/ubugeeei-prod/vize/pull/5387), merged
> 2026-08-30 at `41fe266a2`.

This installment adds the P2-11 DOM corpus-runnable entry. By default it runs a
small committed SFC battery; when `VIZE_DAVINCI_DIFFERENTIAL_CORPUS` points at
the canonical fixture root it sweeps hydrated `.vue` files and reports S2
refusals and byte divergences against the shipped DOM lane.

The durable witnesses are:

- [`davinci_dom_corpus.rs`](../../../../crates/vize_s1_to_s2/tests/davinci_dom_corpus.rs)
  - implements the committed battery plus env-widened corpus sweep.
- [`Cargo.toml`](../../../../crates/vize_s1_to_s2/Cargo.toml)
  - registers the feature-gated test entry.
- [`corpus.rs`](../../../../tests/davinci_test_support/src/corpus.rs)
  - supplies the fail-closed canonical corpus scope proof.

This installment does not tick P2-11. It creates the runnable evidence lane;
the hydrated evidence run and production-lane switch remain open.
