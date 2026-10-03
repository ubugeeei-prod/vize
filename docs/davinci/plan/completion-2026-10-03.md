# Native completion ledger (2026-10-03)

Verified baseline: main `05b8b11b90168c59be71e4a1f8a529f2ca7028dd`.
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

| Surface             | Available bounded functionality                                                                                  | Remaining acceptance                                                                   |
| ------------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| L0                  | Runtime ownership, derive, allocator accounting and dependency enforcement                                       | Platform/config/i18n/profiler separation, resource criteria                            |
| L1                  | Container admission, native component syntax, retained Program/embedding observations, bounded Vue 1 text syntax | Complete dialect and embedding semantics                                               |
| L2                  | Canonical File, original Program declarations/references and checked source coordinates                          | Full syntax, original template-body ownership, controls and product consumers          |
| L3                  | DOM decisions over bounded genuine File artifacts                                                                | Complete shared DOM/SSR/Vapor decisions and native control integration                 |
| Compiler            | Explicit native SFC entry for scriptless static/literal DOM                                                      | Setup scripts, controls, bindings, all dialects, JSX/TSX, SSR/Vapor and history parity |
| Formatter           | Native Doc construction and plain template/attribute layout                                                      | Directive and embed coverage, style/corpus properties and history parity               |
| LSP                 | Source snapshots, cancellation/current-publication guards and shared project APIs                                | Complete level retention and native IDE operations/response parity                     |
| Linter/type checker | Foundations and existing legacy regression preparation                                                           | Native product acceptance and replacement                                              |

Representative merged implementation: [allocator #7438](https://github.com/ubugeeei-prod/vize/pull/7438),
[Vue 1 text #7452](https://github.com/ubugeeei-prod/vize/pull/7452),
[File decisions #7453](https://github.com/ubugeeei-prod/vize/pull/7453),
[native DOM #7455](https://github.com/ubugeeei-prod/vize/pull/7455),
[SFC observation ownership #7457](https://github.com/ubugeeei-prod/vize/pull/7457),
[explicit native SFC #7459](https://github.com/ubugeeei-prod/vize/pull/7459),
[project #7466](https://github.com/ubugeeei-prod/vize/pull/7466) and
[cancellation #7467](https://github.com/ubugeeei-prod/vize/pull/7467).
These entries deliberately do not assert whole-dialect or whole-product parity.

The security unblock [#7469](https://github.com/ubugeeei-prod/vize/pull/7469)
actually merged at `2026-10-03T03:21:19Z`.
Its [protected merge-group Check](https://github.com/ubugeeei-prod/vize/actions/runs/37091781490)
passed full workspace/tooling validation and the unchanged 100 instruction
ceilings. Source-head validation and prospective candidates remain separate
from that actual signed merge.

## Parallel implementation lanes

The following are implementation targets, not completed features. Each lane
publishes its lowest ready source prefix without waiting for unrelated work.

| Lane                               | Next concrete implementation                                                                   | Integration gate                                                                                         |
| ---------------------------------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Original syntax and File ownership | Selected original component/descriptor, full header/body cursor, retained conditional operands | Genuine provider ancestry; partial, foreign and interrupted owners cannot mint completion                |
| Vue compiler                       | Setup-script assembly, Element/body integration, then v-if/v-for and binding families          | Native L2 completion, shared L3 facts, complete emitted modules/maps and executed Vue reference behavior |
| Vue dialects                       | Vue 2 text/filter syntax followed by the remaining historical and document profiles            | Pinned upstream grammar/runtime evidence; no conversion into a different dialect's admission             |
| JS/TS and JSX/TSX                  | Genuine Program JSX component/intrinsic/member facts and missing TS syntax                     | Original AST/source/profile, one existing walk, precise unsupported boundaries                           |
| Formatter                          | Static bind/prop heads, then remaining directive/embed layouts                                 | Whole output, fixed point, parse preservation and native corpus coverage                                 |
| Linter                             | Native L1 img-alt diagnostics followed by native syntax/semantic rule coverage                 | Whole diagnostic/help/label/fix parity and genuine L1/L2 inputs                                          |
| Type checker                       | L4 whole-Program projection and an opt-in existing-checker adapter                             | JS checked as JS, TS as TS, complete diagnostic positions/messages/codes and authored mapping            |
| LSP                                | Native definition/reference queries over the current original Program/File                     | Complete response envelopes, UTF-16 positions, stale/change/close/reopen/cancellation checks             |

The read suffix [#7460](https://github.com/ubugeeei-prod/vize/pull/7460),
[#7461](https://github.com/ubugeeei-prod/vize/pull/7461) and
[#7465](https://github.com/ubugeeei-prod/vize/pull/7465) remains unfinished at
this baseline. It must replay its own changes onto the merged security parent
and obtain fresh exact-head validation before its contiguous native Stack
prefix enters the queue. Old green runs do not validate a new head.

The independent original-owner provider prefix is published as
[#7471](https://github.com/ubugeeei-prod/vize/pull/7471) and
[#7472](https://github.com/ubugeeei-prod/vize/pull/7472), registered in native
Stack #7473. Publication is not merge evidence. This earliest source prefix
does not wait for Element/control emission or unrelated product features.

Known source findings remain visible: the new original-owner DOM route needs
Vue whitespace eligibility/normalization; raw lossless Text alone is not
semantic DOM admission. Parser [#7444](https://github.com/ubugeeei-prod/vize/issues/7444)
also remains open. Neither a conservative refusal nor an unproven memoization
candidate closes those feature/robustness requirements.

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
