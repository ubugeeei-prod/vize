# Per-file Pinia availability in editor linting

Issue: [#7827](https://github.com/ubugeeei-prod/vize/issues/7827).

## Contract and repair

The reported project contains a plain imported `useCounterStore` composable and
does not install Pinia. The CLI already disables
`ecosystem/pinia-prefer-store-to-refs` when no ancestor of the linted file contains
`node_modules/pinia/package.json` (`commands/lint/entry_rules.rs`). Maestro omitted
that per-file condition and therefore emitted the name-based rule's warning.

Apply the same availability condition when resolving Maestro's private Patina
options. Resolve the original file URI; a sibling project's package does not
enable the rule. Recheck the filesystem for subsequent diagnostics so removal
does not retain a stale enabled rule. Non-file documents keep their existing
unknown-project behavior. Existing explicit rule disabling still takes priority.

The Patina rule's matching/import policy, CLI behavior, presets, severity,
diagnostic text/ranges/URLs, native routing and parser stages stay unchanged.
This fixes the original no-Pinia project. Distinguishing an imported non-Pinia
composable when Pinia is also installed requires a separate semantic change and
is outside this availability repair. No public API or instruction ceiling changes.

## Authored complete regression

`tests/_fixtures/differential/lsp/pinia-availability/` retains both complete
original issue code blocks, pinned by SHA-256. `references.json` independently
authors the complete existing warning and a `storeToRefs` repair; these references
are not captured from current runtime output. The positive fixture uses an
authored `defineStore` module and a physical ancestor package identity, without
installing or executing packages.

`lsp-pinia-availability.test.ts` requires the existing exact source-build receipt
and launches real `vize lsp` over stdio. It compares whole versioned publications
for the original absent-Pinia open, available-Pinia warning, repaired change,
warning restoration and package removal. A second real session checks complete
open/change publications with the rule explicitly off. Three source-built CLI
runs preserve the original complete empty JSON results and both input files for
default, ecosystem and opinionated presets. Passive wire capture retains the
actual sessions. Rust controls cover ancestor/sibling/missing identities, package
removal and non-file documents without adding SFC parses.

The new notification corpus is separate from the immutable historical shared
request-response manifest. The PR tooling selector includes it; compiled source
Actions, protected full suites/104 probes, actual merge and release remain
required. Source inspection grants no runtime or native/history admission credit.

The reporter's verified public GitHub identity is `ubugeeei`, ID `71201308`, with
no public email. The meaningful source commit and PR supply
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.
Actual final trailers and primary authorship must be checked separately.

## Existing product fixture E2E audit

The independently retained original-fixture audit is now recorded alongside this
concrete missing lint notification regression. It qualifies exact historical
sources rather than this new head or release publication.

[Full Check 37200530806](https://github.com/ubugeeei-prod/vize/actions/runs/37200530806)
passed on `0db778dffa8b2209f10a88e61804b7c0abba0a6a`. Its unsharded tooling job
`111431130411` uses receipted CLI `vize 0.433.0`, SHA-256
`4b1d34924a0773e5467e07f7fdf5fbfac271a602bbf81c6dbafc3dbd06e9beef`;
artifact `11302568860` retains the product reports and observer receipts.

| Product | Actually executed original contract                                                                                                                                                                                                                                                                             |
| ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| lint    | 44 public Rust API cases, two fresh processes each, complete original Insta bodies and stderr (88 calls). This does not prove all 44 through CLI.                                                                                                                                                               |
| fmt     | 12 CLI cases with three chained whole-byte/fixed-point passes (36 calls); original API packs have 276 whole-output/fixed-point cases, 21 typed-error cases and three internal observations (300 rows/852 calls). Five historical CLI plans retain all four calls each, including three expected check failures. |
| compile | One whole-module SSR CLI fixture, plus five complete original public API results per DOM/SSR/Vapor cohort in two fresh processes. Fifteen API fixtures are not fifteen CLI fixtures.                                                                                                                            |
| LSP     | Twelve source-built real stdio sessions, 19 complete original response values: documentLink 1, onTypeFormatting 4, foldingRange 5, documentHighlight 8, hover 1.                                                                                                                                                |

[Protected Check 37198344364](https://github.com/ubugeeei-prod/vize/actions/runs/37198344364)
passed on `61c975f8889a002a1b53265d86d0985447ffab63`; its four tooling jobs also
execute those contracts. The protected typechecker report has all 32 original
ordered diagnostic rows matching `batch-start-diagnostics-v1`; four successful
workers' JUnit counts 3/6/4/2 cover all 15 registered bodies without fixture
failures or skips. Artifact `11302635808` ZIP SHA-256 is
`e70cbc3a4246975a207e53a7061f6737925e2643beda2282de3aa8034ea82ee5`.
Its contract leaves blockType/exitCode/success unbaselined and omits end positions,
related information and raw backend diagnostics. The release Rust job also
passes all 15 bodies but does not produce that protected aggregate capture.

All 12 historical LSP sessions initialize with lint/typecheck disabled and retain
empty diagnostic publications. They prove their registered editor responses,
not all historical typed/lint notifications over RPC. This repair and #7833 add
separate genuine notification regressions; transporting every existing typed/lint
history input with independently authored complete notifications remains TODO
under #6883. API/batch observations and synthetic transport tests supply no
automatic CLI/RPC credit. Source PR selection defers old runtime inventories;
the merge group preserves full execution. The release tooling run reports 6,125
tests, 6,070 passed and 55 unrelated skips, with zero failed/cancelled tests.
Neither this audit nor the new corpus closes #6883 or enables native defaults.
