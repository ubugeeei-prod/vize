# P2-11 installment 40 — publish graph firewall

PR: [#5214](https://github.com/ubugeeei-prod/vize/pull/5214)
Issue: [#5213](https://github.com/ubugeeei-prod/vize/issues/5213)
Commit: `be344e787`
Date: 2026-08-29

This installment resolves the publish-graph half of the P2-11 dependency
decision without switching the shipped DOM lane.

The current phase-2 decision is a firewall:

- `vize_davinci`, `vize_s1`, `vize_s2` and `vize_ricalco` stay
  `publish = false` while the IR and lowering contracts are still in phase-2.
- Publishable crates may exercise those stage crates only through
  dev-dependencies whose metadata requirement is `*`, which Cargo strips from
  the packaged release graph.
- Any normal or build dependency from a publishable crate into those unpublished
  stage crates is a release blocker, because it would make the crates.io graph
  unresolvable.

The machine check is
`tests/tooling/davinci-stage-dependencies.test.ts`: it reads locked
`cargo metadata`, asserts the stage crates remain unpublished, and scans every
publishable workspace package for an unstripped stage edge. The existing
`moonbit-publish-crates` test still owns release ordering for crates that are
actually published; this installment pins the P2-11-specific firewall before the
production-lane switch can be attempted.

No DOM emit behavior changes here. The old DOM lane remains the production path;
S2 DOM witnesses remain test-only.

## Validation

- `node --test tests/tooling/davinci-stage-dependencies.test.ts`
- `git diff --check`
