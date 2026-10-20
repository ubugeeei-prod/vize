# #6852 — legacy fix differential inputs

A legacy behavior fix must add its input to a registered product differential
corpus. A product regression test or snapshot alone does not establish that the
Davinci lane sees the same input.

## Decision (approved 2026-10-20)

The maintainer approved enabling the requirement now, before native product
adapters exist, with this scope:

- **Who:** pull requests whose title is a conventional `fix(...)` and that
  change legacy product source under `crates/*/src/`: compiler
  (`vize_armature`, `vize_relief`, `vize_atelier_*`), formatter (`vize_glyph`),
  linter (`vize_patina`), typechecker (`vize_canon`) and LSP (`vize_maestro`,
  `vize_resident`). Tests and Davinci level crates do not count.
- **What:** for each affected product, the PR adds a new active case to
  `tests/_fixtures/differential/<product>/manifest.json` whose inputs are
  pinned by SHA-256. A new compiler case also pins its complete expected SSR
  module (`<root>/module.expected.txt`). Missing, unchanged or mismatched
  manifests fail the `pr-source-checks` planner step.
- **When:** PRs opened from 2026-10-20 06:00 UTC, just after the gate merged.
  Earlier PRs keep the previous report-only summary, so in-flight work is not
  blocked.
- **Parity:** native whole-product parity is not judged by this step. Every
  product manifest still has `native: null`, so parity stays a report from the
  differential suites until native adapters exist.

The planner reads the PR title and creation time from the pull request event
and runs `tools/support/compat/github/report-legacy-fix-fixtures.ts`.

## Not covered yet

- Non-`fix(...)` behavior changes and JS product sources (CLI, Vite/unplugin,
  Oxlint, Nuxt, VS Code) are not classified. The 2026-09-29 draft mapped them by
  path regardless of title; applied to open PRs on 2026-10-20 it flagged 8 of 9
  and would have required native parity that no product can provide yet.
- Requiring native parity in the merge queue waits for executable native
  adapters for all five products in the shared `#6891` contract.

Legacy bug fixes are owned by the separate bug agent; this work only supplies
their fixture policy.
