# Handoff: refactor/remove-codenames (#6832, gate `levels.codenames`)

Base: 4a0fb417ed. The branch is a full scripted replay. A rehearsal from the derived commit lists
reproduces the branch tree byte for byte. Gate `levels.codenames`: 0 paths / 0 lines.

## Maintainer rules for this PR (2026-11-19)

- Hard rename without aliases; every public rename listed under "Breaking changes" in the PR body.
- Refined gate measure accepted (decision record updated).
- Move/rename-only PR passes if: optimized LLVM IR is identical to base after normalizing symbol
  names and paths (IR-identity checker, being built by the levels-inputs agent), output is
  byte-identical, and the release (fat LTO, 1 CGU) binary does not grow. ci-opt and release
  instruction row shifts are reported for information.

## Replay (merge window)

The agent scratchpad has `codenames/replay-plan.sh <repo> <base> <branch> <pre> <post>`, which
derives the lists (the pre-move commits plus every later commit that touches only the script and
its test; the post commits are the rest; it drops the move, rewrite and `tmp:` commits).
`codenames/replay.sh <worktree> <base> <branch> <pre> <post>` then: reset, cherry-pick pre,
`--move` + commit, `--rewrite` (node 26 outside `nix develop`, pinned rust bin on PATH) + `git add
-A` + commit, cherry-pick post, `--check`. Drop the handoff commit for the final PR.

## Evidence so far

- Behavior (ci-opt `vize`, 2078 .vue files, A/A clean): compile DOM/SSR/Vapor, lint, fmt, check and
  virtual TS are byte-identical. Only the help text changes (the renamed CLI names).
- Tooling: every regression found by the 11-19 full checks is fixed locally. Left: CI-only tests
  (literal sources need Actions) and differential-compiler-cli (needs the CLI build).
- Rust, `--profile ci` per package, local: l0, l1, l1_to_l2, l2, l2_to_l3, l3, croquis, armature,
  patina, atelier_core/dom/ssr/vapor/jsx, incremental and musea (after the golden regeneration) are
  green. Two failures are environmental (they also fail on base or are path aliasing): l4
  `tsrx_dom` (fails on base too) and carton `declared_platform_follows_its_hoisted_slot`
  (/tmp vs /private/tmp). maestro `level_ts40_projection` is green. atelier_sfc, maestro (full) and
  vize are not yet run (paused at 58 GB free).
- ci-opt size: base 71,862,992 B, head 71,862,256 B (−736). ci-opt instruction shift is explained by
  name hashing (probe runs 44469785640, 44469790948). The base release-profile instruction
  measurement is in run 44488538947.

## Open

- Full check on the branch head (dispatch queued behind the CI backlog).
- IR identity (checker pending), release size (release-measure workflow pending), A/A perf
  observation, and the remaining local Rust packages once disk ≥ 60 GB.
