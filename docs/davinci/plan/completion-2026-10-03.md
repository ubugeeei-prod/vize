# Native completion ledger (2026-10-03)

Source snapshot: main `bf3b6ee55883dc5bd431e2b184678efc3edaa5c9`, verified
2026-10-03 11:50 UTC. Functionality below is scoped to that immutable source;
open-lane observations are dated separately and do not follow later merges.
The [prior f971 snapshot](https://github.com/ubugeeei-prod/vize/blob/e76ab1574611d1154115863bb3f433f86ddb5cee/docs/davinci/plan/completion-2026-10-03.md)
and [September 30 snapshot](./completion-2026-09-30.md) retain their original
source qualifications and historical receipts. Earlier `05b8b11b` and
`5f66f931` observations are not relabeled as current acceptance.
A merged slice does not close its whole roadmap issue. **Davinci is unfinished.**

## Delivery boundary

Vue Fes Japan is October 24. The delivery scope is Vue templates and every
Vue dialect, JS/TS, and JSX/TSX. TSRX, Solid, Flow and other framework work
retain their later roadmap order. No legacy-backed path receives native
completion credit.

The directory split is merged: levels, guest and dialect infrastructure live
under `davinci/`; Carton, products and transitional adapters stay under
`crates/`. The normal/build dependency direction remains one-way.

The five default product replacements are **0/5 complete**. Compiler,
formatter, linter, type checker and LSP still require their own native
acceptance and fix-history gates. Opt-in functionality below is useful
implementation, not permission to switch a default or delete its legacy path.

## Merged native functionality

| Surface      | Available bounded functionality                                                                                               | Remaining acceptance                                                                                          |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| L0           | Runtime ownership, derive, allocator accounting and dependency enforcement                                                    | Platform/config/i18n/profiler separation and resource criteria                                                |
| L1           | Component/Document lexical custody, original interpolation/For/style observations, bounded Vue 1 and Vue 2 text/filter syntax | Browser tree policies, complete dialect/embedding semantics and historical products                           |
| L2           | Canonical File, original Program facts, selected ordinary HTML body/static-header custody and checked position queries        | Remaining headers/directives, controls, semantic admission and product consumers                              |
| L3           | Bounded original-owner DOM/setup READ decisions and native SSR eligibility in the shared walk                                 | Complete DOM/SSR/Vapor decisions and native control integration                                               |
| Compiler     | Explicit native static/literal DOM and primitive mutable JS setup; original ordinary unscoped CSS output/maps                 | Const/TS setup, semantic/scoped styles, controls, directives, dialects, JSX/TSX, SSR/Vapor and history parity |
| Formatter    | Bounded original Expr-to-Doc and selected interpolations, template/attribute layout and full static/dynamic directive heads   | Remaining syntax/embeds, whole-output properties, native corpus and history parity                            |
| LSP          | Snapshots/cancellation; opt-in JS/TS queries retaining original L1 Program/L2 File in bounded workers                         | Native Vue/JSX/workspace operations, L3/L4 retention, default routing and whole response history              |
| Linter       | Opt-in selected-owner img-alt, iframe-title and static positive-tabindex diagnostics                                          | Classified accessibility headers, other rules/fixes, native corpus and history parity                         |
| Type checker | Genuine whole-Program JS/TS projection and original-Module Canon backend adapter with authored diagnostic links               | Vue/JSX projection, project revision capture, default routing and complete diagnostic history                 |

Representative merged implementation: [allocator #7438](https://github.com/ubugeeei-prod/vize/pull/7438),
[Vue 1 text #7452](https://github.com/ubugeeei-prod/vize/pull/7452),
[File decisions #7453](https://github.com/ubugeeei-prod/vize/pull/7453),
[native DOM #7455](https://github.com/ubugeeei-prod/vize/pull/7455),
[SFC observation ownership #7457](https://github.com/ubugeeei-prod/vize/pull/7457),
[explicit native SFC #7459](https://github.com/ubugeeei-prod/vize/pull/7459),
[project #7466](https://github.com/ubugeeei-prod/vize/pull/7466) and
[cancellation #7467](https://github.com/ubugeeei-prod/vize/pull/7467).
Later actual merges include the original-owner Stack
[#7471](https://github.com/ubugeeei-prod/vize/pull/7471) /
[#7472](https://github.com/ubugeeei-prod/vize/pull/7472),
[native navigation #7476](https://github.com/ubugeeei-prod/vize/pull/7476),
the setup READ prefix
[#7460](https://github.com/ubugeeei-prod/vize/pull/7460) /
[#7461](https://github.com/ubugeeei-prod/vize/pull/7461) /
[#7465](https://github.com/ubugeeei-prod/vize/pull/7465),
[static directives #7482](https://github.com/ubugeeei-prod/vize/pull/7482),
[img-alt #7489](https://github.com/ubugeeei-prod/vize/pull/7489) and
[selected roots #7483](https://github.com/ubugeeei-prod/vize/pull/7483).
GitHub's terminal merge state and current main were checked for each.
These entries do not assert whole-dialect or whole-product parity.

The six earlier deliveries retain their accepted candidate receipts. The
22 further source, fixture, documentation and CI slices below bring this
receipt-tracked set to **28 actual merges**; this is not a count of complete
products or native features. Every newly listed protected Check was verified
as a terminal successful merge-group run at the actual merge SHA.

| Slice                                       | PR                                                       | Actual merge   | Protected Check                                                               |
| ------------------------------------------- | -------------------------------------------------------- | -------------- | ----------------------------------------------------------------------------- |
| Selected original-root L3 DOM decisions     | [#7493](https://github.com/ubugeeei-prod/vize/pull/7493) | `460895474557` | [37109748407](https://github.com/ubugeeei-prod/vize/actions/runs/37109748407) |
| Native primitive mutable JS setup           | [#7497](https://github.com/ubugeeei-prod/vize/pull/7497) | `21f3e560a7c5` | [37110202908](https://github.com/ubugeeei-prod/vize/actions/runs/37110202908) |
| Native dynamic directive formatting         | [#7505](https://github.com/ubugeeei-prod/vize/pull/7505) | `80cc646225e0` | [37110594067](https://github.com/ubugeeei-prod/vize/actions/runs/37110594067) |
| Genuine Vue 2 text/filter syntax            | [#7478](https://github.com/ubugeeei-prod/vize/pull/7478) | `c05e0ee71b60` | [37110894124](https://github.com/ubugeeei-prod/vize/actions/runs/37110894124) |
| Maintainer-selected incremental tier rename | [#7507](https://github.com/ubugeeei-prod/vize/pull/7507) | `0514f1508bd4` | [37111176007](https://github.com/ubugeeei-prod/vize/actions/runs/37111176007) |
| Native selected iframe-title diagnostics    | [#7506](https://github.com/ubugeeei-prod/vize/pull/7506) | `f9718b1b0ed8` | [37111533378](https://github.com/ubugeeei-prod/vize/actions/runs/37111533378) |

| Further delivered slice                        | PR                                                       | Actual merge   | Protected Check                                                               |
| ---------------------------------------------- | -------------------------------------------------------- | -------------- | ----------------------------------------------------------------------------- |
| Selected ordinary HTML bodies                  | [#7511](https://github.com/ubugeeei-prod/vize/pull/7511) | `be9b7be49211` | [37112347462](https://github.com/ubugeeei-prod/vize/actions/runs/37112347462) |
| Full directive heads without arguments         | [#7518](https://github.com/ubugeeei-prod/vize/pull/7518) | `44533d11deed` | [37112538763](https://github.com/ubugeeei-prod/vize/actions/runs/37112538763) |
| Parser facade in Nix dependency builds         | [#7520](https://github.com/ubugeeei-prod/vize/pull/7520) | `9f42382f8bf8` | [37112627881](https://github.com/ubugeeei-prod/vize/actions/runs/37112627881) |
| Original SFC style custody                     | [#7512](https://github.com/ubugeeei-prod/vize/pull/7512) | `f7502080e110` | [37113401709](https://github.com/ubugeeei-prod/vize/actions/runs/37113401709) |
| Native static positive-tabindex diagnostics    | [#7515](https://github.com/ubugeeei-prod/vize/pull/7515) | `b967435f4734` | [37113432095](https://github.com/ubugeeei-prod/vize/actions/runs/37113432095) |
| Vue 2 blank/trailing filter lists              | [#7527](https://github.com/ubugeeei-prod/vize/pull/7527) | `ffb7aaba0cbe` | [37113433493](https://github.com/ubugeeei-prod/vize/actions/runs/37113433493) |
| Original intrinsic For observations            | [#7463](https://github.com/ubugeeei-prod/vize/pull/7463) | `750bed7799e4` | [37113554378](https://github.com/ubugeeei-prod/vize/actions/runs/37113554378) |
| Asynchronous delivery ledger                   | [#7532](https://github.com/ubugeeei-prod/vize/pull/7532) | `e76ab1574611` | [37113900505](https://github.com/ubugeeei-prod/vize/actions/runs/37113900505) |
| Original-Module Canon backend adapter          | [#7526](https://github.com/ubugeeei-prod/vize/pull/7526) | `a9b3c3b1f127` | [37114013982](https://github.com/ubugeeei-prod/vize/actions/runs/37114013982) |
| Selected static HTML L3 ownership laws         | [#7522](https://github.com/ubugeeei-prod/vize/pull/7522) | `2ec78e483f43` | [37114177343](https://github.com/ubugeeei-prod/vize/actions/runs/37114177343) |
| Original File position queries                 | [#7513](https://github.com/ubugeeei-prod/vize/pull/7513) | `69ee3233673f` | [37114303548](https://github.com/ubugeeei-prod/vize/actions/runs/37114303548) |
| Original retained Expr-to-Doc                  | [#7534](https://github.com/ubugeeei-prod/vize/pull/7534) | `e8cd967a5540` | [37114708070](https://github.com/ubugeeei-prod/vize/actions/runs/37114708070) |
| Native SSR eligibility in the shared walk      | [#7509](https://github.com/ubugeeei-prod/vize/pull/7509) | `d6a445eb6291` | [37115724559](https://github.com/ubugeeei-prod/vize/actions/runs/37115724559) |
| Original selected For operands                 | [#7530](https://github.com/ubugeeei-prod/vize/pull/7530) | `6cf392627192` | [37115725501](https://github.com/ubugeeei-prod/vize/actions/runs/37115725501) |
| Original ordinary unscoped CSS output/maps     | [#7528](https://github.com/ubugeeei-prod/vize/pull/7528) | `0fa168a645c3` | [37115726407](https://github.com/ubugeeei-prod/vize/actions/runs/37115726407) |
| Original strict-literal eligibility            | [#7536](https://github.com/ubugeeei-prod/vize/pull/7536) | `6da68f131e79` | [37116283360](https://github.com/ubugeeei-prod/vize/actions/runs/37116283360) |
| Original File navigation worker retention      | [#7545](https://github.com/ubugeeei-prod/vize/pull/7545) | `9a047d759728` | [37116457087](https://github.com/ubugeeei-prod/vize/actions/runs/37116457087) |
| Native selected-template interpolations        | [#7541](https://github.com/ubugeeei-prod/vize/pull/7541) | `5c6f735a7dd8` | [37116673116](https://github.com/ubugeeei-prod/vize/actions/runs/37116673116) |
| Bounded original static HTML headers           | [#7535](https://github.com/ubugeeei-prod/vize/pull/7535) | `73474250649d` | [37117461045](https://github.com/ubugeeei-prod/vize/actions/runs/37117461045) |
| Native Document lexical ownership              | [#7544](https://github.com/ubugeeei-prod/vize/pull/7544) | `8501146f79e0` | [37117555148](https://github.com/ubugeeei-prod/vize/actions/runs/37117555148) |
| Complete import-sorting API history fixtures   | [#7550](https://github.com/ubugeeei-prod/vize/pull/7550) | `e0a3b75670e0` | [37118060547](https://github.com/ubugeeei-prod/vize/actions/runs/37118060547) |
| Complete merge tooling partitions / pinned Pkl | [#7537](https://github.com/ubugeeei-prod/vize/pull/7537) | `bf3b6ee55883` | [37118104098](https://github.com/ubugeeei-prod/vize/actions/runs/37118104098) |

The setup merge has source/lowering/module evidence, but its full-only Node
capture step was skipped in that queue run. The const and SSR lanes must
execute their source-built module/reference laws on a guaranteed queue path;
a skipped runtime step receives no hosted runtime credit. The rename closed
[#7304](https://github.com/ubugeeei-prod/vize/issues/7304); the transitional
crate remains under `crates/` with `publish = false` and its current legacy
dependencies.

The security unblock [#7469](https://github.com/ubugeeei-prod/vize/pull/7469)
actually merged at `2026-10-03T03:21:19Z`.
Its [protected merge-group Check](https://github.com/ubugeeei-prod/vize/actions/runs/37091781490)
passed full workspace/tooling validation and the unchanged 100 instruction
ceilings. Source-head validation and prospective candidates remain separate
from that actual signed merge.

## Parallel implementation lanes

These are implementation targets, not completed features. Publish the lowest
ready authentic provider prefix while unrelated accepted candidates execute.

| Lane                 | Next concrete implementation                                                      | Integration gate                                                                       |
| -------------------- | --------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Original syntax/File | Remaining selected headers, original controls and Document tree semantics         | Genuine provider ancestry; partial/foreign/interrupted owners cannot mint completion   |
| Vue compiler         | Const/TS setup and executed native target families                                | Same-source module/maps, native L2/L3 facts, full runtime oracle and unchanged budgets |
| Vue dialects         | Historical/Document products and complete Vue 2 filter arguments                  | Pinned dialect grammar/runtime; no admission borrowed from another dialect             |
| JS/TS and JSX/TSX    | Older original JSX L2 provider, genuine L3/L4 consumers and remaining TS syntax   | Original AST/source/profile, one existing walk and explicit unsupported boundaries     |
| Formatter            | Remaining original expression/embed families and complete history                 | Whole output, fixed point, parse preservation and native corpus                        |
| Linter               | Original classified accessibility headers and other rules/fixes                   | Whole diagnostics/help/labels/fixes over genuine owners                                |
| Type checker         | Repair original Vue Ref semantics, then Vue/JSX backend acceptance                | JS checked as JS, TS as TS; complete authored diagnostic mapping                       |
| Semantic queries/LSP | Original Vue workers, workspace/external symbols and remaining artifact retention | Whole wire envelopes, source/version identity, cancellation and nonblocking lifetime   |
| SSR / Vapor          | Owner-bound native targets with guaranteed protected runtime capture              | Complete actual modules/maps, typed refusals, pinned runtime and all100                |
| Styles               | Original CSS syntax, scoped output and binding/module/preprocessor semantics      | Same-source CSS/maps, runtime identities and explicit unsupported cases                |
| Incremental          | Rename is merged; complete original artifact retention remains a product task     | Invalidation, concurrency and actual product acceptance                                |

### Active delivery snapshot

Observed on GitHub at 2026-10-03 11:50 UTC; these are open assignments,
not functionality credited to the source snapshot. Later source pushes and
queue changes require fresh checks. Exact-source, manual, queued and actually
merged states remain distinct.

| Owner            | Existing delivery                         | Observed state                                                   | Remaining work                                                          |
| ---------------- | ----------------------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------------- |
| JSX              | #7486 → #7503 → #7519, native Stack #7504 | OPEN; older owning L2 provider has priority                      | Original L3/L4 body/read consumers and native module output             |
| Handler / events | #7510 → #7514, native Stack #7517         | OPEN; parent queued, child prefix queued                         | Older original Handler L1 provider first, then genuine body resolution  |
| SFC setup        | #7521 → #7549, native Stack #7540         | OPEN; Const and TS source prefixes queued                        | Actual native module/maps/runtime capture and accepted protected merges |
| Selected root    | #7511/#7522/#7535 merged; #7566 OPEN      | New native L4 target under source validation                     | Executed actual selected modules; remaining header/DOM semantics        |
| Vue 2 / Document | #7478/#7527/#7544 merged                  | Bounded syntax/lexical owners delivered                          | Full tree, embedding and historical product semantics                   |
| For              | #7463/#7530 merged                        | Original observations/operands delivered                         | Genuine native control consumers                                        |
| Formatter        | #7518/#7534/#7541/#7550 merged            | Bounded syntax plus one history family delivered                 | Remaining syntax, properties and complete #6882                         |
| Linter           | #7515 merged; #7546 → #7552 OPEN          | New classified-header provider/consumer under validation         | Native remaining rules, full diagnostics/fixes and #6881                |
| Styles           | #7512/#7528 merged; #7558 → #7569 OPEN    | Scoped provider/consumer replay and proof repair                 | Guaranteed actual CSS/module/maps/DOM capture; semantic styles          |
| SSR              | #7509 merged; #7523 OPEN                  | L3 provider delivered; L4 target under source validation         | Guaranteed executed same-source SSR modules/maps/runtime                |
| Vapor            | #7551 and target work OPEN                | Original decisions/runtime target under implementation           | Actual pinned runtime capture, whole output/maps and typed refusal      |
| Type checker     | #7526 merged; #7547 OPEN                  | Vue projection dequeued after genuine Ref-unwrapping flaw        | Correct original Vue semantics and fresh native backend proof           |
| Queries / LSP    | #7513/#7545 merged; #7568 OPEN            | JS/TS original workers delivered; Vue ownership under validation | Native Vue/workspace operations and L3/L4 retention                     |
| CI               | #7520/#7537 merged                        | Protected delivery accepted                                      | Whole-gate latency/balance; additional guaranteed native capture paths  |

These owners work in separate `wt` trees. Genuine native Stack membership
and positions were queried on GitHub. A queued parent cannot accept a new
layer: preserve its in-flight candidate, then refresh a remaining consumer
after its actual merge. A parent-base link alone receives no Stack credit.

Older provider ownership remains authoritative: keep the original JSX L2
and Handler L1 sources, then branch their genuine consumers from those heads.
Do not replace an older owner with a newer look-alike implementation or infer
completion from shared-looking type names.

The parser-facade Nix repair [#7520](https://github.com/ubugeeei-prod/vize/pull/7520)
is actually merged. Its separate source `09e8f9e098dd`
[manual Check](https://github.com/ubugeeei-prod/vize/actions/runs/37111603790)
is terminal SUCCESS, including the real Nix build; its protected receipt is
listed above. This is source-qualified manual evidence, not a claim that the
schedule/manual-only Nix lane ran in the merge queue or on current `bf3`.
Earlier Nix failures remain historical failed receipts.

The [CI partition delivery](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-5968669843)
keeps complete full-tooling union, serial files within each isolated runner,
full Rust and unchanged all100 gates. The observed tooling maximum was 5:10
for tests / 6:45 for its job, versus earlier 15:15 / 16:48 at another head.
The genuine original schema/declaration golden law passed with checksum-pinned
Pkl dependencies in a fresh private cache. These different-head observations
are not a controlled benchmark or proof of a whole-gate two-minute target.
Required aggregates remain fail-closed on their existing independent runners;
no prospective census or runner migration receives merged credit.

The audit still finds unresolved native requirements: Document/in-DOM
tree rules, historical dialect products, full script/control/JSX semantics,
native Vapor, correct Vue type projection, remaining LSP retention, and the
five complete fix-history corpora. Ordinary unscoped CSS now has its genuine
bounded output receipt; that does not admit arbitrary `v-bind()`, scoped,
module, preprocessor or external style semantics.

Known source findings remain visible: the new original-owner DOM route needs
Vue whitespace eligibility/normalization; raw lossless Text alone is not
semantic DOM admission. Parser [#7444](https://github.com/ubugeeei-prod/vize/issues/7444)
is closed after its shared parser repair; it is not evidence of additional
native syntax coverage. Conservative refusals keep unsupported feature work
visible rather than closing its roadmap requirement.

## Finish gates and merge operation

1. Land authentic provider APIs before their consumers. Keep move-only commits
   and isolated `wt` worktrees; preserve dirty and active work.
2. For dependencies, branch children from actual parent heads, register a
   GitHub native Stack, and verify its ordered membership. Queue the highest
   contiguous ready prefix; never auto-merge an individual dependent layer.
3. For an independent PR, enable squash auto-merge after its required PR checks
   pass. Protected merge-group full checks remain mandatory. An additional
   manual full run is not a separate queue-entry barrier.
4. Watch every candidate through terminal checks and actual merge. Remove
   known-red candidates; after a prefix merges, refresh remaining children on
   signed current main and rerun their checks.
   Use affected PR checks and protected full/instruction checks for ordinary
   slices. Dispatch an additional manual campaign only to resolve a specific
   missing validation path, such as the schedule/manual-only Nix build.
5. All five fixture gates were verified **OPEN** at this snapshot. Before
   replacing each default, close that product's fixture gate:
   [type checker #6879](https://github.com/ubugeeei-prod/vize/issues/6879),
   [compiler #6880](https://github.com/ubugeeei-prod/vize/issues/6880),
   [linter #6881](https://github.com/ubugeeei-prod/vize/issues/6881),
   [formatter #6882](https://github.com/ubugeeei-prod/vize/issues/6882) and
   [LSP #6883](https://github.com/ubugeeei-prod/vize/issues/6883).
   Require its native-only differential corpus, zero fallback, source maps,
   performance/resource gates and fresh-main evidence.

Local source, a peer review, a compiling skeleton, an open PR and a queue entry
each have their own status. Only terminal accepted implementation and actual
merge count as delivered; whole completion still requires every gate above.
