# Native-only acceptance rate publication (#6853)

The shared result envelope fixes every planned `(case ID, target)` coordinate.
The acceptance reporter uses those coordinates as its denominator, with one
count per target and one total per product. It validates the manifest digest and
expected source revision before counting. The machine-readable artifact is
`vize.differential.acceptance` v1 with `scope: "registered-fixtures"`,
`plannedCases`, `total`, and `targets`. Each target has `planned`,
`nativeHandled`, `nativeEquivalent`, `unsupported`, `legacyBacked`, and
`unverified`. A failed, skipped or unverifiable run remains in `planned` and
counts as `unverified`. Only the shared whole-product native provenance
classifier may increase `nativeHandled`; each adapter supplies its exact
required stages and verifies the build receipt and observation. Equality needs
an independently verified product comparison in addition to native handling.

The formatter CLI's five planned `fmt` inputs yield **0/5 native-only**, five
unsupported, and zero equivalent. The exact-head Actions execution publishes
the JSON at `target/differential/native-acceptance.json` and a table in the job
summary. The source-built linter observer now publishes its independently
validated report and acceptance artifact under
`target/differential/linter-api/`: **10/59 native-only**, all ten equivalent,
49 explicitly unsupported, and zero legacy-backed or unverified rows. Its
Actions summary uses the same shared reporter and denominator.

These rates measure registered fixtures only. They do not certify complete
dialect, T1/T2, or product coverage. Compiler has a separate fixture-scope gate
in [#7080](https://github.com/ubugeeei-prod/vize/pull/7080), with Croquis-backed
compiles credited zero. No rate is fabricated for unregistered coverage.

## Current publication audit (2026-10-21)

At main `d8eb81336e018fa4be4e337ddeb2d14e9d492d4f`, all five registered
product reporters have explicit denominators. The source-built compiler
observers publish one acceptance artifact and Actions summary per target:
DOM 0/8, SSR 0/5 and Vapor 0/5, with every row unsupported. Formatter publishes
0/21, also all unsupported. Linter publishes 10/71 native-only and equivalent,
with 61 unsupported; four additional input-only registrations remain unmeasured
because they do not yet have executable API oracles. The full-workspace
type-checker verifier publishes 0/62, and the source-built LSP observer publishes
0/37; both keep every row explicitly unsupported. These values grant no credit
to a legacy-backed fact.

The compiler and LSP execution tests now persist their previously assertion-only
acceptance objects beside the raw observations and append the shared Markdown
table to the Actions summary. The type-checker verifier already persisted JSON
and now appends the same summary. Denominators are derived from the validated
manifests where fixture growth is expected, while the inventory tests retain
the exact current census.

This closes the publication asymmetry only. Complete product inputs, dialects
and T1/T2 plans are not registered, so their acceptance remains unmeasured and
#6853 stays open.

Next: bring the complete dialect/input plans into the shared manifests; require
source/build-bound runtime observations and product comparators for every
product; publish one combined per-product table across the complete T1/T2
corpus. #6853 remains open until that work is done.

## Registered manifest dialect measurement audit (2026-10-29)

The manifest census at `cfb80f5f72f06751d9a20475646ad5118382edfa`
contained five coverage-state / measurement-unknown rows. Source inspection of
their immutable inputs now records provisional dialect measurement for exactly
those rows:

| Product     | Case                                                                   | Pinned source and SHA-256                                                                                              | Dialects                                  | Selector                                               |
| ----------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------ |
| compiler    | `compiler/sfc/jsx-vapor-runtime-slots-6889`                            | `jsx-vapor-runtime-slots-6889/Layout.vue.txt`<br>`d0efd072e39bb5c7979ccf95061f62811d565883764dce352e07e2a2cb3aebf2`    | `vue3-sfc`, `html-template`               | `compiler-cli-ssr-v1 default Vue 3 dialect`            |
| compiler    | `compiler/sfc/scoped-checkout-stability-10067`                         | `scoped-checkout-stability-10067/Layout.vue.txt`<br>`1a3f415e27c6a2a541da2b717ddd5b1b3e4bf8b9b3770e7975a6e39990f47526` | `vue3-sfc`, `html-template`               | `compiler-cli-ssr-v1 default Vue 3 dialect`            |
| formatter   | `formatter/sfc/plain-style-nested-comments`                            | `sfc-plain-style-nested-comments/App.vue`<br>`3bb6591e44487de32067d123b9dfdf303a4381c53f0c8bcebf3026c23b4021ed`        | `vue3-sfc`, `html-template`, `css-scoped` | `default Vue version with --no-config`                 |
| formatter   | `formatter/sfc/split-v-pre-indentation`                                | `sfc-split-v-pre-indentation/App.vue`<br>`7ee5086667bb92cab2e96bbf0e19507182fd8643cd9c9194efd9a6d391b45660`            | `vue3-sfc`, `html-template`, `v-pre`      | `default Vue version with --no-config`                 |
| typechecker | `typechecker/nuxt-plugin-injection-types/authored-dependency-identity` | `nuxt-plugin-injection-types/project.json`<br>`2ad499a9665600b7ce468b7568641491f1262c3010dc40d5d4c53b2b5b775412`       | `ts`                                      | `batch-typechecker-fixture-v1 pinned authored sources` |

The typechecker measurement opens the pinned `project.json` container, checks
its schema and closed authored `files` map, and derives `ts` from those virtual
source names. The ratchet verifies both the manifest-declared hashes and the
current source bytes.

| Product     | Measurement unknown |   Provisional | `unknownReadinessCases` |
| ----------- | ------------------: | ------------: | ----------------------: |
| compiler    |               2 → 0 |       37 → 39 |                 39 → 39 |
| formatter   |               2 → 0 |       23 → 25 |                 25 → 25 |
| linter      |               0 → 0 |     110 → 110 |               110 → 110 |
| LSP         |               0 → 0 |       69 → 69 |                 69 → 69 |
| typechecker |               1 → 0 |     214 → 215 |               215 → 215 |
| **Total**   |           **5 → 0** | **453 → 458** |           **458 → 458** |

`source-inspected` means provisional measurement only. All five rows retain
`readiness: "unknown"` until verified execution. This audit changes no
`nativeHandled`, native-equivalence, production-lane, threshold, release-scope,
or published acceptance-rate claim. #6853 remains open for those independent
execution and whole-product acceptance gates.
