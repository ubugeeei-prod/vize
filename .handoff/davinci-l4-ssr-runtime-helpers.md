# Handoff: lane B (L3/L4 backends), 2026-11-19

Delete this file before merging.

## Branches and PRs

| Slice | Branch | PR | State |
| ----- | ------ | -- | ----- |
| L3-1 record | `davinci/l3-shared-decisions` | #13398 | merged |
| L4-1 one `EmitDocument` | `davinci/l4-one-emit-document` | #13390 | code final candidate (v7); instruction evidence incomplete |
| L4-2 SSR helpers in `vize_l4::runtime` | `davinci/l4-ssr-runtime-helpers` (this branch, stacked on L4-1) | none yet | code done; parity and suites done; ci-opt vs its v5 parent (run 44494869586 vs 44490444478): 12 `atelier_ssr_*` rows fall (up to −3.2%), 89 identical, 0 regress; twins and re-measure on the final L4-1 still owed |

## L4-1 measurement state

Base for every instruction run: `9cf1ccf4b1` (ci-opt runs 44477685028 / twin 44479823816;
release-shaped runs 44481946281 / twin 44483432053, A/A 0 on all 101 rows). "Release-shaped" =
`[profile.ci-opt] lto = "fat", codegen-units = 1`, as in #13340. Variants were grafted onto the
old head `6be122088b` (branches `measure/laneb-l41-*`), so they share the base.

| Variant | `EmitDocument` inlining | ci-opt regressions | release-shaped regressions |
| ------- | ----------------------- | ------------------ | -------------------------- |
| v1 (`74d3eacfd2`) | level doc as it was (`#[inline(always)]` on push_spanned/expression/escaped) | 20, ≤ +0.73%, untouched stages | 25, up to +2.4% (push_spanned inlined into SSR/Vapor callers) |
| v3 | identical to the deleted atelier copy | 46, up to +3.6% (push_escaped out of line, slower) | 21 |
| v5 | v1 + link bookkeeping out of line, lean push_escaped `inline(always)` | 12, ≤ +0.48% | 7, up to +1.65% (SSR wrappers outlined) |
| v6 | v5 with linking mutators left to the inliner | 8, ≤ +0.58% | 5, ≤ +0.11% (SSR `flush_push` escape loop) |
| v7 (`35b30eb935`) | v6 + push_str `inline(always)`, push_escaped not forced | 31, up to +2.2%: out-of-line push_escaped in Vapor generate | 2: +0.014% (SSR flush_push), +0.006% (untouched l1_to_l2) |
| **v6 = PR head `8567dd13a1`** | see v6 row; ci-opt run 44492585295, release-shaped 44492579701 | 8, all untouched stages | 5, all at most +0.11% |

Every remaining ci-opt increase is in code this PR does not touch, per callgrind self-cost diffs
(`core::str::from_utf8` in Croquis rows, `RuntimeHelpers::add` in atelier_core transforms,
closures in the native DOM artifact, recursion splits in Vapor lowering). That is thin-LTO
codegen-unit repartitioning after `vize_atelier_core` lost two modules and gained a dependency.
It moved differently with every variant. Gains in the touched rows are large (ci-opt v1: SSR
codegen −24%, Vapor generate −20%).

Tools (scratchpad of the 2026-11-19 session; recreate if gone):
- `lbic.py <base> <head> <base-twin> <head-twin> [kind]`: row table from downloaded artifacts.
- `lbcg.py <base-run> <head-run> <row>`: per-function callgrind self-cost diff. The artifacts already
  contain `run-N/*.callgrind*` dumps, so no extra workflow is needed.
- `lbgraft.sh`, `lbrelcommit.sh`: plumbing-only measurement commits (no checkout).

## Next steps

1. #13390 now carries v6 (the best balance of the two profiles; v7 was worse on ci-opt). Run the
   head twins and read the PR performance observation for `8567dd13a1`.
   Release size probe: the base `vize` is 57,219,936 B and vitrine napi is 80,706,640 B
   (`a74306cee7`, release profile, macOS arm64). Build the head under the fleet lock
   `/private/tmp/claude-501/vize-release-build.lock`.
2. The residual ci-opt shifts in untouched stages need a maintainer ruling. Size already has the
   release-profile rule; instructions do not. If the ruling is "release-shaped decides, as for size",
   v7 (or v6) is mergeable.
3. L4-2: rebase on the final L4-1, then measure the instructions against the L4-1 head
   (`measure/laneb-l42-head` was grafted onto v5). The PR body draft is in the session scratchpad
   (`laneb-l42-body.md`). Expected: `atelier_ssr_*` rows fall (bit mask instead of two
   `FxHashSet`s plus sort).
4. L4-3: native SSR still parses with armature first (`compile/inner.rs`). It also runs the legacy
   transform even when the native plan emits, because callers read the transform's
   `CompilerError`s. `frozen_slots.rs` takes v-pre frozen spans from the legacy parser. To parse
   legacy only on refusal you need (a) L1 frozen-element spans and (b) the SFC-visible errors
   (parse fatal/recoverable plus transform) produced from L1/L2 diagnostics with identical
   messages. Do (b) first, as its own parity slice.
5. L4-4 (move `atelier_ssr/src/l4` into `vize_l4::targets::ssr`), then L3-2 per #13398.
