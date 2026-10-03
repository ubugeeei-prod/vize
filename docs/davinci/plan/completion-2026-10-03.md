# Native completion ledger (2026-10-03)

Verified baseline: main `f9718b1b0ed8ea65cd3f8a4dff7a0ba11d10d76c`
(2026-10-03 09:21 UTC). Earlier snapshots used `05b8b11b` and `5f66f931`.
This supersedes the current-work sections of the
[September 30 snapshot](./completion-2026-09-30.md); its historical receipts
remain valid for their original commits. A merged slice does not close its
whole roadmap issue.

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

| Surface      | Available bounded functionality                                                                                                   | Remaining acceptance                                                                    |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| L0           | Runtime ownership, derive, allocator accounting and dependency enforcement                                                        | Platform/config/i18n/profiler separation, resource criteria                             |
| L1           | Container admission, native component syntax, retained Program/embedding observations, bounded Vue 1 and Vue 2 text/filter syntax | Complete dialect and embedding semantics                                                |
| L2           | Canonical File, original Program declarations/references, selected text/comment roots and checked source coordinates              | Full syntax, original Element/body ownership, controls and product consumers            |
| L3           | DOM decisions and setup READ facts over bounded genuine File artifacts                                                            | Complete shared DOM/SSR/Vapor decisions and native control integration                  |
| Compiler     | Explicit native SFC entry for static/literal DOM and primitive mutable JS setup declarations                                      | Const/TS setup, controls, bindings, all dialects, JSX/TSX, SSR/Vapor and history parity |
| Formatter    | Native Doc construction, plain template/attribute layout and checked static/dynamic directive heads                               | Remaining directive and embed coverage, style/corpus properties and history parity      |
| LSP          | Snapshots, cancellation, shared project APIs and opt-in original JS/TS definition/reference queries                               | Complete artifact retention, Vue/workspace IDE operations and response parity           |
| Linter       | Opt-in native selected-owner img-alt and iframe-title diagnostics                                                                 | Other syntax/semantic rules, fixes, native corpus and history parity                    |
| Type checker | Genuine L4 whole-Program JS/TS projection with original source links                                                              | Canon adapter, Vue/JSX projection, native diagnostics and history parity                |

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
[native navigation #7475](https://github.com/ubugeeei-prod/vize/pull/7475),
the setup READ prefix
[#7460](https://github.com/ubugeeei-prod/vize/pull/7460) /
[#7461](https://github.com/ubugeeei-prod/vize/pull/7461) /
[#7465](https://github.com/ubugeeei-prod/vize/pull/7465),
[static directives #7482](https://github.com/ubugeeei-prod/vize/pull/7482),
[img-alt #7489](https://github.com/ubugeeei-prod/vize/pull/7489) and
[selected roots #7483](https://github.com/ubugeeei-prod/vize/pull/7483).
GitHub's terminal merge state and current main were checked for each.
These entries do not assert whole-dialect or whole-product parity.

The following new slices have terminal GitHub merges on this baseline. The
listed protected Check ran against the actual merge candidate.

| Slice                                       | PR                                                       | Actual merge   | Protected Check                                                               |
| ------------------------------------------- | -------------------------------------------------------- | -------------- | ----------------------------------------------------------------------------- |
| Selected original-root L3 DOM decisions     | [#7493](https://github.com/ubugeeei-prod/vize/pull/7493) | `460895474557` | [37109748407](https://github.com/ubugeeei-prod/vize/actions/runs/37109748407) |
| Native primitive mutable JS setup           | [#7497](https://github.com/ubugeeei-prod/vize/pull/7497) | `21f3e560a7c5` | [37110202908](https://github.com/ubugeeei-prod/vize/actions/runs/37110202908) |
| Native dynamic directive formatting         | [#7505](https://github.com/ubugeeei-prod/vize/pull/7505) | `80cc646225e0` | [37110594067](https://github.com/ubugeeei-prod/vize/actions/runs/37110594067) |
| Genuine Vue 2 text/filter syntax            | [#7478](https://github.com/ubugeeei-prod/vize/pull/7478) | `c05e0ee71b60` | [37110894124](https://github.com/ubugeeei-prod/vize/actions/runs/37110894124) |
| Maintainer-selected incremental tier rename | [#7507](https://github.com/ubugeeei-prod/vize/pull/7507) | `0514f1508bd4` | [37111176007](https://github.com/ubugeeei-prod/vize/actions/runs/37111176007) |
| Native selected iframe-title diagnostics    | [#7506](https://github.com/ubugeeei-prod/vize/pull/7506) | `f9718b1b0ed8` | [37111533378](https://github.com/ubugeeei-prod/vize/actions/runs/37111533378) |

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

The following are implementation targets, not completed features. Each lane
publishes its lowest ready source prefix without waiting for unrelated work.

| Lane                               | Next concrete implementation                                                                  | Integration gate                                                                                         |
| ---------------------------------- | --------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Original syntax and File ownership | Original Element/body/header cursor, style block custody and consuming HandlerBody owner      | Genuine provider ancestry; partial, foreign and interrupted owners cannot mint completion                |
| Vue compiler                       | Immutable const exposure, plain CSS output family, then controls and directive families       | Native L2 completion, shared L3 facts, complete emitted modules/maps and executed Vue reference behavior |
| Vue dialects                       | Full Vue 2 filter arguments and genuine historical/Document profiles                          | Pinned upstream grammar/runtime evidence; no conversion into a different dialect's admission             |
| JS/TS and JSX/TSX                  | Genuine Program JSX component/intrinsic/member facts and missing TS syntax                    | Original AST/source/profile, one existing walk, precise unsupported boundaries                           |
| Formatter                          | Full directive heads, retained Expr-to-Doc provider and proven interpolation preparation      | Whole output, fixed point, parse preservation and native corpus coverage                                 |
| Linter                             | Native positive-tabindex diagnostics, then other syntax/semantic rules                        | Whole diagnostic/help/label/fix parity and genuine L1/L2 inputs                                          |
| Type checker                       | Authoritative original-file Canon adapter over the genuine L4 projection                      | JS checked as JS, TS as TS, complete diagnostic positions/messages/codes and authored mapping            |
| Semantic queries / LSP             | Shared original File position queries, then artifact retention and Vue/workspace operations   | Complete response envelopes, UTF-16 positions, stale/change/close/reopen/cancellation checks             |
| SSR                                | Genuine SSR eligibility in the existing L3 walk and static native L4 output                   | Escaping, full target output/maps, typed refusal, runtime oracle and unchanged instruction ceilings      |
| Incremental tier                   | `vize_incremental` rename merged; complete original artifact retention remains a product task | Native retention, invalidation, concurrency and actual product acceptance                                |

### Active delivery snapshot

This is a dated work assignment, not a replacement for live GitHub state.
Implementation continues while earlier candidates run; exact-head success,
queue admission and actual merge remain distinct states.

| Owner         | Existing delivery                         | State at this baseline                                           | Next independent work                                      |
| ------------- | ----------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------- |
| JSX           | #7486 → #7503 → #7519, native Stack #7504 | Exact-head source Actions running                                | Original JSX L4 JS-module emitter                          |
| Selected root | #7511 → #7522, native Stack #7525         | Parent candidate `be9b7be49211` in queue; child Actions pending  | Genuine static HTML Attribute header closure               |
| SFC setup     | #7521, independent                        | Const source Actions running at `b11d16b04e0e`                   | Genuine parser-certified TS setup eligibility              |
| Vue 2         | #7527, independent                        | Filter-list source Actions starting at `0e7e948793db`            | Remaining historical and Document producers                |
| For           | #7463 plus genuine selected-operand child | Fresh-main replay and exact-head Actions                         | Original control integration without reparsing             |
| Events        | #7510 → #7514, native Stack #7517         | Unmergeable parent removed from queue; canonical replay required | Whole original HandlerBody L2 ownership/resolution         |
| Formatter     | #7518, independent                        | Candidate `44533d11deed` in queue                                | Retained Expr-to-Doc and sealed interpolation preparation  |
| Linter        | #7515, native Stack #7516                 | Iframe parent merged; child retargeted and revalidated           | Remaining rule coverage and complete diagnostics           |
| Styles        | #7512 → #7528, native Stack #7529         | Genuine custody parent refreshed; CSS child Actions starting     | Plain CSS output/maps; typed refusal for semantic styles   |
| SSR           | #7509 → #7523, native Stack #7524         | Provider source green pending aggregate; target assertion repair | Executed full SSR modules/maps and native runtime oracle   |
| Type checker  | #7526, independent                        | Owned URI/inventory failures under repair; no queue credit       | Original-file backend acceptance, then Vue/JSX projections |
| Queries / LSP | #7513, independent                        | Unmergeable entry removed; fresh-main replay in progress         | Genuine artifact retention on a safe consumer worker       |
| CI steward    | #7520, independent                        | Required source checks passed; candidate `9f42382f8bf8` in queue | Terminal merge and fresh-main validation                   |

These owners work in separate `wt` trees. Genuine native Stack membership
and positions were queried on GitHub. A queued parent cannot accept a new
layer: preserve its in-flight candidate, then refresh a remaining consumer
after its actual merge. A parent-base link alone receives no Stack credit.

The shared Nix dependency-build failure was traced to Crane replacing the
new parser compatibility facade with an empty dummy library. The targeted
fix [#7520](https://github.com/ubugeeei-prod/vize/pull/7520) restores both real
vendored parser roots. Its hosted
[Nix build](https://github.com/ubugeeei-prod/vize/actions/runs/37111603790/job/111170442970)
passed at source `09e8f9e098dd`; the surrounding manual Check was still
running at this snapshot. Earlier full runs for #7507 and #7512 retain their
historical Nix failures. No fresh-main or terminal whole-Check success is
inferred from the targeted job.

The audit still finds unresolved native requirements: Document/in-DOM
tree rules, historical dialect products, full script/control/JSX semantics,
native Vapor, Vue type projection, all-level LSP retention, and the five
complete fix-history corpora. Structural style custody receives no CSS
compilation credit; arbitrary raw CSS would omit `v-bind()` semantics.

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
5. Before replacing each default, close that product's fixture gate:
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
