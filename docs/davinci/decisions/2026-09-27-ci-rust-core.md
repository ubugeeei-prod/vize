# Affected Rust archive and shard wiring

Tracked in [#6861](https://github.com/ubugeeei-prod/vize/issues/6861),
[#6862](https://github.com/ubugeeei-prod/vize/issues/6862) and
[#6863](https://github.com/ubugeeei-prod/vize/issues/6863).

Wire the affected Rust plan to one nextest archive and four independent PR
shards. The merge queue retains the complete timed workspace test phases,
required TSGO environment, shared differential feature tail and fixture checks.
The strict Rust report accepts success only from every required executing job.

Install the declared Node runtime before source planning and Rust 1.98.0 only
when the coarse plan requires Cargo metadata. Restrict documentation exemptions
to Markdown so a Rust file under docs still receives conservative validation.
Canonical `vize_l0` through `vize_l4` prefixes, including derive, conversion
and SSR parts, retain browser validation when crates move from legacy names.
Keep existing compiler prefixes until those moves complete; renaming a compiler
crate must not silently remove its playground lane.

For PRs, use the tested merge's first parent only when the full checkout SHA,
exactly two parents and matching payload head prove the relationship. Read
commit headers before fetching so shallow checkouts can prove the base. Fetch
that exact base and share it across source and Rust planning; otherwise retain
the event base. Merge groups retain their event base and full validation.
Synthetic shallow Git tests cover stale event bases and rejected relationships.

Keep this Rust change separate from tooling selection, inventory tiers,
workflow security selection and generated ledgers. Existing JS, tooling and
playground work remains unchanged in this intermediate branch. Preserve the
baseline decisions and split source assurances with move-only commits.

Actual archive/runtime and PR latency require fresh Actions evidence. The
recorded phase baseline and executable fake-Cargo tests prove timing behavior;
they do not establish a speedup or native product completion.

Focused verification on this intermediate branch: 44 executable planner,
archive, timing and workflow tests pass with no skips or cancellations. Root
`vp check` passes on all 15 changed JS/TS/MJS files with zero warnings or errors.
Both changed workflows pass actionlint, and the 350-line growth ratchet passes
against `8ae28f16c`. Fresh PR Actions and actual archive/shard execution remain
required before merging.

The first real archive run `36305270275` transferred and verified every shard,
then exposed two preparation/tier issues. Restore Cargo's empty `target/tmp`
before extraction, using `--extract-overwrite` for the verified archive so the
baked `CARGO_TARGET_TMPDIR` remains writable in the identical workspace.
An actual pinned nextest smoke reproduced the missing-directory failure after
cold archive transfer, then passed all four selected tests with this command.

Exactly three `vize_maestro` runtime cases require real TSGO and are deferred
from `profile.pr` by full exact names, preserving all other tests in the module:

- `ide::rename::corsa_session_tests::concurrent_real_corsa_rename_sessions_are_isolated`
- `ide::rename::corsa_session_tests::direct_first::dependency_change_rearms_the_shared_editor_transport_once`
- `ide::rename::corsa_session_tests::direct_first::direct_first_renames_survive_twenty_session_shutdown_overlap`

These three tests remain unchanged and mandatory in the complete T1 Cargo
workspace lane with required TSGO. They provide no native acceptance credit
until that full lane executes successfully. Pinned nextest list JSON on the
smoke retained both future same-module unit cases: full seven tests, PR four,
and exactly these three exclusions. Fresh four-shard Actions proof is required.
