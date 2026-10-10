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

The corrected normal-mode campaign at source `295289873e982e58845c8ec628d6f4f5e6e77d1b` ([Actions run 38078816799](https://github.com/ubugeeei-prod/vize/actions/runs/38078816799)) retained all 624 fresh reports, 131 completed process receipts and 262 whole raw streams. Every one of the original 104 paired allocation counts and peak allocation-byte measurements equals baseline `2703daa2f03f5d943dd6b31089aee4a8c5bc7a9e` in each of three repetitions. The seven candidate-only increments recorded for the earlier `3c92` campaign are absent; the unchanged baseline counters match that earlier baseline for all 104 cases. Both sides still fail exactly the same 57 original 100-row allocation equality laws (40 increases and 17 decreases against the normative constants), in every repetition. The four formatter rows remain report-only without allocation ceilings. RSS and quick wall samples differ and are retained without an improvement or causal claim. This campaign remains unqualified and transfers no source, instruction, runtime or memory qualification to parent `504` or a later composition; the parent's own qualification remains independent. Parser +24 B and Core context +16 B measured layout costs remain reported. The next required child refresh follows the parent's actual merge and reruns its own exact-source gates.

After the actual signed SSR merge `ceb3677dfc29b9b6a9c5751771b8f01cb0969c52`, GitHub native Stack 8527 retargeted this child to `main` and replayed its source as `e39085cb9c66f5fe06d2436797a953abfa2a4db2`. The complete incoming foreign tree and seven tooling, law and workflow files are unchanged. The two factual record clauses from the old `295` merge resolution are restored before this new campaign record. This actual-main successor still needs fresh source/full/native, separate 104-row instruction and complete 624-report normal qualification; old results provide no acceptance transfer.

## Authenticate the released harness version on each side

The exact `38206df1` campaign stopped at its first complete 104-row base/head
pair because the collector required the same raw harness version. The fixed
`2703` baseline reports `0.441.0`; the `382` candidate reports `0.442.0`. Both
versions match their pinned whole workspace manifest and actual Cargo harness
library package IDs. The complete retained artifact contains 208 raw reports,
26 normal child outcomes and 71 complete process receipts. Its metadata says
observations are incomplete, qualification is false, and no original budget
judge ran. This result supplies no 624-report or original 100-row acceptance.

The corrective collector uses the already resolved pinned Cargo image to retain
`cargo metadata --locked --no-deps --format-version 1` for each arm. Before Cargo
runs, it joins the whole workspace manifest, harness manifest and harness library
source bytes to their exact mode and Git blob in the existing retained source
listing. After the build, those three bodies must remain equal before their exact
workspace, package ID, manifest, library artifact and version can issue a source
witness. The witness retains both SHA-256 and Git blob identities. Each later
report admission rechecks those source bodies and matches the raw version and
report path to its own source arm. These are before/after observations; they do
not provide compiler-open-time telemetry or detect a transient source change
that is restored between observations.

An explicitly supplied pair of issued witnesses permits different declared
source versions. A copied, missing, foreign, swapped or report-mismatched witness
cannot qualify. The original comparison without witnesses still requires
identical versions. Every completed suite's raw report copies and row receipt
are saved before the additional own-source version assertion. The genuine
`382` failure remains retained; neither the missing early data from a canceled
child nor the original schema/allocator refusal boundary is claimed to change.

The finite change adds no product provider, parser, stage, manifest edit or
budget waiver. All other input/window/platform identities remain exact. All
624 reports must precede the six untouched 100-row judges; the four formatter
rows remain uncapped. The original `382` failure and all prior complete 624-report
REDs remain retained. First-pair allocation and peak-allocation differences are
zero for all 104 inputs, with no claim for missing repetitions, RSS, wall time,
causality or the 57 previously observed allocation-equality failures. The new
source laws use synthetic metadata and a synthetic completed probe; their 23
passing source tests are not a native build or hosted benchmark qualification.
The untouched Rust comparator also independently requires equal raw harness
versions (`bench-compare.rs`, `assert_comparable_identity`). The source witness
changes only collector admission: a complete next campaign still executes all
six original judges, and the candidate judges will retain that version refusal.
No normalized report copies or comparator exception are introduced. Fresh
exact-source Actions are required; the optional child remains off queue.

## Coalesced source-owned judge boundary

The preceding collector-only record describes immutable correction `e726`.
The next coalesced successor adds one private optional Rust argument,
`--source-harness-pair`, after all 624 observations. The default two-directory
comparison still requires equal raw harness versions, with its original exact
identity output and all numerical verdicts preserved. The opt-in argument
refuses subset benches and baseline refresh. It never normalizes report bytes
or changes the original 100-row allocation, platform peak or conditional wall
judgments; the four formatter rows remain report-only.

A validated pair requires complete source/build/report custody for both arms.
Its source roles bind the actual inherited workspace version to one retained
pinned Cargo metadata result, a unique library artifact and the pre-build Git
blobs/source hashes. Completed process outcomes and full streams, source-built
image hashes, canonical source/report paths, all 104 identities in three runs
per arm and exact whole raw/copy bodies are mandatory. Each suite retains the
original normal argv, allocator/environment and actual emitted-file markers.
The six original numerical comparisons use the complete 100-row level plane;
a missing, extra, foreign or formatter row refuses before metric judgments.
Every judgment retains its original base/run-N baseline operand and the same
repetition N for the current base-or-head operand. Each side requires its own
source version. A run-1-only carrier was rejected by the new source law before
publication; no original numerical comparison input changes.

Different versions alone do not establish comparability. Both complete harness
and test-support source trees, all 13 original provider bodies and the source
suite registry must be identical, including complete path sets and modes.
Harness manifests, inherited non-version package metadata, build profile,
compiler/Cargo images, toolchain and all external lock records must match.
Only four named dev-product workspace dependency release-version fields may
follow their respective authenticated workspace versions; paths, features and
all other declarations remain exact. Reporter, scenario, allocator, schema,
provider, added/removed path or dependency drift refuses. These checks retain
the same before/after limitation: they are not compiler-open-time attestation
and do not rule out transient restored mutation between observations.

The new controls use explicitly synthetic source/process/report packets and
provide no native execution or performance credit. The original default
identity and numerical fixtures remain unchanged. Whole source review and
fresh Actions must exercise the source-owned judge before at most one final
paired 624-report campaign. Original `382` partial208/version RED, `295`
complete624 RED and earlier outcomes remain retained. The 57 original equality
failures are still debt, with no cap/baseline waiver or acceptance claim. This
optional child stays off queue while unqualified and does not hold releases.

The first frozen judge net `09597707` remains retained as a rejected source snapshot. Independent review found three concrete joins missing: inherited release-profile settings, the absolute resolved Cargo/Rustc image and metadata/build image equality, and the exact authored `=ownVersion` requirement for the four dev products. The corrective net binds the complete authored profile table, both actual pinned build executables and identical metadata/build Cargo images, and only those exact version requirements. Synthetic source laws preserve the authored requirement shape in both arms and refuse inherited-profile, stable foreign-image and unprefixed/foreign-version substitutions. No native execution, numerical acceptance or performance credit is inferred from this source correction.

## Source-file responsibility correction

Published `d03f4183` exercised all eight new source-owned Rust control groups
successfully in PR tooling group 4/4 on its actual `02dffeba` projection. Its
original check-js formatting and 350-line source guard failures remain retained.
The corrective successor removes two extra documentation blank lines and moves
existing witness, synthetic-fixture and source-control bodies into modules at
their existing responsibility boundaries. All 18 Node controls and eight Rust
source-control bodies are preserved; the original default identity file is
restored byte-for-byte. Whole-body inverses retain the original collector
recipe, provider inputs, numerical judgments and source ownership checks.

All split JS/TS files have at most 329 lines. The existing 1,180-line Rust
helper remains unchanged within the guard's original `tools/support/` boundary;
no guard exclusion or limit is added. Fresh Actions must resolve every original
required gate. Passing source laws does not resolve the original 57 allocation
equality failures. No knowingly unqualified 624-report campaign is dispatched,
and the optional memory PR remains off queue without holding releases.
