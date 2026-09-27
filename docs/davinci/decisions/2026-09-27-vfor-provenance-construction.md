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

## Second measured result on actual main

The owner published only the reviewed `56fcfd91a048c7e02693f12fb49f2f920b99920a`
to the existing temporary ref, with a fresh exact `5e402` lease. Its
[measured run 36323473099](https://github.com/ubugeeei-prod/vize/actions/runs/36323473099)
fails seventeen unchanged ceilings. All 100 probes have three equal observations;
their input identities, windows, protocol and guest context match the preserved
baseline. Artifact `10933710975` preserves this failed measurement.

The iteration probe is now `147642 <= 148047`, Patina JSX is `43809 <= 45385`,
and emit p2_11 is `195261 <= 197851`. These individual improvements do not accept
the complete candidate: Patina S2 is `543205 > 539107`, with parse, DOM and SSR
probes also exceeding their original ceilings. No retry, raised cap, changed
fixture or public-eight replay follows from these failures.

[Full Check 36323470941](https://github.com/ubugeeei-prod/vize/actions/runs/36323470941)
passes Clippy, both no-std lanes and the exact allocation gate. Its nine raw
reports confirm iteration allocations `11` and Linux peak `1499`, JSX `0` and
Patina S2 `7`; artifact `10933286266` has SHA-256
`11701b762be9162dbc5a50557dd600e2e17152d496b291cd7600456ca294ff1b`.
MacOS current-source evidence remains unknown, with strict `1495` unchanged.
The primary Rust Test and instrumented source-coverage jobs are still running;
the twelve new controls and full eleven-feature differential remain pending.
Strict JavaScript checking fails a Markdown table's formatting in the binding
cost companion; the mechanical spacing fix is prepared privately without
changing or restarting the frozen source. Full Check is not accepted.

Exclusive function attribution identifies outlined expression safety fallback
work and Patina name interning as the next bounded production investigations.
Any guard integration or single-lookup intern change needs concrete source
review and complete semantic controls before another measured candidate.
The original measurements and source refs remain preserved; #6831 and #6832
remain open, with no numeric, native, queue or main acceptance claimed.

The same frozen source's instrumented Source coverage job `108631662393` later
completed successfully. Its raw `cargo llvm-cov --workspace` log explicitly
executes each of the twelve named controls successfully, plus
`stress_deep_lowers_and_tears_down_on_a_small_stack`. The log's SHA-256 is
`3b9b0c45b86a8a8501e581c8e942b18e1930351447776e7b70c732848c676af9`.
This is actual typed default-workspace evidence for the added controls, not
eleven-feature differential acceptance. The primary Rust Test remains running;
the failed instruction ceilings and Markdown formatting failure remain intact.

The primary Rust job `108631662424` subsequently completes successfully on the
same frozen source. Its workspace run executes all twelve controls and the
small-stack test; the unchanged eleven feature-enabled recipes also complete.
Their feature phase records 117 passes, zero failures and one existing ignored
Vitrine typecheck doctest. The first lowering/DOM/Pug/production recipes retain
their existing committed-battery/fixture scope when the external corpus variable
is unset; SSR and Patina retain their external-corpus environment. This accepts
the actual recipe's scope, without claiming every repository file was admitted.
The raw Rust log has SHA-256
`707e1a2bf1a33223c4dbf49dc48a7810de0ff4170fe6e85126f4cf03cb1cedb6`.
Full Check still fails Markdown formatting, and the seventeen instruction
ceilings still fail. No public replay, new dispatch or native acceptance follows.

## Reviewed third private composition

The root concretely reviewed three bounded production edits: one ordinary
`#[inline]` on the unchanged extracted safety predicate, single Entry lookup
in `Facts::intern`, and a combined newline/first-indent prefix through the
existing buffer push operation. Their linked companion records preserve the
exact attribution and semantic scope. No unrelated source relocation, forced
inline, benchmark edit or raised ceiling joins the composition.

The Entry change keeps the existing standard FxHashMap, hasher and exact keys;
five complete logical-state controls cover all 146 committed identities,
namespace/case/order, the 256 limit and exact overflow defects/member bits.
Vacant Entry may reserve before overflow rejection. No logical insertion
occurs, but unchanged allocation/capacity cost is not established by that claim.
The previous twelve controls passed only on frozen `56fc`; all seventeen and
the unchanged feature recipes require fresh execution on this next source.

After composing the three reviewed edits, the owner author-preservingly merged
exact actual main `ae3c16b3b94354e623b5221100bb90cf3a710571` once as
`2a0c59b1b5e6cab042cbc6055ec470dcbe47a0b2`, retaining both parents and the
original source/failure ancestry. This freezes the next verification parent;
later main advances are not chased. The only conflict was central-record EOF:
main's complete Options API computed pointer stays, while the existing binding
cost bullet moves verbatim into Performance. Main production has no conflict.

Official consumer regeneration changes just the linter loader import line
`3 -> 5` for the Entry import. Its nineteen-file check and the unchanged
twenty-four-file v-on check pass; storage and measured-v-on fixtures need no
new row change. Thirty existing inventory/budget tests pass with zero skipped
under pinned Node24.14.0. Source-length comparison against this actual main
passes; the existing main words and destinations remain in order.

TODO: review the complete producer/source-preservation packet before one fresh
leased temporary-ref update. Dispatch the unchanged measured instruction
workflow first. Only after all 100 targets pass their original ceilings in
three executions, dispatch unchanged Full Check once for all seventeen controls,
exact allocations, metadata and the complete feature recipe scope. If numeric
ceilings fail, preserve and attribute that source's artifacts before another
reviewed repair; do not occupy another full-validation wave for a rejected
numeric candidate. This coordination changes no workflow, benchmark method,
fixture, window or ceiling and cancels no existing run.

The instruction workflow compiles the actual producers and executes its existing
two harness-library suites; that does not execute the five new Patina controls.
Their typed runtime, the previous twelve on this new source, full feature scope
and exact allocations remain pending until Full Check. No measured result alone
accepts those semantic requirements. MacOS stays unknown with strict `1495`
preserved. Existing first-member coordination and the held seven are separate;
no public restack or native acceptance follows.

## Third actual measured result

The reviewed frozen source `444ff3460aec443bbe9e83f700e3c142d5e7c1f0`, tree
`d4aa1e0a2711fa2a403a2d0f179a5c70c0aa4532`, was published with the exact
fresh `56fc` lease only to the existing temporary verification ref.
[Measured run 36326522402](https://github.com/ubugeeei-prod/vize/actions/runs/36326522402),
attempt one, job `108640237391`, actually fails seventeen unchanged ceilings.
Independent raw 100-by-three verification accepts equal observations, exact
inputs, windows, method, eight guest variables and the immutable-base ratchet.
Artifact `10934182096` has SHA-256
`992ad0cf1ef7b323e30f9ee4d496a23341ce734ec7ff0ef98055e6b63ac7c052`.
Ceilings retain SHA-256
`17c3947044c033d03b0a86d13b2c3fa6dc2d1e90cb2222033921f649e97a826f`.

V-for `147554 <= 148047`, Patina JSX `43798 <= 45385`, p2_11
`192179 <= 197851` and deep DOM compilation meet their original ceilings.
The complete source remains rejected: parse-medium `245614 > 243340`, Patina
S2 `542626 > 539107` and two-per-bucket v-on emission `46310 > 46119`,
with fourteen other parse, transform, DOM, Vapor and SSR failures preserved.
Full Check was not dispatched. All seventeen authored controls, unchanged
feature recipes and exact allocations on this source remain unknown; previous
`56fc` runtime evidence does not transfer. MacOS strict `1495` stays unknown.

TODO: attribute this exact failure and concretely review bounded production
work reductions before any further source/ref update. No numeric retry, cap
increase, changed fixture, public tail replay or native credit is granted.
The original source and raw failures remain preserved. This later private
fact-only record changes no measured producer or remote verification head.

## Reviewed fourth private composition

The root concretely approved two further work reductions on frozen `444`:
the [short prefix-operator bound](./2026-09-27-short-prefix-operator-guard.md)
and [first-row fact capacity](./2026-09-27-patina-facts-first-row-capacity.md).
For at most 31 UTF-8 bytes, the original prefix run cannot exceed its existing
31 limit; only that predicate skips work. Longer fallback, complete balancing,
numeric limits, OXC admission and stack safety retain their original bodies.
The table estimate counts first-four-column-row direct member tokens once,
excludes text/category references and caps at 256. Malformed rows do not seed;
zero-direct-token rows reserve zero. Original streaming members, keys, IDs,
case order, rows, children and defect order stay unchanged.

The Facts ID and children maps are private. Production accesses them by key;
there is no production map-order iteration or serialization consumer in the
read source scope. Public universe/name and case fallback iterate the names
and cased vectors in their preserved insertion order. Map iteration/capacity
identity is not promised, and duplicate/invalid tokens may overreserve.
Prescan work and actual allocation/instruction savings remain unmeasured.

The previous seventeen controls stay intact. Six new complete prefix/safety
controls and four new complete table-state/defect controls bring the pending
set to twenty-seven; no new control has runtime acceptance on this composition.
The unchanged full legacy/linter/DOM differential recipe scope and exact
allocation gates are required after all 100 unchanged numeric targets pass.
MacOS strict `1495` stays unknown. The parallel empty dynamic-props allocation
candidate is not included; no public tail head changes.

TODO: complete official inventories and exact-source/foreign-main/raw-method
proof, then root final-tree review before one fresh `444` lease update of the
owned temporary ref. Dispatch unchanged measured workflow once first; preserve
any failure without a Full Check wave. Only all-100 success permits one unchanged
Full Check for the twenty-seven controls, full feature scope and allocations.

The next verification now imports exact actual main
`754c0acd2c9246a534ad43f22cfc7dd5b6960e1a` once, retaining original authors
and failed-source ancestry. This preserves the subsequently merged cache,
Canon and Slot source/oracles/matrices before any numeric or full validation.
Every foreign source blob must equal that main outside the narrowly reviewed
owned ranges; exact helpers, source selection and measurement contract remain
required. Integration and final-tree proof precede the authorized ref update.
