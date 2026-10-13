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
  [canonical collection](./docs/davinci/decisions/README.md), rooted in the
  [foundation record](./docs/davinci/decisions/2026-09-27-level-restructure.md)
  and its "Order of work" table.
- **Detailed designs:** the design comments on
  [#6836](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
  (L1 embeds),
  [#6839](https://github.com/ubugeeei-prod/vize/issues/6839#issuecomment-5847890775)
  (L3) and
  [#6840](https://github.com/ubugeeei-prod/vize/issues/6840#issuecomment-5847908963)
  (L4).

**Picking work:** prefer the lowest-stage ready slice. The
[2026-09-28 maintainer decision](https://github.com/ubugeeei-prod/vize/issues/6826#issuecomment-5871526916)
allows independently reviewable #6832 and L1-L4/dialect structural PRs to
open and merge in parallel while #6832 remains open. Actual provider APIs and
product fix-history gates still apply; use the [order record](./docs/davinci/decisions/2026-09-27-level-restructure-order.md).

## Working rules

- **Record every decision and TODO in an issue or in `docs/`.** Never keep
  them only in session or agent memory.
- **Record each decision in two places in the same change:** as a concise
  comment on its existing issue and a dedicated record under
  [`docs/davinci/decisions/`](./docs/davinci/decisions/README.md). The full
  collection, including the foundation and its companions, is canonical.
  Follow the [recording workflow](./docs/davinci/decisions/README.md): give
  independent decisions their own files, without appending each new decision
  to the shared foundation or index.
- **Keep PRs small** and give them conventional titles, e.g.
  `refactor(l1): …`.
- **Finish PRs.** Drive every PR you open to an actual merge, or close it with
  a one-line reason. Do not leave PRs waiting, and do not block other work
  while CI runs. Merge iteration speed is the bottleneck, so slice work to
  avoid conflicts.
- **Use `wt` for isolated worktrees** and assign independent slices to
  sub-agents. Recheck remote heads before writing to an existing PR branch.
- **Clean up as you go.** When a slice lands, remove its worktree, its local
  branch and its cargo target directory. Local disk is a shared, limited
  resource.
- **Prioritize third-party reports.** They are urgent. Include the issue
  reporter in a `Co-authored-by` trailer with a verified public identity.
  Carry the fix through actual merge and a published release, then close the
  issue with the released version.
- **Release often.** Cut a release for each user-facing fix instead of
  batching. Release gates exist to stop regressions; a gate that requires
  perfection on every surface stops fixes from reaching users.
- **Keep third-party issue comments rare and short.** Comment only when the
  reporter needs the information, in one or two sentences.
- **Upstream repositories are read-only.** Never comment on, open issues or
  pull requests against, or push to a project that is evaluating or consuming
  Vize (for example n8n). Track the work in a Vize issue instead.
- **Tooling languages:** write tools, scripts and fixture helpers in
  TypeScript or Rust. Python and MoonBit are not accepted, and new `.mjs`
  files are not added
  ([#8840](https://github.com/ubugeeei-prod/vize/issues/8840)). Formal
  verification uses Kani, not Lean.
- **Keep output small.** Use `rtk` for git, gh, grep and test output so that
  agent context is spent on the work.
- **Source layout:** authored files may contain up to 500 lines; existing larger
  files must not grow. Keep small Rust tests inline, or use a sibling
  `<module>_tests.rs` with `#[cfg(test)]` and `#[path]`. Do not create a
  directory solely to house `test.rs` or `tests.rs`. Production modules use
  ordinary module discovery.
- **Put moves in move-only commits.** Git rename detection then keeps
  in-flight fixes rebasable.
- **Script renames.** On a conflict, re-run the script on `main`.
- **Compatibility:**
  - Davinci (the level crates) has no users yet, so you may break it freely.
  - Legacy products keep byte-exact output, checked by the differential
    corpus.
  - A PR that fixes legacy behavior must add a corpus fixture.
  - Before replacing a product's legacy path, that product's fix-history
    fixture issue must be closed:
    [#6879](https://github.com/ubugeeei-prod/vize/issues/6879) type checker,
    [#6880](https://github.com/ubugeeei-prod/vize/issues/6880) compiler,
    [#6881](https://github.com/ubugeeei-prod/vize/issues/6881) linter,
    [#6882](https://github.com/ubugeeei-prod/vize/issues/6882) formatter,
    [#6883](https://github.com/ubugeeei-prod/vize/issues/6883) LSP.
- **Layout:** Davinci levels and shared infrastructure live in `davinci/`;
  legacy products and their transitional adapters live in `crates/`.
- **Dependencies:** `crates/` may depend on `davinci/`. The reverse is forbidden
  for normal and build dependencies, including optional and target-specific
  declarations and transitive workspace paths. Dev-only differential oracles
  are permitted. Level crates never take normal dependencies on legacy
  crates (`vize_armature`, `vize_relief`, `vize_atelier_*`,
  `vize_croquis`).
- **Naming:**
  - Crates carry level names only (`vize_l0` … `vize_l4`, `vize_l1_to_l2`,
    `vize_l2_to_l3`, `vize_l0_derive`).
  - Codenames never appear in crate, file, module or type names, or in
    serialized strings. The maintainer-selected `davinci/` program root is
    the directory exception alongside `docs/davinci/`.
- **Performance:**
  - Add no extra pipeline stages and no serialization between levels.
  - Instruction-count gates run in the merge queue.
- Budgets only ratchet down.
- The type checker has a **10x speed target**. Record a reproducible baseline,
  compare the same workload and diagnostics on Actions, and report the measured
  result separately from the target.
- **Asking the maintainer:** ask only for decisions that are genuinely
  theirs, and present them as multiple-choice options.

## Merging

For dependent slices, use a GitHub native Stack. Branch each child from its
parent's head, create each PR with the parent branch as its base, and register
all existing PRs with `gh stack link --remote origin <bottom> ... <top>`.
Verify GitHub reports the same Stack number and ordered positions for every
PR. Parent-base links or prose alone do not establish a native Stack.
Do not auto-merge an individual layer. Once a contiguous prefix's exact-head
Actions passes, run `gh stack merge <highest-ready-PR> --yes --squash` to enter
the protected merge queue. After that prefix actually merges, rebase and
retarget remaining children onto fresh `main`, rerun Actions, and verify their
Stack membership. Track each queue candidate and actual merge before completion.

For an independent PR, enable auto-merge (squash) once its PR checks pass.
The merge queue runs the full suites. If a queue candidate fails, remove its
PR from the queue until the failure is fixed; do not leave a known-red
candidate blocking later PRs.

## Vue Fes Japan (2026-10-24)

- **In scope:** Vue (templates and every Vue dialect), JS/TS, JSX/TSX.
- **Unfinished work** is reported as unfinished.
- **No legacy-backed shortcuts.**
