# Session handoff — 2026-11-18 ~02:30Z (break requested by the maintainer)

This branch (`wip/handoff-2026-11-18`) only carries notes. **Never merge it.**
Contents: this file, `lane-brief.md` (common rules every lane followed), `brief-*.md` (Davinci worker briefs),
`pr-drafts/` (PR bodies drafted for branches that have no PR yet), `scripts/` (orchestration helpers; paths inside
are machine-specific).

## Maintainer's order (2026-11-17 13:15Z)
Release third-party fixes with zero regressions; then carry every open PR, finish Davinci, make every real
project in the fixtures work completely, and keep performance/edge-case regressions at zero.

## Releases and third-party issues
| Issue | Reporter | Fix | State |
|---|---|---|---|
| #13237 #13238 #13239 #13121 | IlyaSemenov | #13254 #13255 #13256 | Shipped in v0.778.0 (14:50Z), "Fixed in" posted |
| #13284 | IlyaSemenov | #13288 (merged c92bb47db0) | **Not released yet.** v0.779.0 run 44211756428 failed in the new glibc-floor package smoke (#13242's first real run). Fix #13316 is first in the merge queue. |
| #13295 (Pug escaped attribute mapping, found by us) | — | #13302 | In merge queue |
| #12994 / #13070 | FLPPPPPP | #13272 (fixes a regression live since v0.777.0: 15 extra reports) → #13274 (stacked) | #13272 in queue; #13274 needs rebase onto main after #13272, a `benchmark.yml patina_project_batch` run, then `merge-async` |
| #13002 | FLPPPPPP | branch `fix/patina-declared-type-argument-defaults-13002` @ d1889f16a1 (stacked on #13274) | No PR yet; needs benchmark (adds checker requests). The "blocked-upstream" label no longer applies. |
| #12985 invalid[141] | FLPPPPPP | not started | Plan: sub-span mapping for the rewritten `export default <expr>` row in vize_canon + corpus pair |
| #13319 | naokihaba | branch `fix/imports-template-ref-13319` @ 2662628f6f (WIP, fix verified) | Needs unit test, tsgo CLI test + differential fixture, corpus pair (nuxt-ui), then PR `Closes #13319` with `Co-authored-by: naokihaba <59875779+naokihaba@users.noreply.github.com>` |

**Release procedure:** dispatch `release.yml` with `source=<40-hex main SHA>` only after that SHA's `Check push` or
`Check merge_group` succeeded; run `typecheck-corpus-pair.yml` (misskey,n8n,nuxt-ui,reka-ui) against the last release
SHA first when canon changed. Keep ~90 min between releases (crates.io quota). After npm shows the version, comment
"Fixed in vX (#PR)." on the issue. On this machine `scripts/release-after-13316.sh` was left running to do exactly
that for #13284 once #13316 merges — check whether it ran (`gh run list --workflow release.yml`).

## Merge queue at the break (in order)
#13316 #13276 #13257 #13293 #13289 #13302 #13272 #13278 #13305 #13287 #13304 #13303 #13309 #13279 — all armed and
expected to merge unattended. CI capacity was the bottleneck all day: Blacksmith x86 runners stalled 4 times
(13:19–13:48Z, ~17:55–18:40Z, 19:05–19:47Z, 21:12–21:49Z, plus throttling to ~10 jobs from 23:20Z). With ≥8 queued
groups × ~80 jobs nothing finished, so I dequeued the tail and fed it back in order (`scripts/queue-feeder.sh`).

## Open PRs not in the queue
- #13318 (pin CORSA_PATH in two matrix steps) — armed, CI running.
- #13311 (LSP union alias conflict) — armed, BLOCKED by the davinci assertion lint: 3 loose asserts in
  `crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context/cache/union_conflict_tests.rs` (L104, L123,
  L129) must become exact oracles.
- #13317 (per-app typecheck targets) — not armed. vuetify lib target: vue-tsc baseline fails on errors inside
  node_modules `.d.ts` (@testing-library/dom, happy-dom); decide skipLibCheck-equivalent baseline vs
  configuration-only + issue. lx-music-desktop scoped run not dispatched yet.
- #13299 (canon: dynamic v-bind argument supplies required props, #13269) — misc lane.
- Davinci: #13306 (dup attribute error, after #13304), #13307 (tbody; rebased version local to the lead, after
  #13304), #13308 (authored structure in Patina, after #13304), #13313 (compiler option A, after #13308/#13307),
  #13315 (a11y rules on facade; needs one `benchmark.yml` run), #13262 (jade; after #13231), #13231 (lang registry;
  blocked on picomatch audit).
