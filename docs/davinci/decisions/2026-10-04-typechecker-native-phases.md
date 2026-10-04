# Native backend phase evidence

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698); [same-slice decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5976448513).

Initial unchanged main production source: `ac1675fef35c5637c38b1e366a91d6bdfe419885`.
The source must be refreshed and the phase probe rerun after the corrected
performance Stack merges; this independent lane creates no dependency on it.

This is an independent benchmark-only follow-up to the fresh-process type
checker target. Production generation, source membership, compiler semantic
options, checker counts and all 104 instruction ceilings remain unchanged.
The 42.55 ms end-to-end target is unfinished. This experiment implements no
incremental cache, backend replacement, persistent session or product stage.

## Purpose and current status

Existing profiles wrap each native command and the sharded command wall;
they cannot attribute startup, program construction, binding or checking.
The current lane first collects the pinned native runtime's actual
`--extendedDiagnostics` for every CLI shard in generated500,
shared-leaf501-default and shared-barrel500, in max and 1T modes.
The 1T mode selects one front-end server and one Rayon thread; it does not
override the native scheduler. Each receipt retains the actual native arguments,
including the checker count.

Hosted evidence from the unchanged main production source is **unmeasured**
until the exact-head Actions probe completes. A local protocol-only discovery
used an explicitly identified development binary, without a main or speed
claim: all 15 original/replayed native diagnostic streams, statuses and
program-file counts matched across the three full corpora. Runtime 7.0.2
emitted Config, Parse, Bind, Check, Emit and Total time. It reported no
independent startup or program-construction field. The strengthened repeat also
matched every native program member byte, native binary and config before/after
all 15 replays. Local source bytes equal development commit
`e027dacb36da6ae5614288fc8fcd7063753b5b16`; that CLI was built before the commit,
with binary SHA-256 `798d5775728fac8c84eca26774b40652e7862e80bce59ac2dc9ccb3a739f1fab`.
This is separate from unchanged-main hosted evidence. These missing quantities
remain `null / unknown`; an unexplained residual does not establish OS startup.

## Protocol

- Build a fresh CLI from the infrastructure PR's exact head with the existing
  `ci-opt` settings, after proving that the production tree matches the main
  merge base. Pin the CLI and standalone projection executable and record
  the native runtime's exact path, version and SHA-256. Recheck identities.
- Preserve the neutral generator files byte-for-byte from production provider
  `3712c7cb784848655050221538a6b9cfa8bb0dd9`, recording their SHA-256 in the
  artifact. Copying benchmark inputs creates no provider production dependency.
- Run direct-native `vize check` and an untimed transparent wrapper. Each
  wrapper obtains native membership and hashes every program member, executes
  the original native arguments, then replays the same live config, cwd and
  arguments with `--extendedDiagnostics`. A second `--listFilesOnly` request
  must reproduce every member/path/byte identity and the native reported Files
  count. Native binary and config hashes must also remain unchanged. This graph
  preread deliberately warms filesystem state. Only complete successful
  instrumentation forwards the original output/status to the CLI; unsupported
  fields or replay mismatch instead leave partial receipts and fail the lane.
- Archive the original and phase stdout/stderr/status, actual shard config,
  full ordered CLI report, semantic options and membership vectors. A strict
  parser separates only a complete terminal numeric footer; malformed,
  incomplete, duplicated or unsupported evidence fails the observation.
  Missing fields are explicit unknowns. No alternate runtime or successful
  fallback report substitutes for a failed phase receipt.

All wrapper child-wall and native phase values are instrumentation observations.
An early wrapper may replay while other original shards are still running,
so even its original child wall has different contention. They are not fresh
CLI timing samples. Parallel child sums and native phases may overlap or
aggregate work; do not subtract them from a separately measured command median.

## Acceptance and freshness

- Direct and wrapped CLI runs must agree on every actual saved Vue virtual TS
  file and shared helper byte, full authored file/diagnostic order, diagnostic
  ownership/counts/severity/messages/positions, exit status and effective
  program membership/options. Minimal and full-corpus planted failures are
  required in both modes; no weakened semantic options or missed plant passes.
- Freeze and fingerprint the complete authored corpus and dependency identity.
  Retain complete public document code, pre-rewrite code, every mapping row,
  subspan, feature/kind and all internal semantic-link rows, plus the public
  mapper DTO. These public default-option views are labeled variants; actual
  saved CLI code is the authoritative batch output.
- The projection probe verifies Unicode-prefix mapping shifts, same-byte-length
  source edits and original restoration at the same path. CLI probes create a
  local error, repair it with an equal-length edit and restored mtime, then
  delete the source; every phase is compared to a new direct-native CLI.

Four populations stay distinct: empty semantic-cache first use; filesystem-warm
fresh CLI without semantic state; filesystem-warm fresh CLI with disk semantic
state; and a live persistent session. This lane measures no disk-cache or live
session gain. Filesystem caches are not evicted, so it does not certify cold disk.

TODO after phase evidence: decide whether a small isolated program-cache probe
is justified. It must preserve all bytes/maps/ordered diagnostics and compare
every local/shared/barrel edit, create/delete, missing-to-present import,
same-length restored-mtime edit, config/extends/alias, package/declaration and
helper/runtime identity change to a new uncached checker. Put any per-shard
build-info outside virtual-tree pruning. A later persistent benchmark must
assert starts/reuses/refreshes/fallbacks; an empty `check_incremental(&[])` call
currently performs a fresh CLI check. Neither an advertised flag nor this phase
observation proves a default speedup, universal gain or 10x feasibility.

No queue entry is authorized during the release freeze. Record exact Actions
source, artifacts, unsupported fields and limitations before proposing any
production change or merging this infrastructure lane.

## Rejected first hosted receipt

[Same-slice correction decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5976715965).

Infrastructure head `a3a856b7d67110c9b25a4da6918d6a2685c0aa44`, with unchanged
production base `ac1675fef35c5637c38b1e366a91d6bdfe419885`, failed the final
generated500/1T full-corpus plant comparison in
[run 37177005703](https://github.com/ubugeeei-prod/vize/actions/runs/37177005703).
The native receipt retained both planted errors, status 1 and unchanged
config/member bytes, but the CLI received a truncated output stream and missed
the planted errors. The wrapper called `process.exit` immediately after buffered
stdout/stderr writes. The correction uses `process.exitCode` and lets both
streams drain. A transport regression checks exact hashes for more than 1 MiB
of Unicode stdout and stderr, with nonzero status, through pipes in both normal
and version forwarding. Every ordered diagnostic comparison remains strict.
The control fails with the original wrapper (65,536 received stdout bytes versus
1,600,000 expected) and passes with the corrected wrapper.
This failed observation establishes no complete phase or performance result.

The rejected raw artifact is
[11293752943](https://github.com/ubugeeei-prod/vize/actions/runs/37177005703/artifacts/11293752943),
SHA-256 `82425ed4d03716094c58622c5b39e60f891a76bb57fd811e3163d3283ac1c52a`.
The same head's Tooling 2/4 check rejected the new 489-line coordinator because
new source files are limited to 350 lines. The correction extracts the CLI
acceptance gates into a separate runner; no limit or instruction ceiling rises.
The restored-mtime gate now starts with an exactly representable integer-second
timestamp and asserts nanosecond equality after the equal-length repair.
Corrected exact-head hosted evidence remains pending.
