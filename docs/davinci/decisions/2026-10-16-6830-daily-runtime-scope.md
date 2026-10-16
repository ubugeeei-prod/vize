# Nuxt runtime feedback and complete merge qualification

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The Nuxt module job took 472 seconds in successful merge run
[38861800129](https://github.com/ubugeeei-prod/vize/actions/runs/38861800129).
Its source-built Nuxt 3 build, SSR and browser verification took 11 seconds.
Five additional campaigns consumed 330 seconds: historical SPA prefetch (22),
Nuxt 3/4 lint configuration (67), compiler defaults (21), relative globs (12),
and JavaScript workspace products (208). Evidence upload added 34 seconds for
the complete 617 MB artifact. Dependency installation and the warm native build
took only 4 and 23 seconds respectively.

Every affected PR retains the source-built Nuxt 3 compiler binding, its exact
source and binary receipts, client and SSR compilation, four real routes,
page-metadata layouts, scoped CSS, hydration, counter interaction and browser
navigation. The affected JS package tests and the separate Nuxt 4 style checks
also continue. Nuxt 4 style tests use a pinned published native dependency;
they cannot replace this source-native Nuxt 3 acceptance.

The five extended campaigns, their extra fixture installations and the source
CLI build run on merge groups, main pushes and manual qualification. Their
original assertions and complete evidence remain unchanged. This retains the
historical prefetch comparison, actual Nuxt 3/4 linting, sixteen real compiler
option scenarios, seventy-two relative-glob observations, and the Vite/Nuxt
JavaScript workspace CLI, browser and LSP checks before automatic merging.

Merge groups invoke this workflow through `nuxt-module-qualification` in the
source workflow. The source report requires its success, and the branch's
required `test-report` depends on that source report. A failed, cancelled,
missing or skipped queue qualification cannot produce a green required gate.
Only PRs omit this duplicate complete campaign. The standalone merge trigger
is removed so each queue candidate runs it once.

PR browser setup installs Chromium directly and does not build the CLI needed
only by the extended workspace campaign. Timing from the former complete job
identifies the avoided work; it does not establish the new PR duration. Verify
the changed path in Actions before claiming the five-minute target.