- #13248 (grouped CLI diagnostics) — **maintainer format decision pending** (hint wording, one location line per
  file, collapse after 8 files, `--verbose` name, `hint` field in `check --format json`).

## Pushed branches without PRs (open when the queue allows)
- `fix/multiline-prop-callback-context` @ f0fe07db90 — vuestic-admin TS7006 (multi-line `:rules` callbacks on a
  component with a listener). Body: `pr-drafts/pr-vuestic.md`. Needs corpus pair.
- `perf-canon-route-inputs-set` @ 49fb3a13e6 — alias build kept 126k route inputs for 5.7k distinct paths (14% of
  preparation samples). Body: `pr-drafts/pr-canon-route-inputs.md`. Needs corpus pair + LSP bench.
- `test/canon-ancestor-node-modules` @ 3cdbbc186a — pins TS's ancestor @types/node_modules walk. Body:
  `pr-drafts/pr-anc-body.md`. Run `cargo test -p vize --test integration check_ancestor_node_modules` with
  `VIZE_TEST_REQUIRE_TSGO=1` first.
- `ci/matrix-profile` @ bf87f7476c — `ci-matrix` profile (ci-opt + debug-assertions + overflow-checks). Measure one
  LSP-heavy shard cold/warm vs main, then one PR switching all 22 shards.
- `fix/picomatch-ghsa-fc5q` @ 637ed57b00 — WIP: patches for picomatch 2.3.2 and 4.0.7 (upstream #246 + literal
  fallback). Remaining: patchedDependencies + lock, proof modules in the braces style, tooling test, audit
  acceptance gated on the proof. Blocks every dependency-changing PR (e.g. #13231).
- Real-project lint lane and Davinci branches: see their sections below.

## Davinci (lead + workers all stopped; everything pushed)
Gates: 17/45 at the last measurement. In the queue: #13304 (foster parenting only for in-DOM), #13303 (S7 #6844),
#13309 (control character), #13279 (TSRX keyed fragments), plus #13289, #13257, #13305.

**Rebase order once #13304 merges** (`git rebase --onto origin/main b68a542555`, then `gh pr edit <N> --base main`,
wait for green, enqueue):
1. #13307 — rebase branch `lead/13307-on-foster` @ dae137fa9e (has the `markup/l2/construct.rs` split that keeps
   l2.rs at 443 lines), then `git push -f origin HEAD:fix/sfc-tables-keep-authored-rows-w`.
2. #13306 — its commits a75ca7661c, 7efbe236fa, 75fe523a02 (trivial conflict with #13308 in
   parse_diagnostics/tests.rs).
