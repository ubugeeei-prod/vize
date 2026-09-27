# Fixed v-for provenance construction (#6868)

The initial private source candidate was based on
`b3e3f944a11d2f28cfc928eac980503fae866e7f`, which includes the separately
reviewed Patina empty-binding optimization. It changes only fixed provenance
construction in L1-to-L2 lowering. It does not add the proposed ASCII expression
classifier or change any stack guard.

## Decision and scope

`lower/cx/for_parts.rs` builds `fact value=... key=... index=...` using
`vize_l0::String::with_capacity` and `push_str`. Capacity is the exact sum of
the UTF-8 byte lengths of all literals and `spell()` values. Pending `?`, absent
`-`, name bytes, provenance ownership/order/node/span, and fact-map insertion
remain unchanged. Numeric `scope #N bindings=M` formatting remains unchanged.

`lower/forop.rs` applies the same construction to `ui.for source=... value=...`.
Both existing `desc()` calls remain, including their opaque/foreign spellings.
This second bounded site reduces more real lowering work without shifting it
outside the measured window. Neither change adds a pipeline stage or serialization.

The frozen #6906 failure is `150340` in all three executions versus cap
`148047`. Its complete `+2293` is under the first `pthread_getattr_np` stack-limit
query: one extra `/proc/self/maps` line, with `sscanf` and `getline` calls
`56 -> 57`. Vize/OXC self costs and call counts are unchanged. The additional
mapped object is unknown because maps/ELF snapshots were not archived. Root/shallow
stack guards stay intact; warming them outside the window is not part of this change.

Frozen formatting costs are `2004` inclusive Ir for two `attach_for_parts`
formatting calls and `730` for the `lower_for` formatting call. Only the fixed
text calls change. These costs are not savings estimates: whether this candidate
recovers the excess is unknown until the unchanged Actions measurement runs.

## Local source evidence

- `rustfmt --edition 2024 --check` passes for both production files and the new
  `tests/vfor_provenance.rs` integration test.
- The actual assertion scanner reports zero findings in the new test, with no
  allowlist change. Eight authored controls compare complete three-record
  provenance vectors: named positions, destructuring, absent positions, Unicode,
  long ASCII/Unicode ownership, and pending key positions. A two-scope control
  compares the complete six-record vector, including page order and every span.
- Both production edits invert byte-exactly to the base outside their fixed text
  construction and necessary imports. All touched source/docs files stay below
  350 lines.
- All 35 protected raw paths match the base, including every benchmark source,
  both verification workflows, driver/library/harness, both budget registries,
  expression guard, and stack guard. The instruction budget SHA-256 remains
  `17c3947044c033d03b0a86d13b2c3fa6dc2d1e90cb2222033921f649e97a826f`.
- The 100 IDs, raw input digests, measured windows, protocol and caps are unchanged.
  No benchmark source, fixture, baseline, ceiling, counter ID or workflow was edited.

No local Cargo build, Rust test execution, dependency install, publication or
workflow dispatch was performed. Rust runtime correctness and numeric acceptance
remain pending. Source-authored expectations are not regenerated runtime captures.

## Required Actions evidence

After root review, the owner may verify one temporary ref with unchanged
`check.yml` (full Rust tests and differential corpus) and
`level-instruction-counts.yml` (actual 100 probes, three executions each).
The new provenance controls, existing v-for/loop snapshots and small-stack
recursion control must pass. All 100 probes must satisfy existing ceilings.
No numeric acceptance, public-eight restack or gate exception follows from these
local checks. Further optimization, if needed, requires a separate reviewed change.

