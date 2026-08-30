# P2-11 Installment 58 - CI DOM Corpus Lane

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5401](https://github.com/ubugeeei-prod/vize/pull/5401), merged
> 2026-08-30 at `e7715fbc5`.

This installment runs the feature-gated DOM corpus entry in the required Rust CI
job. On ordinary pull requests it exercises the committed battery because the
hydrated corpus environment variable is unset; the full canonical evidence run
remains a separate hydrated fixture operation.

The durable witnesses are:

- [`check.yml`](../../../../.github/workflows/check.yml)
  - runs `davinci_dom_corpus` explicitly after `cargo test --workspace`.
- [`davinci-lowering-ci-lane.test.ts`](../../../../tests/tooling/davinci-lowering-ci-lane.test.ts)
  - pins the check-job command and Cargo feature registration.
- [`davinci_dom_corpus.rs`](../../../../crates/vize_s1_to_s2/tests/davinci_dom_corpus.rs)
  - reports when the run is committed-battery only.

This installment does not tick P2-11. CI now guards the battery lane; hydrated
corpus evidence and the production-lane switch remain open.
