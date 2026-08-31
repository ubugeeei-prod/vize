# P2-11 Installment 41 - Corpus Comparison Count

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5359](https://github.com/ubugeeei-prod/vize/pull/5359), merged
> 2026-08-30 at `5b5ac0924`.

This installment pins the comparison-count contract for the hydrated P2-11
DOM differential record. The corpus scope is counted by manifest project row,
not by unique fixture path: the current compiler-surface lane must record
144 DOM-output comparisons. A 142-comparison result is stale because the
`primevue` fixture path contributes three project rows: `primevue`,
`primevue-volt` and `primevue-showcase`.

The durable witnesses are:

- [`davinci-corpus-comparison-count.test.ts`](../../../../tests/tooling/davinci-corpus-comparison-count.test.ts)
  - asserts the current manifest yields 144 compiler comparisons, while the
    unique fixture-path count remains 142.
- [`corpus-baseline-artifact.mjs`](../../../../tools/rust/davinci_corpus.rs)
  - exposes the shared expected-count helper used by corpus baseline and diff
    scope proof.
- [`p2-11.md`](../p2-11.md)
  - records that the count contract is pinned but the hydrated zero-divergence
    corpus run remains open.

This installment does not tick P2-11. It closes the comparison-count blocker
only; the production-lane switch, hydrated zero-divergence corpus run and
remaining patch-flag program stay open.