The source candidate is `01dff5012`. Its paired [#6868 issue record](https://github.com/ubugeeei-prod/vize/issues/6868#issuecomment-5856053395)
states the private/unpushed scope and pending runtime/numeric evidence. The central
Performance section links this companion.

## First composed measurement and allocation ratchet

The owner published only temporary verification ref
`verify/native6962-physical-l3-gates-20260927`, source
`5e40274da99c9a25cd0b6c11649d57debf31ec45`; no new PR was created.
[Measured run 36320906189](https://github.com/ubugeeei-prod/vize/actions/runs/36320906189)
failed 13 unchanged instruction ceilings. Patina's two probes now meet the
original ceilings, but the iteration probe is `149089 > 148047`: 1251 fewer
instructions than the frozen physical L3 failure, still 1042 over budget.
Other emitter regressions require their separately reviewed producer repair.
The raw 100-by-three reports, input identities and failed results are preserved.

[Check 36320903163](https://github.com/ubugeeei-prod/vize/actions/runs/36320903163)
passed strict Clippy and both nine-test no-std lanes. Its allocation gate
rejected a measured improvement: allocations `12 -> 11` and Linux peak bytes
`1511 -> 1499`. The subsequent primary workspace test step was skipped. Its independent
Source coverage job `108624411411` later passed `cargo llvm-cov --workspace`
and explicitly executed all four new Patina and two new provenance controls
successfully on exact `5e402`. This instrumented runtime evidence does not
provide the unrun eleven-feature differential corpus or numeric acceptance.
The tooling check also found
a stale generated linter consumer shard; regenerate it with the official
inventory command after the final source candidate is composed.

`bench-compare.rs` compares allocation counts and each registered platform's
peak for exact equality, including improvements. The private downward ratchet
therefore changes only this probe's allocation count to `11` and measured
Linux peak to `1499`, plus their existing tooling expectations. It preserves
wall/report-only policy, equality comparisons and every instruction ceiling.
The preserved allocation report is artifact `10932132615`; its iteration JSON
has SHA-256 `b2a561c2d47193eb5909d316114118a7efb935b17f24a36dc6ccbd92aa5a8869`.

The final composed source must confirm those exact Linux results again.
MacOS remains unmeasured for this candidate: keep its existing `1495` strict
peak, mark its current-candidate evidence unknown, and obtain actual matching
MacOS evidence before any later downward update. Do not infer a new platform
value from Linux, loosen equality into an upper bound, prewarm stack lookup,
change the measured window, or claim numeric/native acceptance from this draft.

A read-only workflow audit found no current MacOS allocation runner for this
probe. The allocation gate in `check.yml` is Linux-only. The existing MacOS
lanes in `davinci-incremental.yml` run artifact/key tests, while
`native-smoke.yml` and `fresco.yml` build or smoke-test host products; none
executes `davinci_storage` with its allocation report. Criterion, Check Bench
and Tool Benchmark use Linux reference runners. A future explicit MacOS
measurement route must record the exact source, unchanged probe and raw report;
this private draft neither adds a workflow nor grants MacOS acceptance.

## Reviewed private composition and source inventories

The next private composition retains the reviewed fixed provenance edits and
adds the reviewed [constant indentation chunks](./2026-09-27-indentation-chunks.md)
and [complete ASCII word safety path](./2026-09-27-ascii-expression-safety.md).
The original expression predicate is extracted before its bounded optimization;
OXC admission, its scanner/operator fallback and stack protection stay intact.
All twelve new named controls require actual execution on this final source:
four Patina, two provenance, two indentation and four expression guard/admission.
The earlier six-control result does not accept these added changes.

The official Rust commands `consumer-migration-surfaces.rs --write` and
`v-on-corpus.rs --write` update only the linter consumer shard and Patina v-on
shard. Their matching `--check` commands pass. The new test module shifts three
existing import-line references by one and adds two test/dev import rows;
its authored `@click.stop` adds one natural corpus occurrence. These are source
inventory observations, not runtime captures or instruction baseline changes.

The storage policy has no write generator: its existing `scanStorage` reports
`lower/cx/for_parts.rs` L0 String direct/bound counts `1/1`, and
`lower/forop.rs` bound uses `7 -> 8` with other counts unchanged. Only these two
reviewed rows are updated; no allocation-Vec category, opaque/std allowance or
storage rule changes. Existing source inventory tests verify the resulting
artifacts and measured-v-on evidence. Final-source Actions and unchanged queue
ceilings remain pending; no public-eight replay follows from local metadata.

## Actual merged global-fix parent

All seven existing global/linter/LSP/Vue changes actually merged at
`2026-09-27T13:33:22Z`. Their frozen main receipt is
`c681035734fa84c223ee77395ea26bdd07297fb2`, tree
`fd9c7c5952df569fdf1b8017deb5a930d6bcae1f`. The owner merged that exact main
into private `850c72326a3723fa22dcaf46918632056b3be788`, preserving both
parents in merge `b7b98c197cc95234dc3107431af2e21203466bd7`.
No unrelated advancing source is chased before this verification.

The shared Git tracked-file helper, its nine-control test and its two original
consumers are byte-identical to actual main; no duplicate buffer fix is added.
Main's source-selection/LSP build workflow is inherited. The only Check workflow
deltas are the already reviewed dependency-direction job/required need, physical
L3 portability package identity and existing comment line compensation.
Check remains 690 lines; the central record is 294, retains all main and owned
pointers, and needs no global reflow. Official consumer/v-on checks and thirty
existing inventory/budget controls pass under pinned Node24.14.0 after the merge.
All original main words/links and the approved producer semantics are checked
before one leased verification-ref update and two unchanged Actions dispatches.
The twelve controls, full differential features, current Linux allocations and
100-by-three instruction acceptance remain pending on this exact composition.
The public eight are unchanged and dequeued; no main/queue acceptance is granted.
