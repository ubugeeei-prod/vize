# Vue dialect legalization owner (2026-10-01)

Issue: [#6841](https://github.com/ubugeeei-prod/vize/issues/6841).
Provider: [shared Vue capability owner](./2026-10-01-dialect-capability-owner.md).

## Decision

The L1-to-L2 conversion dialect module owns the existing Vue sugar legalizer
and its seven helpers. `dialect::legalize` consumes the shared L1 capability
projection through the conversion dialect module, so this slice depends on
the shared capability provider.

The legalizer retains its current algorithms for filters, `slot-scope`,
`.sync`, `.native`, numeric event modifiers, id shifts and side-table rekeying.
Its filter handling also applies to pre-Vue-2 lines; the common entry therefore
is not named `vue2`. It reuses the same neutral page-order walker and the same
Vue 3 pass descriptors. `pass::legacy` becomes a narrow public compatibility
re-export, keeping existing callers and tests valid.

The pass name, kind, fusability, preservation, order, grouping and skip
conditions remain unchanged. This move adds no stage, walk, allocation,
serialization, product-route switch, fixture change, snapshot change or numeric
instruction-budget change. Generic scheduling still reaches the policy through
the transitional `pass::legacy` adapter; complete policy isolation remains
open under #6841.

## Replay and acceptance

Eight file moves form a move-only commit. Replay them with:

```sh
vp node tools/support/levels/move-dialect-legalization.ts move
```

After applying the wiring commit, `check` verifies the new owner, helper moves
and narrow adapter. The script preflights every move, refuses conflicting or
unreviewed helpers, and preserves exact source bytes. Reviewed storage rows
change paths only, with all category and count cells preserved.

Existing lowering, legalization and filter-output tests remain the behavior
oracle. Acceptance requires exact-head Actions, byte-exact differential
fixtures and all immutable instruction ceilings before a contiguous native
Stack prefix enters the protected merge queue. Local moves do not establish
acceptance or merge.

Focused local validation passes twenty-nine existing lowering, legalization
and emitted-filter laws, plus ten replay/isolation/storage laws. Reviewed
storage cells retain their previous categories and numeric counts.

## Remaining work

The file descriptor, conversion-pattern tables, residual dialect operations,
full core isolation, L4 options and non-Vue language modules remain open.
This slice does not close #6841 or establish native product parity.
