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
regression must be repaired before source admission; no ceiling, input,
population or comparator is relaxed. An inherited baseline mismatch keeps this
optional collector unqualified and visible; it does not qualify or block a
separately qualified contiguous parent Stack prefix by itself.

Completed child executions retain raw reports and returned process
stdout/stderr/status/signal, exact
source trees and SHAs, before/after executable hashes, Rust/Cargo image resolution,
build receipts, original caps, actual host identity and selected recipe settings.
Runner credentials are excluded. The upload step uses `always()`. Buffer limits,
spawn/wait errors or job/host termination can prevent complete returned streams,
final process receipts or upload; missing evidence stays unqualified. An incomplete
or failed run grants no qualification; local validator success is tooling proof
only. The first requested baseline is actual main
`2703daa2f03f5d943dd6b31089aee4a8c5bc7a9e`.

The first complete [Actions run](https://github.com/ubugeeei-prod/vize/actions/runs/38068714196)
captured all 624 reports for that baseline and candidate
`3c92caeab65bbab04a2e2ebe9ef308e138729939`. All six original level-plane
judgments failed. Each baseline repetition has the same 57 exact allocation
mismatches: 40 increases and 17 decreases against the committed values. The
comparator requires equality, so decreases also fail. Each candidate repetition
has 58 mismatches, including the new `armature_parse_stress-wide` change from
204 to 208 allocations and 114649 to 115673 peak bytes. `armature_parse_large`
changes from 320 to 322 allocations and 115428 to 115684 peak bytes; its original
allocation value is already 272. These two allocation differences propagate
into the existing whole compile rows. The complete reports, all paired metrics
and six failed judgments remain retained; no original100 qualification is
claimed. The four formatter rows remain uncapped observations.

The allocation authority comparison is source-bound: all 85 values recorded at
P1-13 commit `0901ee419c` remain unchanged in the current registry, which adds
15 later rows. The original six ladder input files are byte-identical. The
benchmark parse routine and single normal allocation probe remain the same,
while the product parser and provider sources differ from that August source.
This comparison establishes authority drift without attributing the 57 changes
to any one product modification. Later source requires its own measurement;
the failed candidate, inputs and original allocation authority remain intact.

The earlier twelve-provider/hard104 collector is retained as rejected source-only
preparation: the actual level registry has 100 rows, so that recipe cannot collect
the complete population. Its new source-derived population laws failed before
correction. No native build or hosted measurement ran on that rejected recipe.
The first published collector source retains fourteen genuine lint warnings:
twelve test-registration promises and two missing sort comparators. Explicitly
await registrations and use the same UTF-16 code-unit ordering as the original
default string sort; report population, metrics, judges and all provider bytes
stay unchanged. That failed source remains failed, and the successor requires
fresh qualification.
The first tooling worker also retained the original historical-route law's
failure: its two literal ordinary-job guards predated the new memory mode.
Require the exact additional `!inputs.level_memory_only` exclusion in both
expectations, retaining every original production and historical predicate,
exact-head/ancestry assertion and frozen replay control. The unchanged six
historical laws and twelve collector laws pass locally after this expectation
correction; this is source-law proof, not hosted benchmark qualification.

The collector now composes exactly once with corrected SSR source
`aa89d08dee18d8856892b13cf28974e3ef6b1c71`, retaining all prior collector
commit messages and the six-path lint/law correction. Both full provider and
authority sets remain unchanged, and the paired comparison baseline stays
`2703daa2f03f5d943dd6b31089aee4a8c5bc7a9e`. Fresh source/full/native, the
separate 104-row instruction gate and all 624 normal observations still require
qualification on this actual successor. The original 100-row allocation-equality
REDs remain visible; the independently qualified SSR parent may enter its native
Stack prefix without waiting for this optional memory campaign.
