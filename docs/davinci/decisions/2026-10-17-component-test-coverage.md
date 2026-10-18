# Component test quantities and measured coverage

Record execution counts separately from test declarations, LLVM coverage, editor
conformance, and released-package acceptance. These snapshots identify where to
investigate; neither a large count nor a high line percentage proves completion.
Historical observations were verified on 2026-10-17 UTC. The dated 2026-10-19
update below adds retained execution evidence without relabeling older sources.

## Frozen execution snapshot: 8356b40

[Full run 38998303660](https://github.com/ubugeeei-prod/vize/actions/runs/38998303660)
at `8356b40b683caa2e38d4e5f042bcaace9339c8b4`, attempt 1, was intentionally
canceled after four independent source failures: the VS Code configuration
watcher, test-module layout guard, assertion-lint allowlist, and tooling timing
identity. The failing aggregate jobs are not additional independent defects.

Its [Rust/doctest job 117295643091](https://github.com/ubugeeei-prod/vize/actions/runs/38998303660/job/117295643091)
completed successfully. The retained Nextest inventory and JUnit agree on all
executed suites:

| Owned component                                                  | Nextest passed | Ignored in this workspace run |
| ---------------------------------------------------------------- | -------------: | ----------------------------: |
| Compiler: Armature and Atelier core/dom/jsx/sfc/ssr/vapor        |          3,078 |                             2 |
| Vapor, a non-additive compiler subset                            |            498 |                             1 |
| Canon / typechecker                                              |          2,214 |                             2 |
| Glyph / formatter                                                |            786 |                             0 |
| Patina / linter                                                  |          3,969 |                             1 |
| Maestro / LSP                                                    |          1,427 |                             3 |
| Davinci                                                          |          3,177 |                             1 |
| Croquis / CroquisCF semantic support                             |            800 |                             0 |
| Shared CLI                                                       |          1,874 |                             1 |
| Other workspace packages, including vendor and benchmark support |          1,258 |                            27 |

The workspace total is **18,583 passed, zero failed, 37 ignored**. Inventory lists
18,620 cases in 1,220 binaries; JUnit contains 1,190 nonempty executed suites. The
other 30 binaries are empty or ignored-only. Davinci means `davinci/**` plus
`tests/davinci_test_support`; its extension contract/host belong there, while
`tools/benchmarks/crates/davinci_harness` remains benchmark support.

Doctests are separate: **435 passed, zero failed, 36 ignored**, including vendor
packages. All 51 result phases were accumulated, including multiple phases for a
single Rust 2024 crate. These counts are not static attributed-function counts,
fixture counts, JavaScript call sites, or coverage percentages.

The locked Linux x86_64 workspace used Rust 1.99.1, Cargo profile `ci` and Nextest
profile `full`, with default workspace feature resolution. These are profiles,
not `--all-features` or a platform matrix. The inventory explicitly included
ignored cases; execution did not enable them:

```sh
cargo nextest list --locked --workspace --cargo-profile ci --profile full --run-ignored all --message-format json
cargo nextest run --locked --workspace --cargo-profile ci --profile full
cargo test --locked --workspace --profile ci --doc
```

The separate instrumented source-coverage job reached 18,582 passed/38 skipped,
then its coverage postprocessing was canceled. Its
[partial native-syntax artifact 12088423357](https://github.com/ubugeeei-prod/vize/actions/runs/38998303660/artifacts/12088423357)
is not a completed source-coverage qualification and does not replace the
uninstrumented execution counts above.

## Measured LLVM snapshot: 06a5157

At **`06a51573858160eca1dfe4d54a06f90812f0f6b9`**, attempt 1,
[source-coverage job 117301783478](https://github.com/ubugeeei-prod/vize/actions/runs/39000354748/job/117301783478)
completed successfully. Its separate
[Full run 39000354748](https://github.com/ubugeeei-prod/vize/actions/runs/39000354748)
**failed**: 41 jobs succeeded, four skipped, and the sole failing
[editor-host-smoke job 117301783266](https://github.com/ubugeeei-prod/vize/actions/runs/39000354748/job/117301783266)
reported `ownAttrs.invoiceTotal documentation must resolve as Markdown` in the
rich real-server VS Code host. Coverage success does not waive that failure.

Sum the LLVM per-file covered/total counts under each component's ownership roots;
do not average file percentages. All exported file totals reconcile with the
workspace LLVM totals. These denominators include measured test/helper source,
including inline test code; they are not a pure production-code-only measure.
Files absent from the export are not proven covered.

| Component              | Files |    Lines covered/total | Functions covered/total |    Regions covered/total |
| ---------------------- | ----: | ---------------------: | ----------------------: | -----------------------: |
| Compiler               |   629 | 68,535/78,690 (87.09%) |    6,038/6,677 (90.43%) | 105,615/122,058 (86.53%) |
| Vapor, compiler subset |   149 | 13,708/15,860 (86.43%) |    1,250/1,428 (87.54%) |   21,116/24,214 (87.21%) |
| Canon                  |   555 | 61,073/67,579 (90.37%) |    5,908/6,494 (90.98%) |   88,646/99,225 (89.34%) |
| Glyph                  |   100 |  9,814/10,494 (93.52%) |        839/888 (94.48%) |   15,883/17,096 (92.90%) |
| Patina                 |   577 | 57,429/62,851 (91.37%) |    6,965/7,371 (94.49%) |   86,710/95,289 (91.00%) |
| Maestro                |   430 | 44,176/52,225 (84.59%) |    4,655/5,291 (87.98%) |   67,034/79,934 (83.86%) |
| Davinci                |   813 | 68,310/74,185 (92.08%) |    7,886/8,564 (92.08%) | 103,413/114,453 (90.35%) |

This workspace command did not use `--branch`, so every branch denominator is
zero: **branch coverage is unmeasured, not 0%**. The separate nightly branch task
measures only Armature/Core/Carton; its percentage cannot describe Maestro,
Glyph, Canon, or the entire compiler.

```sh
cargo llvm-cov nextest --workspace --profile full --test-threads 8 --json --summary-only --output-path target/llvm-cov/source-summary.json
```

## Specific follow-up work

The measured LSP implementation handler had 0/94 lines reached and CSS-module
completion 6/67. Formatter literal-line handling reached 141/192 and native SFC
observation 47/65. Canon's Vapor-directive expression handling had 0/65. Inspect
the corresponding branches and existing integration tests before choosing a
regression; a file-level gap is not itself a demonstrated product bug.

The workspace also ignored Maestro's hydrated TS-42 corpus-edit check,
idle-host warm-overlay benchmark, and Pkl-runtime case. Canon's native snapshot
and Tier-L delta checks were ignored there. Dedicated native/corpus lanes may
exercise these behaviors, but their results and instrumentation must be linked
explicitly; they cannot be inferred from workspace counts or line coverage.

For editor behavior, [Editor conformance 38999781971](https://github.com/ubugeeei-prod/vize/actions/runs/38999781971)
on the same 06a source succeeded separately. Its
[VS Code job 117300280374](https://github.com/ubugeeei-prod/vize/actions/runs/38999781971/job/117300280374)
ran the original host before packaging the VSIX and TS45. Retained packets show
existing edits to `vite.config.ts`, `vize.config.json`, and nested
`packages/app/tsconfig.app.json` each changed initialize/LSP-process counts from 1
to 2 with native typechecking disabled. Each TS45 client passed 16 steps: Neovim,
VS Code and Helix used editor mode; Zed used replay mode. This does not resolve
the separate rich-host Markdown failure above. The ordinary Check's skipped
editor-host job is not credited with these executions.

## Public evidence and integrity

Download only the relevant small summary or inventory, and compare its archive
SHA256 with GitHub's artifact metadata before parsing:

- [835 Rust inventory/JUnit artifact 12087453880](https://github.com/ubugeeei-prod/vize/actions/runs/38998303660/artifacts/12087453880)
  (16,315,602 bytes), SHA256
  `630964fdefc6f7f684e9c504a5292f32681901f96f632872ca87ac0d46376a9d`.
  Read `workspace-inventory-ci.json`, `workspace-junit-ci.xml` and
  `workspace-timing.json`; the timing receipt binds source/run/attempt and commands.
- [06a LLVM summary artifact 12089167899](https://github.com/ubugeeei-prod/vize/actions/runs/39000354748/artifacts/12089167899)
  (195,887 bytes), SHA256
  `0fae48a63bd739b923229b8536a43394204a1eb25e09a61dae8a1613044f6c60`.
  Read `source-summary.json`: sum `data[0].files[].summary` counts for the
  explicit component roots above and compare all-file sums with `data[0].totals`.
- [06a original VS Code lifecycle artifact 12088068695](https://github.com/ubugeeei-prod/vize/actions/runs/38999781971/artifacts/12088068695)
  retains the before/after content-edit packets. GitHub archive SHA256:
  `497516bde7708809c3625032ce12ffcb03f2598710f484fc7fbb97609867f8dd`.

No raw archives or giant JSON exports are duplicated in the repository. These
are frozen source observations, not acceptance of a released package or a newer
main commit. Preserve the failed/canceled runs and qualify the actual repaired
source and installed release separately.

## Historical declaration inventory: 3a4ea97

This inventory pins main `3a4ea97dfde6cac13a20ff88680a507ff9b96338`. Its
**declarations** are freshly counted from Git blobs; execution and LLVM values
below belong only to the successful historical [Full 39032830916](https://github.com/ubugeeei-prod/vize/actions/runs/39032830916)
at `b3665e75cb397d096c10e4d6c87c36c00828732c`, attempt 1. They are not current-main
or released-package coverage.

| Component               |                                Declarations at 3a4ea97 |      Executed Rust pass / ignored at b366 |     LLVM lines at b366 |
| ----------------------- | -----------------------------------------------------: | ----------------------------------------: | ---------------------: |
| Compiler                |                                   3,225 Rust functions |                                 3,081 / 2 | 68,532/78,687 (87.09%) |
| Linter                  |                3,987 Rust functions; 214 JS call sites |                                 3,977 / 1 | 57,569/63,002 (91.38%) |
| Typechecker             |                                   2,241 Rust functions |                                 2,233 / 2 | 61,602/68,084 (90.48%) |
| Formatter               |                                     785 Rust functions |                                   786 / 0 |  9,814/10,494 (93.52%) |
| LSP                     |                                   1,686 Rust functions |                                 1,427 / 3 | 44,204/52,225 (84.64%) |
| Story                   |                  115 Rust functions; 347 JS call sites |                                   115 / 0 |   3,238/3,684 (87.89%) |
| Editor                  |            17 ERT declarations; 11 assertion harnesses |          Separate real hosts; Emacs 17/17 |           Not measured |
| CI / shared integration | 5,163 test call sites in 1,304 files, not CI-exclusive | 16 Tooling shards; no combined case count |           Not measured |

At b366 the workspace ran 18,567 Nextest cases successfully, with 37 ignored;
434 passed/36 ignored doctests are separate. Story's Musea JS invocation ran
308 passed/31 skipped cases. Conditional browser skips are not browser passes.
Static extraction masks Rust strings/comments and uses TypeScript test-call ASTs;
fixture/vendor/generated directories and shared CLI ownership remain separate.
Macro expansion, loops and cfg explain why declaration and runtime counts differ;
do not divide one by the other. LLVM includes test/helper code. Glyph/Maestro
branch coverage remains unmeasured, regardless of their line percentages.

Historical public evidence has another unit: published 0.463's retained Linux run
[39049959377, attempt 2](https://github.com/ubugeeei-prod/vize/actions/runs/39049959377/attempts/2)
executed the original 32 LSP transactions plus four 18-reply atomic sequences,
with 36 clean process lifecycles. These are protocol contexts, not 104 independent
unit tests or code coverage. Its separate gallery succeeded; trusted VRT still
failed at the first Left iframe. Preserve that whole-campaign failure.

## 2026-10-19 update: latest complete retained quantity sample

The latest **complete workspace inventory retained for this comparison** is
`793fd6fbb80c40505af9510b5afdd00ae79d1d94`,
[Full 39132001467](https://github.com/ubugeeei-prod/vize/actions/runs/39132001467),
attempt 1. Its inventory and JUnit match every nonignored test name:
**18,621 passed, zero failed, 37 ignored**, from 18,658 registered cases in
1,208 binaries (1,182 nonempty executed suites). The whole workflow nevertheless
**failed** in the Editor/aggregate boundary; successful Rust results do not waive it.

| Product ownership                                         | Executed Rust passed | Ignored | Other measurement unit             |
| --------------------------------------------------------- | -------------------: | ------: | ---------------------------------- |
| Compiler: Armature and Atelier core/dom/jsx/sfc/ssr/vapor |                3,087 |       2 | 331 registered binaries            |
| Patina / linter                                           |                3,991 |       1 | 103 registered binaries            |
| Canon / typechecker                                       |                2,252 |       2 | 61 registered binaries             |
| Glyph / formatter                                         |                  789 |       0 | 82 registered binaries             |
| Maestro / LSP                                             |                1,429 |       3 | 6 registered binaries              |
| Musea / Story                                             |                  115 |       0 | 3 registered binaries              |
| Editor integrations                                       |         Not combined |       — | Separate real-host observations    |
| CI / shared integration                                   |         Not combined |       — | 16 successful mixed-tooling shards |

The six product rows total 11,663; the remaining 6,958 workspace tests belong to
shared CLI, language stages, parser, vendor and support packages. The separate
compiler cohort's 6,983 passes at source `f6b9e8bf` include shared packages and
must not be added to the compiler-product row. Likewise one Unicode regression
contains 15 authored SFCs × two providers × two modes: 60 child processes are
not 60 additional Rust tests. Binaries, declarations, fixture rows, protocol
replies and editor assertion strings are distinct units.

These counts apply only to that run's package/feature resolution, not an
all-feature/platform matrix or newer main. They do not replace the historical
835/06a/b366 observations or provide new LLVM percentages. LLVM includes measured
test/helper source; branch coverage was not workspace-instrumented.

The [793 inventory/JUnit artifact 12138496683](https://github.com/ubugeeei-prod/vize/actions/runs/39132001467/artifacts/12138496683)
contains 18,706,250 bytes, archive SHA256
`97b4facb9e6aa21be52c81572339534ace4687f1fa3fa677f2bf574730c961a6`.
The retained inventory SHA256 is
`7f64d7478b08a4e8b9c1471ccfd82048e1e15f12abf87ceb946de2342cdf447f`;
JUnit SHA256 is
`66ad87a730d8404e17d8ae09748ee31f07ed972717a2057cb5a597b6ea9e7583`.
No new inventory, native build, download or coverage run was made for this update.

### Next: whole implementation results across an unsaved edit

Under existing [#3953](https://github.com/ubugeeei-prod/vize/issues/3953) and
[#3952](https://github.com/ubugeeei-prod/vize/issues/3952), extend the existing
`tests/tooling/lsp-implementation-capability.test.ts` scenario. It currently takes
`firstLocation` for one implementation and checks only its start position. Add
an interface with two implementations across authored SFC/TSX files; compare
**every** returned URI and complete UTF-16 range, then change/remove/restore one
implementation in an unsaved CRLF/Unicode buffer. Reject duplicates, stale
locations and generated-only URIs anywhere in the response. Preserve real
checker provenance and require the backend in the qualifying lane.

Existing native SFC/TSX round trips already cover implementation navigation;
0/94 historical instrumented handler lines do not mean the feature has no test.
The missing case is the complete multi-target response through edits and a
released/packaged-host replay, not another non-null smoke assertion. Do not
recreate #9409's already-added CSS-module unsaved-revision source scenario.

### Next: run existing formatter semantics through the published CLI

Under existing [#6098](https://github.com/ubugeeei-prod/vize/issues/6098), reuse the
frozen `authored-crlf-auto` continuation row and `deep-template-literal`
expression-width row. Source tests already verify complete CLI outputs,
check/write/fixed points and pinned stock DOM/SSR/event contracts. The missing
boundary is a **fresh installed public CLI**, which the current normal public
acceptance workflow does not invoke for formatting. Check package/archive/SRI
and executable identity, original check refusal, complete output/config bytes,
second/third-pass identity and the same whole DOM/SSR/runtime-value observations.
Use the original fixtures and independent contracts unchanged; a successful
source binary cannot substitute for the published executable.

Formatter is not weak by aggregate test quantity. Historical lower-reached
`literal_lines.rs` (141/192) is a search hint, not proof of a missing runtime
oracle: semantic source tests already exist. The completed #9422 private preset
constructor matrix also cannot prove an unchanged public CLI integration.
Keep #6098/#6847/#6875 open beyond this bounded evidence improvement.

### Reproduce and compare the evidence

After the normal workspace install, use the existing inventory on a checkout of
the source being compared:

```sh
mkdir -p target
node tools/benchmarks/scripts/test-inventory.ts --root . --json target/test-inventory.json
cargo nextest list --locked --workspace --cargo-profile ci --profile full --run-ignored all --message-format json
vp run --workspace-root coverage:source
```

The inventory command reports static declarations using its own broader text/fixture
heuristics; it is an independent cross-check, not an exact replay of the Git-only
masked-Rust/TypeScript-AST counts above. The latter commands reproduce the
runtime inventory and LLVM recipe when run in the qualifying CI environment;
retain their exact source SHA, features, commands and artifacts. Their results
must never be relabeled as measurements of another source or public release.

The analysis that produced this snapshot was read-only: it reused retained
reports and inspected source without creating Issues/comments, executing native
or Cargo builds, dispatching CI, installing public packages or running browsers.
The two follow-up cases remain proposals pending separate review and execution.
