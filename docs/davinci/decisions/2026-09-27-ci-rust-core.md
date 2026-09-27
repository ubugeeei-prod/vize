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

Authored `docs/davinci/plan/**` is a shared Rust input before any Markdown
exemption in both planners. Compiler CFG thresholds compile
`complexity-metrics.md`; Rust tests compile `key-manifests.md` and
`fact-alpha-schemas.md`; resident and compiler gates consume `budgets.toml`.
Schema, source-map and reach budgets there are runtime test inputs as well.
Keep the entire directory conservative for Rust, JS, tooling and playground
validation so a new contract or renamed dependency cannot become prose-only.
Source-witness tests confirm the actual include paths and both planner results.

The exact `npm/cli/schemas/vize.config.schema.json` path is also a shared Rust
input: `crates/vize/src/config.rs` compiles it into `VIZE_CONFIG_SCHEMA`, which
CLI command paths write to projects. A real Git mutation verifies both planners
select Rust for that schema; unrelated npm schemas keep their existing scope.
This changes selection only, with no product output or pipeline stage change.

Restack on parent `afde2f3c2` after preserving successful complete PR proofs for
`806cb00f3` and `123484db5`. Drop merged selector snapshots, preserve every
decision and capture link, and retain the parent's queue replay deduplication.

After #6901's actual queue merge `dd047952c`, preserve main's inventory
composite while transporting core, unfiltered T1 shards and tooling in order.
Keep the preceding successful PR heads and complete queue evidence intact.

#6933's failed-only retry exposed an archive name based on the consumer's
`run_attempt`, although the successful producer remained in attempt 1. Name
both upload and download by exact `run_id` and immutable `github.sha` instead.
A completed full rerun overwrites that same artifact; mutually exclusive T0/T1
builders remain the sole producer. Use no wildcard or latest-artifact fallback.
Keep receipt source SHA/tree, absolute workspace, runner, nextest and content
hash validation before extraction. Executable temporary-Git/FS tests retain an
attempt-1 producer, retry only its attempt-2 consumer, replace a completed full
rerun, and reject another run, source revision or corrupted archive.

The feature-tail contract compares NUL-delimited arguments recorded by the
actual fake Cargo subprocess, in execution order, for success and failure.
It no longer requires one command per YAML line; environment and nonzero
propagation checks remain unchanged.

Integrate actual main `c6916de9c` at the core head before native stack queueing.
Retain the instruction-count dependency and main's level inventory action,
native package preparation and full UI acceptance. Resolve only the obsolete
source-job declaration in the split workflow test; each stack prefix must
merge into main without relying on a later member's conflict resolution.
Preserve the previous successful heads and replay only each descendant's
reviewed changes before fresh current-head Actions and the atomic queue.
