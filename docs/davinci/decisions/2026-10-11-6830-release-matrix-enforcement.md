# Enforced Matrix evidence for release preflight

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Release preflight dispatches all six Real Project Matrix modes as `enforce`,
including typecheck divergence and the separate Davinci corpus job. The primary
workflow labels a run `Real Project Matrix enforced @ <H>` only when every mode
resolves to `enforce`. The existing release selector requires that title and
the exact source head, workflow path, event, and successful producer jobs.
Manual observational runs remain available under an observational title.

Shard surface reports use schema version 2. Every original surface retains
its actual `budgetMode` and `rawOutcome` alongside its policy-adjusted outcome.
Release validation requires the exact head and shard, all eight original
surface names in order, enforced successful raw outcomes, and no failed names.
An observational failure or skipped typecheck can retain its observational
success status without becoming eligible release evidence.

The runtime requires complete typecheck evidence and exact parity. It retains
the original dependency linkage, authored project coverage, nonempty matching
Vue corpus, seeded clean/broken/repaired mutation packets, and zero unexplained
false positives and false negatives. Lint's original budget must report a
complete successful project count with no failed, unusable, or breached
projects. No budgets, golden files, thresholds, or checks are removed.

This supersedes only the Optional typecheck policy preserved by the
[Matrix attempt record](./2026-10-08-release-matrix-attempt-evidence.md).
Its authenticated attempt selection, historical custody, and 22 complete
producer archives remain required. All five original release workflows,
explicit App E2E qualification, and source-PR native qualification retain
their existing roles. This change grants no evidence to an earlier release
and does not publish or rewrite a retired cut.

The regression controls execute the actual Rust surface producer and the full
22-archive Rust preflight, including normalized observational successes,
omitted typecheck evidence, a retained false negative, and a failed lint
budget. The immutable preceding main accepted those malformed receipts;
the correction refuses them. Primary workflow enum/title controls and the
original selector, corpus, mutation, and preparation suites remain in the
tooling tests. Hosted exact-head Actions and the protected merge queue must
qualify the source before a new release can use it.
