# Native Vue toolchain delivery checkpoints (2026-10-01)

Paired decision: [#6826](https://github.com/ubugeeei-prod/vize/issues/6826).
This records execution targets for Vue Fes Japan on 2026-10-24, not a
guarantee that unfinished work will fit the deadline.

## Scope and acceptance

The event scope remains Vue templates and every Vue dialect, JS/TS and
JSX/TSX. Compiler, type checker, linter, formatter and LSP must consume
the shared native structure. Existing product behavior and private
implementations are not native completion evidence.

As of this decision, none of the five products has completed its native
migration. The five fix-history gates remain open. Checked source and
syntax, canonical L2 ownership and native L3 decisions are being delivered
as bounded providers; product integration remains unfinished.

Every accepted case must identify the actual native route and compare its
complete public result. Unsupported inputs stay explicitly unsupported.
Legacy-backed fallback never counts as an accepted native case. Levels
share arena data without serialization or additional pipeline stages.

## Checkpoints

| Target                   | Required evidence                                                                                                                                                                                      | Consequence of missing it                                                                                             |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| 2026-10-10               | A representative Vue input reaches L1, canonical L2, L3 and L4 through a real native producer and emitter, with no legacy implementation on that route; exact-head Actions and protected merge finish. | Reassess the event scope and report the missing native stages explicitly. Full event-scope delivery becomes unlikely. |
| 2026-10-17               | Event-scope functionality is implemented and exercised through the five products, with explicit per-product native acceptance and the relevant fix-history gates satisfied.                            | Report remaining functionality as unfinished; do not relabel a demo or a partial corpus as completion.                |
| 2026-10-18 to 2026-10-23 | Stabilize the implemented scope: regression, dialect/project coverage, full merge-group suites and instruction ceilings.                                                                               | Keep failing candidates out of the queue and state the remaining blockers before the event.                           |

These are checkpoints for action and reassessment. They do not replace
the roadmap order, provider prerequisites or product replacement gates.

## Parallel delivery

Use isolated worktrees and independent agent ownership. Prioritize the
real producer/consumer chain while product fix-history work proceeds in
parallel. Keep move-only commits separate and use Rust or TypeScript for
replay scripts.

Dependent slices use a GitHub native Stack with verified ordered
membership. An exact-head green contiguous prefix enters the protected
merge queue as a stack. Independent PRs use squash auto-merge after their
checks pass. Record actual merge commits and fresh-main ancestry; an open
PR or queue entry is unfinished delivery.

The immediate core sequence is retained L1 syntax, canonical L2 ownership,
single-walk native L3 decisions, fused L1-to-L2 production, and native L4
expression/runtime/emitter implementations. Product adapters consume
these real APIs only after their gates pass. Shared infrastructure stays
in `davinci/`; products and transitional adapters stay in `crates/`, with
normal/build dependency direction enforced.
