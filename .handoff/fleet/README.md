# Davinci fleet handoff — paused 2026-11-19 (JST evening) for a machine switch

Maintainer's order: "davinci を本気で完成させてくれ．ただし動作もパフォーマンスも絶対にデグレするな" and
"PR には必ず計測結果を載せてくれ．auto merge".

- `fleet-brief.md`: the rules every agent follows (hard rules, toolchain, CI gates, disk protocol,
  later maintainer rules at the end). Hand it to every new agent.
- `t2-t6-plan.md`: the evidence-based slice plan for T2–T6, lanes A–F, with the maintainer decisions
  appended at the end.
- `ci/`: the ci-tiers analysis scripts.

Gates: `node tools/commands/davinci/completion-gates.ts --issues`. This moves to
`tools/commands/levels/` after the codename rename lands. At pause: **19/45** (17 at start).

## Rules decided on 2026-11-19 (all final)

- No regression on any metric. Instruction counts AND binary size are judged on the **release profile
  (fat LTO, 1 CGU)**, strictly (every row ≤ base, size not growing). ci-opt is informational.
- **Move/rename-only PRs** pass on identical name-normalized optimized LLVM IR (`ir-identity.ts`, #13410),
  byte-identical output, and no release-size growth.
- A **listed fix** may raise only the rows whose fixture exercises the fixed construct, backed by a callgrind
  diff showing only the fix's own work. The instruction ratchet still needs a mechanism for that, e.g.
  a raise entry keyed to the waiver id (lane C needs it for J3b).
- No metric trade-offs (e.g. −0.1% Ir for +128 B is rejected).
- Codenames: **hard rename** of public names, no aliases. Refined gate measure accepted. List them under
  "Breaking changes".
- `vize_atelier_sfc` stays as the public compiler shell after legacy deletion, holding wiring only. Legacy
  instruction-row names are kept and repointed at native paths.
- `levels.query` = complete `vize_l2::query` API plus the import-rule test (Q6). TSRX = implement `@try`,
  scoped styles and server modules.
- JSX `c && <A/>` keeps today's bytes. Source maps break on JS line terminators (LF/CR/CRLF/U+2028/2029) in
  every backend.
- Linter syntax rules run on a **pure L1 engine** (promote `native/`); the L2 facade is for semantic rules.
- Glyph: delete `native_doc/`. OXC printing of an L1-profile parse counts as "formats from L1".
- `SfcDescriptor` moves to `vize_l1::container::vue` with identical output.
- CI: keep the runner pool, cut jobs per PR, keep the 3/6-min target and fund the redesign.
- Pinned Debug/evidence renames go through ONE shared rename map plus a spell-back helper (#13412).
- Disk: ≥60 GB free before any build. Release builds only via CI, or locally under
  `mkdir /private/tmp/claude-501/vize-release-build.lock`.

## Decisions still open (ask the maintainer)

1. May a bug fix grow the release binary by a few bytes? (#13414, #13413, #13386)
2. Glyph F4: in 11 places the old tag scanner and L1's lexer read tags differently (`<1x>`, `href=/x`,
   `<div$x>`…), and in at least 5 the scanner disagrees with the compiler. Take L1's reading as listed
   fixes, or keep the scanner's output?
3. CI redesign: may build receipts bind to the Rust sources instead of the whole checkout, so prebuilt
   artifacts can be reused? Do tsgo-required Type checker tests stay on PRs (#11489)?
4. The v1-supported-matrix job fails by design until v1 qualification is complete (0/420 rows). Drop it from
   the daily green signal, or give it an explicit "expected incomplete" mode?
5. Instruction harness: lane B suspects fixture alignment (`include_str!` data moves) raises untouched
   release rows when static data shifts. Align the fixtures in the harness, which re-records baselines?
   Lane B's probes 44518541629 (base) and 44518545933 (v8) record each fixture's address mod 64 to test
   that theory.

## Tool status

- `ci/shipped-profile-measure` (b78b6a06c9, no PR): the release-profile measure workflow. **Its A/A failed**
  (run 44510732532): the first fat-LTO build on a fresh runner differs on 26 rows. Fix it with a discarded
  warm-up build, or find the root cause, then re-validate before anyone trusts it. Until it's on main,
  dispatch via `--ref perf/pb-rm-validate`.
- #13410: IR-identity checker (auto-merge on). Its CI runner is branch `bench/levels-ir`. IR run 44517575035
  for #13412 failed without a report, probably a script bug.

## Tracks: state at pause (every branch has a `.handoff/*.md` except where noted)

(filled in below)

### Priority 0 — behavior regressions on main (do first)
- **The daily full Check has been red since 2026-10-18** (last green schedule run 39014647393). Triage is in
  `.handoff/main-health-triage.md` on `ci/full-check-alert` (9e2a064e49). Real regressions:
  - patina markup allocations 0→1 and 7→8 per file (suspect #13351);
  - 3 LSP regressions (window 11-11..11-13): generated helpers like `__vizeDirective` offered as completions;
    completion data and the `defineProps` snippet dropped; call-hierarchy range only covers the name;
  - coverage case `array with as const` (suspect #12639, may be deliberate).
  The rest are stale pins (fix via official tools) and test bugs. The alert guard (`check-full-alert.yml`) is
  committed, but its PR isn't open yet.
- **Fix-history regression fixes:** #13414 fixes #13376; #13413 fixes #13383 and pins #13385, which is not a
  regression. Both are open with auto-merge off and need measurements. Base instruction run 44517750227 is
  shared, heads are 44517661409 and 44517582455. #13384 (LSP global components) is reproduced, not fixed:
  `fix/lsp-global-components`. #13386 is on `fix/aliased-imported-macro-type` (untested). #13411 (test-only)
  closes #6880 when it merges.

### Merged this session (selection)
#13340 #13361 #13374 (−941 KB release) #13362 #13364 #13365 #13366 #13368 #13369 #13371 #13372 #13373
#13375 #13377 #13380 #13381 #13388 #13393 #13394 #13396 #13398 #13402, plus whatever landed from the queue
after the pause.

### In the merge queue / open with auto-merge at pause
#13395 (S6c dead Croquis bridge), #13397 (lint substrate ledger), #13399 (FX1 record), #13400 + #13382 (JSX
rows), #13403 (Vue L1 relocation), #13405 (L0 renames), #13410 (IR identity).

### Open, auto-merge OFF (evidence missing; the reason is on each PR)
- #13379 Q2 L2 query: needs release-profile Ir (`tmp/q2-fat-head`, base run 44489008705, `ic_compare.py`
  in the feat/6871-query-vue handoff).
- #13407 Glyph F2+F3: needs release-profile Ir (base 40b66e3660, head bc0ee89295).
- #13390 L4-1 one EmitDocument: release rows still up (≤ +0.11%), see the lane-B handoff.
- #13409 JSX J2b part 1: release-clean; needs the head twin 44514658304 and the perf table, then auto-merge
  after #13382.
- #13412 L1 component rename + the shared evidence rename map: IR run 44517575035 failed without a report;
  rerun. Release size identical (run 44517626548).
- #13338 style decls: WIP on `davinci-inputs-style-decls-size-wip`.
- #13401 CI tooling selection: ejected from the MQ (tooling shard `docs-heading-hash-browser`); read job
  135516363555.

### Branches waiting to become PRs
- Lane A: `feat/6871-query-imports` (Q6), `feat/6871-query-vue` (Q3; handoff covers both),
  `feat/6844-setup-macro-facts` (S6c core analysis; native list sees through `as X` = listed change).
- Lane B: `davinci/l4-ssr-runtime-helpers` (L4-2: byte-identical, ci-opt 12 SSR rows down; the main handoff
  plus `.handoff/laneb-tools/`), `davinci/l4-one-emit-document-v8` (faster escape loop for #13390, unmeasured),
  `davinci/sourcemap-js-line-terminators` (ledger W-MAP-1..3; open it after L4-1).
- Lane C: `feat/jsx-underscore-modifiers` (J3b, needs the ratchet raise mechanism),
  `perf/l1-to-l2-line-comment-memchr`.
- Lane E: `feat/6876-fix-edit-sets` (FX2: +445,888 B release, needs a symbol diff; reshaped LN2 plan).
- Formatter: `refactor/glyph-delete-native-doc-wip` (F5), `fix/glyph-literal-lines-after-leading-expression`,
  `refactor/glyph-f4-tags-on-l1-wip` (blocked on decision 2).
- levels-inputs: `davinci-levels-l1-components` (#13408 successor plan), `davinci-levels-l1-style-vue`
  (`css/`→`style/` move, now judged by IR identity).
- product-deps: `refactor/sfc-descriptor-l1` (probe 44492961984 on `tmp/pd3-probe`), `handoff/product-deps`
  (inventory).
- perf-backlog: `ci/shipped-profile-measure`, `perf/atelier-vapor-lower-stress-wide`,
  `perf/atelier-ssr-slot-name`, `perf/l1-to-l2-cold-instances`.
- ci-tiers: `ci/tooling-planner-cache-6830` (+ `.handoff/ci-tiers-6830-data/`).
- main-health: `ci/full-check-alert`.
- codenames: `refactor/remove-codenames` @ 2ae2bed148 (`.handoff/codenames/` has the replay scripts and PR
  body). Open: full check 44517440281 vs main 44465592435, IR run (`dispatch-ir.sh` +
  `strings-through-rename.ts`), release size, A/A. Land it in a merge window after the in-flight PRs.

### Gate numbers at pause
- Native-only acceptance: compiler 0, linter 23/724, others 0 (templates on L1/L2: DOM 464/467, SSR 466/466,
  Vapor 278/465; the SFC container and script are still legacy).
- products.dependencies: 36 legacy edges on main (35 after `refactor/sfc-descriptor-l1`).
- levels.inputs owed: 117 on main (about 49 once #13403/#13405/#13408-successor land).
- PR Check p50/p90: 12.3/19.3 min before the CI fixes; no fair after-sample yet (fleet queueing).

### Resume recipe
1. `git fetch`, rerun the gates and `gh pr list`; see what the queue merged.
2. Priority 0 first. Then make the release-measure workflow pass A/A and land it, because every PR needs it.
3. Restart the lanes from their handoffs with `fleet-brief.md` as the common brief; keep ≤ ~8 agents (CI is
   the bottleneck).
