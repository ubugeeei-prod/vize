# CI inventory tiers (2026-09-27)

Tracking issue: [#6864](https://github.com/ubugeeei-prod/vize/issues/6864).

`Check` runs the Croquis consumer census, consumer migration surfaces,
storage policy and generated storage summary checks on every merge group, the daily
schedule and explicit full dispatches, including release validation. They
leave pull requests and ordinary pushes. The inventory step stays before
the JS runtime bootstrap so a stale inventory fails early in the full gate.

The merge queue also runs `check:ci`, including static package checks,
rather than only `check:repo`. Pull requests and ordinary pushes retain
`check:repo`. The full source-check workflow already provides Rust,
tooling, JS package and playground/VRT validation for merge groups; this
change does not add duplicate standalone copies of those jobs.

VRT and required tsgo checks in the reusable source workflow are migrated
alongside affected Rust/tooling selection. Those changes are required to
complete #6864; this inventory change alone does not complete the issue.

The T0 timing goals remain targets. No duration improvement is claimed
until GitHub Actions runs measure the integrated change.

The main inventory composite owns bundle generation, validation and artifact
upload. Keep that composite unchanged when transporting this tier guard onto
the full Rust shard stack; the retired standalone summary command cannot
validate an authored guide as a generated ledger.
