# AGENTS.md

This is the entry point for autonomous agents working on Vize. It matters
most when you have been told to "do the rest" and left to run.

## Where the roadmap lives

- **Roadmap issues:**
  [#6826](https://github.com/ubugeeei-prod/vize/issues/6826) (level
  restructure),
  [#6827](https://github.com/ubugeeei-prod/vize/issues/6827) (products on
  the levels),
  [#6828](https://github.com/ubugeeei-prod/vize/issues/6828) (legacy
  deletion),
  [#6829](https://github.com/ubugeeei-prod/vize/issues/6829)
  (multi-framework) and
  [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) (CI). Each has
  sub-issues, and every sub-issue body has an `**Order:**` line.
- **Decisions:** the
  [decision record](./docs/davinci/decisions/2026-09-27-level-restructure.md),
  including its "Order of work" table.
- **Detailed designs:** the design comments on
  [#6836](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
  (L1 embeds),
  [#6839](https://github.com/ubugeeei-prod/vize/issues/6839#issuecomment-5847890775)
  (L3) and
  [#6840](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5847908963)
  (L4).

**Picking work:** take the lowest-stage open issue whose "Start after"
issues are all closed.

## Working rules

- **Record every decision and TODO in an issue or in `docs/`.** Never keep
  them only in session or agent memory.
- **Record each decision in two places in the same change:** as a comment
  on its issue, and in the
  [decision record](./docs/davinci/decisions/2026-09-27-level-restructure.md).
- **Keep PRs small** and give them conventional titles, e.g.
  `refactor(l1): …`.
- **Put moves in move-only commits.** Git rename detection then keeps
  in-flight fixes rebasable.
- **Script renames.** On a conflict, re-run the script on `main`.
- **Compatibility:**
  - Davinci (the level crates) has no users yet, so you may break it freely.
  - Legacy products keep byte-exact output, checked by the differential
    corpus.
  - A PR that fixes legacy behavior must add a corpus fixture.
- **Dependencies:** level crates never take normal dependencies on legacy
  crates (`vize_armature`, `vize_relief`, `vize_atelier_*`,
  `vize_croquis`).
- **Naming:**
  - Crates carry level names only (`vize_l0` … `vize_l4`, `vize_l1_to_l2`,
    `vize_l2_to_l3`, `vize_l0_derive`).
  - Codenames never appear in crate, file, module or type names, or in
    serialized strings.
- **Performance:**
  - Add no extra pipeline stages and no serialization between levels.
  - Instruction-count gates run in the merge queue.
  - Budgets only ratchet down.
- **Asking the maintainer:** ask only for decisions that are genuinely
  theirs, and present them as multiple-choice options.

## Merging

Enable auto-merge (squash) once PR checks pass. The merge queue runs the
full suites.

## Vue Fes Japan (2026-10-24)

- **In scope:** Vue (templates and every Vue dialect), JS/TS, JSX/TSX.
- **Unfinished work** is reported as unfinished.
- **No legacy-backed shortcuts.**
