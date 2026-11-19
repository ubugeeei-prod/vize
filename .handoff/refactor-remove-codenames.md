# Handoff: refactor/remove-codenames (#6832, gate `levels.codenames`)

Paused 2026-11-19 (fleet stop). Base: 4a0fb417ed. Gate `levels.codenames`: 0 paths / 0 lines.
The branch is a full scripted replay. A rehearsal from the derived commit lists reproduced the
branch tree byte for byte. No PR is open; do not open one until the coordinator opens the merge
window.

## Maintainer rules for this PR (2026-11-19)

- Hard rename, no aliases. Every public rename goes under "Breaking changes" (draft in
  `codenames/pr-body.md`, measurements still to fill in).
- Refined gate measure accepted (decision record updated).
- A move/rename-only PR passes when its optimized LLVM IR is identical to base after normalizing
  symbols and paths (`tools/commands/levels/ir-identity.ts`, #13410), its output is byte-identical,
  and the release (fat LTO, 1 CGU) binary does not grow. ci-opt and release instruction row shifts
  are informational (ci-opt shifts come from name hashing: probe runs 44469785640, 44469790948).

## Tools (`.handoff/codenames/`)

- `replay-plan.sh <repo> <base> <branch> <pre> <post>` derives the replay lists (drops the move,
  rewrite, `tmp:` and handoff commits). `replay.sh <worktree> <base> <branch> <pre> <post>` resets,
  cherry-picks pre, `--move` + commit, `--rewrite` + commit (node 26 outside `nix develop`, pinned
  1.100.0 rust bin on PATH; ~6 min), cherry-picks post, `--check`.
- `pr-body.md`: PR body draft with "Breaking changes".
- `node-regressions.txt`: tooling regressions from the first full check (all fixed locally).
- IR plan: `ir-renames.ts` produced `ir-args.txt` (29 `--crate`, 44 `--rename` pairs).
  `dispatch-ir.sh` dispatches `level-instruction-counts.yml` on `bench/levels-ir` with
  base 4a0fb417ed and head de2750a6a6. **It was NOT dispatched** (killed while the queue was > 40).
  The rename changes string bytes (counters, help text, `repro.dump`), so the IR will differ in
  the functions that reference them. `irtool/strings-through-rename.ts` (imports `ir-normalize.ts`
  from #13410) checks that the saved `.ll` pairs differ only by renamed strings and their lengths.

## Status

- Full check run 44517440281 on de2750a6a6 is still queued or running. Compare its failures against
  main's 44465592435 (main fails Full Rust, coverage, vue-parity, allocations, v1 matrix and two
  tooling shards).
- Behavior: the ci-opt `vize` is byte-identical on main's 2078 `.vue` files. ci-opt size −736 B.
- Tooling: all local regressions are fixed. Left: CI-only literal-source tests and
  differential-compiler-cli (needs the CLI build).
- Rust, `--profile ci`, local: l0–l3, croquis, armature, patina, atelier core/dom/ssr/vapor/jsx,
  incremental, musea and maestro `level_ts40_projection` are green. Two failures are
  environmental: l4 `tsrx_dom` (also fails on base) and carton `/tmp` vs `/private/tmp`.

## Open

1. Read full check 44517440281 and fix any head-only failures.
2. IR identity via `dispatch-ir.sh` (queue < 40), then `strings-through-rename.ts` on the
   artifacts.
3. Release size A/B (perf-backlog release-measure workflow); A/A performance observation.
4. Local Rust: `vize_atelier_sfc`, `vize_maestro` (full) and `vize` (disk ≥ 60 GB).
5. Consider moving the keep-bytes and spell-back rules onto the shared evidence rename map
   (levels-inputs agent) when it lands.
6. Merge window: replay on latest main, gates, push, PR, auto-merge; drop this handoff.