3. #13308 — its commits c35992ab63, f379467b12.
4. #13313 (compiler option A) after #13307 and #13308 merge — new branch from origin/main, cherry-pick only
   ba47c83d24, 96b385ee03, eb665b5102, 6abb425a75, 2aa5ca5cc2, 532c1ddbd8 (not 52ce38a2df / a90102bab1 or the
   #13307/#13308 copies); compile-bench 44189293097 was queued for it.
5. `fix/v-pre-subtree-elements` @ ca90655a96 (#13314) after #13313 — then compile-bench, PR unarmed
   "fix(compiler): keep a v-pre subtree's elements plain, as Vue does" (Fixes #13314), decision record comment.
- #13231 (lang registry) waits for the picomatch remediation; then #13262 from `lead/pug-jade-rebased` @ b52cc0d1eb
  (rebase onto main, drop the duplicated #13268 waiver, force-push to `feat/pug-jade-profile`, retarget, repin
  explain counts on top of #13309).
- #13315 (a11y on facade) needs one `benchmark.yml`; then `lint/markup-vue` @ 57977d875b (9 more rules, fallback
  92 → 84, corpus diff 0) as one PR.
- `tsrx/props-value` @ cf37001c2b after #13279 (run tsrx-mount, vize_l4 tsrx tests, vp check; post its #13131
  option comment). `perf/l4-and-inline-branch-helpers` @ 3763a47828 (halves #13277's +0.14% residual) as a perf PR.
- After #13303 merges, post the S7 progress comment on #6844 (text in the Davinci lead's handoff; summary: normal-
  script bindings come from L2 `OrdinaryFacts`, match Vue 3.5.35 `compileScript`, 456/489 identical, compile-bench
  44163416534 within +0.06%).
- Open Davinci decisions: TSRX `GeneratedName` (A keep refusing — applied / B rename user bindings / C configurable);
  accept #13277's +0.077% residual; v-pre `<component is>` keeps `is` like Vue SSR; v-pre `<slot>` stays markup in
  Vapor. #13280 (destructured setup prop used as a tag → ReferenceError) is a compiler bug to schedule.
- Queued, not started: deprecate Vue 3 paired-projection APIs; `ParsedComponent` rename with a v4 #7502 layer;
  Vapor split (#13310); `html/no-empty-palpable-content` conditional children; `vue/no-script-non-standard-lang`.
- Drafts: `pr-drafts/davinci-vpre-pr-body.md`, `davinci-linter-vue-batch-body.md`, `davinci-pr3.md`.

## Canon fixes from the misc lane
- #13269 → PR #13299, branch `fix/canon-dynamic-vbind-required-props` @ 066006799c (WIP head, compiles, untested).
  Root cause: since #2797 a `:[name]` binding is left out of the props literal (exposed by #3791 in v0.322.0).
  Fix spreads `...{ [name]: value }`. Corpus pair 44151284174: counts identical but 3 misskey files grew 233 bytes;
  the WIP head maps name/value inside the spread and drops the separate `void (…)` copies to stay byte-neutral.
  Next: run pack 2797 + full vize_canon, squash, re-run the corpus pair (no file larger than base), update body, arm.
- #13270 → branch `fix/canon-options-inherited-template-methods` @ 42fdf63a59, no PR. Only an SFC used as
  `extends`/`mixins` was unchecked; new `$AuthoredTemplateInstance` alias for imported `.vue` references only.
  Next: clippy, squash, rebase, corpus pair, PR. (Needs a real `pnpm install` in the worktree, not symlinked
  node_modules.) Script-side `this.cx(1)` in methods of SFC bases is a separate follow-up issue.

## Decisions I made during the session (all reversible)
- Compiler: SFC templates in `standard` mode keep the authored structure like Vue 3.5 (no implied `<tbody>`, no
  HTML tree-construction fix-ups — the old behaviour dropped a nested `<form @submit>` with its handler). In-DOM /
  document inputs keep the fix-ups; Vapor keeps browser repairs for now (#13310 tracks Vue 3.6's split).
- Plugin API v1: template-tree corrections that align with Vue may regenerate the frozen golden with a recorded
  note; schema/framing changes still need a versioned break.
- Lint matrix classifications (Patina correct, reference limited): imported prop contracts/collisions,
  script-setup visits, Vue 2 migration rules on Vue 2 projects, upstream default options for rules whose Patina
  default intentionally differs, require-typed-ref under assertions (#12069), dynamic emit names.
- Patina behaviour: `parser/template` duplicate attribute = error (Vue 3) / warning (Vue 2), matching `vize build`;
  `no-content-outside-block` text only; non-literal `emit(x)` → no unused-event reports.
- `docs/content/rules/all.md` exempt from the 500-line cap (generated; freshness check exists) — #13283.
- RUSTSEC-2026-0543 (chrono) waived with tracking #13267.
- Typecheck/LSP fixture isolation deferred (leak is symmetric); vuetify/lx-music via per-app `targets`;
  TS5101/5102/5107/5108 don't make a unit configuration-only; hoppscotch stays configuration-only (#13298).

## Open maintainer decisions
1. #13248 CLI output format (see above).
2. #13262: should `vue/prefer-pug-lang` be in the default preset?
3. Vite+ catalog upgrade 0.1.24 → ≥0.2.5 (the real fix for #12422; #13296 is the workaround).
4. Move tiny gate/report jobs (`test-report`, `PR source report`, `Rust product report`, `Affected tooling
   report`) to GitHub-hosted `ubuntu-24.04` (free for this public repo) so one Blacksmith slot can't stall the
   merge queue. #12219 pinned them to `blacksmith-32vcpu` deliberately (`tests/tooling/ci-report-runners.test.ts`).
5. Blacksmith capacity/billing: x86 runners stalled four times on 11-17 with nothing visible consuming them.

## Measured outcomes
- Real-project lint divergence (local, main + all lint-lane branches, 57 shared projects vs run 44093557816):
  FP 5230 → 5, FN 3645 → 2. Remaining items are Davinci HTML-tree PRs (#13304/#13308/#13306/#13309) and two
  custody cases that need re-review under eslint-plugin-vue 10.12.0 after an all-mapped CI run.
- Glyph: 0 idempotence / 0 lint-agreement / 0 parse-preservation violations over 146 projects (42,640 files) with
  #13275/#13282/#13276.
- Corpus pairs for every canon change: diagnostics IDENTICAL; the only count rise (nuxt-ui +0.5% inst) is #13254's
  generic slot-props correctness fix, with no wall-time change.
