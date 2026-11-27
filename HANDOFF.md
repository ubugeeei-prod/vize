---
name: vize-handoff-2026-11-27-p0
description: "11-27 evening handoff: Hal-Spidernight P0 round (#13787-#13798), CI cleared for P0, dequeued PRs to re-queue after the P0 release, paused third-party PRs"
metadata:
  node_type: memory
  type: project
  originSessionId: ee71f8ac-090c-40c4-8d7f-efcaae043c92
  modified: 2026-11-27T08:18:05.843Z
---

On 2026-11-27 (~17:30 JST) the user said "最速リリースで頼むぜ．ここからはハンドオフ". Goal: release the Hal-Spidernight P0 fixes as fast as possible. See [[vize-hal-spidernight-p0]].

## CI state (user-authorized)
- The user asked the other sessions to stop and authorized cancelling CI.
- I dequeued the whole merge queue and cancelled about 70 non-P0 runs.
- **Dequeued, re-queue after the P0 release** (`gh pr merge <n> --auto`): #13741 #13726 #13772 #13771 #13773 #13777 #13766 #13776 #13739 #13782.
- #13780 was re-queued alone.
- **Main-side breaks to land first**, because the P0 PRs' tooling shards hit them:
  - #13780: fresco `docs-public-api` JSDoc.
  - #13786: template-ref keys, auto-merge on.
  - #13785: maestro generic navigation, auto-merge on.

## P0 agents (worktrees under `.claude/worktrees/agent-*`)
They share `CARGO_TARGET_DIR=.claude/p0-target` with debug=0, because disk was down to 14 GB.
- #13787 + #13789: canon. `[String, null]` drops `null`. For bare `<img alt>`, the likely answer is a docs note, not a diagnostic.
- #13790: corsa discovery start directory.
- #13788: emits of packaged generic components typed as `any`.
- #13791: Nuxt `viteEnvironmentApi` manifest. PR #13799.
- #13792 + #13793: patina rules.
- #13797 + #13798: glyph. Empty-element `printWidth`, and SFC block spacing under the prettier preset (plus a new option).

## Paused by me for P0 (resume after the release)
- **#13769** (warflash oxlint): green except `docs-public-api`. Update the branch after #13780, then `--auto`. Delete the mirror branch `perf/pr-13769-oxlint-type-world` after it merges.
- **#13774** (#13767): head `ccd03fe8d3`, auto-merge on. If another lockfile change lands first, regenerate the canon lock successor and the listhen key.
- **#13778** (#13770): head `457ace68ed`, needs a full CI run.
- **#13721:** branch `fix/canon-global-fallthrough-root`. Open the PR after #13777 merges (body in scratchpad `canon-1372x/`).
- **#13775** (#13547): auto-merge kept OFF. The three `*_composed` crates are not on crates.io (404), and the user must publish them manually first. Someone had turned auto-merge on; I turned it off.

## Release
- Release with `gh workflow run release.yml --ref main -f source=<full sha>` once the P0 PRs merge.
- Verify on the published packages, then post a 3–6 line comment and close each issue.
- One-line "Fix in review: #PR" comments are posted on #13768 #13767 #13547 #13620 #13720 #13770 #13791.
