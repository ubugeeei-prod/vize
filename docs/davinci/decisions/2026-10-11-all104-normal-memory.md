# Complete normal-mode memory comparison

Issue: [#8504](https://github.com/ubugeeei-prod/vize/issues/8504).

The SSR literal-attribute repair changes private parser storage. Local debug
measurements show `CurrentElement` 56 → 128 bytes, `Parser` 944 → 1040 bytes and
`TransformContext` 704 → 720 bytes. The compact end-only representation retains
the existing eight-entry inline capacity; nine directives spill one 128-byte
allocation. These local observations do not establish optimized production cost
or a result for all 104 workloads. Acceptance requires the complete source-built
matrix on Actions, alongside the unchanged instruction and correctness gates.

The opt-in `level_memory_only` mode of Criterion Bench builds the existing twelve
level-instruction suite providers and the existing formatter-instruction provider
from an exact baseline and candidate, using
isolated Cargo targets, repository Rust 1.99.0, `ci-opt` and
`-C target-cpu=x86-64`. Candidate SHA must equal the actual dispatch SHA; baseline
must be a distinct ancestor. Other Criterion modes cannot be combined with this
mode. The existing 45-minute reference-runner ceiling remains unchanged.

One real Callgrind pass for each binary authenticates its original benchmark IDs,
input-identity-v2 digests and routine/stage-return windows. Actual named dumps,
raw identity streams and allocator initialization are retained. This pass is
identity evidence. The separate three-run instruction workflow remains the
instruction qualification authority, with its original ceilings unchanged.

Both sides then run the same ordered thirteen providers with `--bench --quick` three times,
alternating baseline and candidate repetitions. The fixed guest environment removes
`VIZE_INSTRUCTION_COUNTS`, retains the existing mimalloc reservation/purge and
libc-dispatch options, and uses identical probe paths and argv0. Normal mode does
not perform the instruction-only 64 MiB allocator preinitialization. The existing
counting allocator, RSS process baseline, stage windows and fixture providers
remain unchanged. Quick wall samples do not support a speedup claim.
Pinned Criterion 0.8.2 requires `--bench` to select benchmark sampling; omitting
it selects test mode and cannot qualify this recipe.

Each admitted report requires its exact write marker from the completed suite
process. Preexisting tracked reports are retained separately and cannot qualify
without that invocation's write. Missing, extra, duplicate, foreign, symlinked,
null or forged reports fail. The complete population is the disjoint union of
the original 100 level rows in `budgets.toml` and the original four rows in
`formatter-instruction-registry.toml`. Both original registry files and both
instruction-budget files must remain byte exact. The four formatter instruction
ceilings authenticate their existing input/window identities; they do not supply
allocation ceilings. The whole original104 population, actual input and window
identity, platform and harness version must agree between both sides. Every
paired allocation, peak-byte, RSS and wall p50/p95 value and numeric difference
is retained, including all four formatter rows.

All 624 normal-mode reports are collected before budget judgment. The original
`bench-compare.rs` judges each complete original100 level plane against its
unchanged allocation, platform peak-byte and conditional wall controls. It never
receives formatter rows. The four original formatter rows are separate paired
observations with no allocation ceilings or allocation-admission claim. RSS is
reported for every row; this collector adds no RSS threshold. A failed baseline is retained
as a baseline failure, and does not prove candidate causality. A genuine candidate
breach holds admission; no ceiling, input, population or comparator is relaxed.

Completed child executions retain raw reports and returned process
stdout/stderr/status/signal, exact
source trees and SHAs, before/after executable hashes, Rust/Cargo image resolution,
build receipts, original caps, actual host identity and selected recipe settings.
Runner credentials are excluded. The upload step uses `always()`. Buffer limits,
spawn/wait errors or job/host termination can prevent complete returned streams,
final process receipts or upload; missing evidence stays unqualified. An incomplete
or failed run grants no qualification; local validator success is tooling proof
only. The first requested baseline is actual main
`2703daa2f03f5d943dd6b31089aee4a8c5bc7a9e`; hosted memory results remain pending.

The earlier twelve-provider/hard104 collector is retained as rejected source-only
preparation: the actual level registry has 100 rows, so that recipe cannot collect
the complete population. Its new source-derived population laws failed before
correction. No native build or hosted measurement ran on that rejected recipe.
