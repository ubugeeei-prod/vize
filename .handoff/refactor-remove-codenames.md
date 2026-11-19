# Handoff: refactor/remove-codenames (#6832, gate `levels.codenames`)

Base: 4a0fb417ed. The branch is a full scripted replay (rehearsed four times, deterministic).
Gate `levels.codenames`: 0 paths / 0 lines. Maintainer decisions 2026-11-19: hard rename without
aliases ("Breaking changes" in the PR body); refined gate measure accepted; instruction rows and
binary size are judged on the shipped release profile (fat LTO, 1 CGU), ci-opt shifts are
informational.

## Replay (merge window)

`replay.sh <worktree> <base> <branch> <pre-list> <post-list>` in the agent scratchpad does all of it:
reset, cherry-pick the pre-move commits, `--move` + commit, `--rewrite` (node 26 outside
`nix develop`, pinned rust bin on PATH) + `git add -A` + commit, cherry-pick the post-rewrite
commits, `--check`.

- Pre-move (script and its tests): every `refactor(levels): script …`, `rewrite codename prose …`,
  `delete the deprecated program-named feature aliases`, `keep the old L3 package and op names`,
  and every `fix(levels): keep … / re-pin … / skip … / reflow … / treat …` commit that touches only
  `tools/support/levels/rename-codenames.ts` and `tests/tooling/codename-rename.test.ts`.
- Post-rewrite: alias lines, instruction ratchet, replay-script deletion, storage bridge record,
  translated docs, decision record (x2), regenerated lists, retired-name guards, historical
  compiler rows, fixture digests of touched bench sources, re-blessed remarks baselines,
  reconstructed-original re-pins (#6832 notes), frozen copies read through the rename, q16, archived
  instruction evidence, suite registry, release metadata, native DTO custody, native identity images.

## Rules the script now carries

- Keep bytes: history verifiers and their manifests, L2 v1 wire tests, literal listing sources,
  captured provenance, fix-history TSV ledgers, lint preset audit inputs, archived instruction
  evidence, retained template-emit manifest, 47c6 research packets, `*-observation.json`,
  `/tmp` paths, recorded commit subjects.
- Re-pin (current bytes only): compiler fix-history receipt, native source recipe fixtures,
  compiler differential manifest, Vapor style witnesses. All other digests stay (most pin past
  bytes that only equal main's).

## Still open

- Full check on the replay4 head (queue was >140; dispatch when it drains). Expect remaining CI-only
  checks (literal sources need Actions).
- Rust: regenerate `vize_musea` golden fingerprints (`VIZE_MUSEA_GOLDEN_WRITE=1`) and the
  `vize_maestro` ts40 insta snapshots if the projection text embeds the renamed fixture path.
- Release-profile instruction counts: base twin, head, head twin with the temporary
  `[profile.ci-opt]` fat-LTO override (`bench/codenames-release-*` branches); release size A/B.
- A/A performance observation.
