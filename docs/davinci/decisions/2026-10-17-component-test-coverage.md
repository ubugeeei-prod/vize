# Component test quantities and measured coverage

Record execution counts separately from test declarations, LLVM coverage, editor
conformance, and released-package acceptance. These snapshots identify where to
investigate; neither a large count nor a high line percentage proves completion.
Observations below were verified on 2026-10-17 UTC.

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
