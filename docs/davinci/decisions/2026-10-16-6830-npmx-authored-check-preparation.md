# Prepare npmX's authored Nuxt type providers

**Issue:** [#6830](https://github.com/ubugeeei-prod/vize/issues/6830)

## Failure and classification

The [App E2E check job](https://github.com/ubugeeei-prod/vize/actions/runs/38882529893/job/116912548686)
successfully loaded the original npmX config after the
[config provider repair](2026-10-16-6830-app-check-config-providers.md).
All injected-error controls passed, but the npmX snapshot changed from 387 to
218 errors. Both answers contained exactly the same 222 file records.

The complete diagnostic comparison removed 200 rows and added 31. Installing
the real providers resolved 74 missing-module errors and 102 dependent
implicit-any errors. Twenty-one unknown `defineOgImage` errors changed to
suggestions for the newly visible `definePage`; three other diagnostics also
disappeared. The additional rows were three missing `RouteNamedMap.code`
properties, four nullable `pdsUsers` errors, and three search-provider index
errors. These are not a sound replacement baseline: npmX's authored Nuxt
preparation had not run, and its generated tsconfig references, imports and
typed routes were absent.

In particular, the Nuxt fallback declares `useLazyFetch` as `Ref<T | null>`
without using `options.default`. npmX supplies `default: () => []`. Its real
Nuxt provider has a default-value overload, so the four new nullable errors
must not be accepted as source errors.

## Decision

Only npmX check preparation runs its original `generate:lexicons` and
`generate:sprite` tasks, followed by the original root `nuxt prepare` command.
The config, enabled Nuxt modules, source, manifest and lock remain unchanged.
The existing lint preparation does not run these generators. The check also
attaches the already installed workspace dependencies to their corresponding
original `cli` and `docs` source directories; attaching only the root
`node_modules` leaves real CLI imports unresolved.

Run the commands from the authenticated installed packages with the selected
Node executable. `pnpm 11 exec` attempts to reinstall when it compares the
original manifest with the deliberately separate migrated installer. Direct
execution preserves that reviewed graph while running the same authored
commands. The generator tasks keep the existing `--no-cache` workaround for
nested Vite+ file-spy hooks.

The receipt records executed argv, provider manifest and entrypoint hashes,
workspace dependency locations, and every generated prerequisite and `.nuxt`
input hash. Missing required generated inputs or command failures stop the
check. Verification retains the original 1,162 source blobs, original frozen
lock, effective installer inventory and generated inputs.

The unavailable Lunaria preview is still replaced only in the separate
installer by the previously reviewed official release. The receipt retains
`originalFrozenClosure: false`; this does not reconstruct the unavailable
preview package.

## Validation

- The original tasks and Nuxt preparation run successfully on Node 24.14.0
  with the reviewed installed graph. All original source blobs remain exact.
- The authored `vue-tsc -b --noEmit` command passes after attaching both
  workspace dependency directories. It uses npmX's own pinned
  `typescript-native-bridge@6.0.3-bridge.9.tsgo.7.0.2` override and
  `vue-tsc@3.3.9`.
- The freshly built repair CLI reports 28 errors over 374 files. All prior
  222 files remain; the generated app tsconfig additionally brings in 152
  source and generated dependency files. All 31 incomplete-provider
  additions disappear.
- Clean main `cd861aa001f7ea458f8b0147e5b9c850b9db1c87` and repair
  `99caba071ada01a251c3618683fcfe44701ecfa0` produce byte-identical whole
  JSON, including all file objects, ordered diagnostics and programs:
  SHA-256 `c1da53bc27db099d9bab5f8b16e75662041cea2fe99440ec37bb753e1d475a52`.
  Before/after hashes retain 1,337 source/generated files and three links,
  plus 71,306 installed provider files and 6,522 links. Runtime cache
  directories are excluded explicitly from that graph comparison.
- The npmX golden is the complete prepared answer using the existing
  `stringifyDiagnosticSnapshot` and `assertSnapshot` mechanisms. Its SHA-256
  is `c0ada2517897359ee9be573ee61e786430d06c5af145d481515e50da92d02fe4`.
  Every old file remains in the same relative order. The independent main
  answer also matches that snapshot; no diagnostic rows are filtered.
- A fresh export prepared by the new helper retains 169 generated input
  hashes and all 1,162 original blobs through the existing clean, broken and
  repaired checker passes. The broken source reports the injected `TS2322`;
  the repaired file has no diagnostics.
- Six focused tests pass on Node 24 and Node 26, covering actual subprocess
  ordering/runtime selection, workspace resolution, provider immutability,
  failed generators, missing generated imports, and the existing injected
  checker environment contract. Scoped formatting and lint pass.

## Remaining product discrepancies

The 28 residual rows already occur on current main, and the authored
`vue-tsc` oracle reports none. They remain visible in the exact golden and
are not declared valid project errors. Follow-up work under #6830 must cover:

| Code     | Rows | Observed surface                                                                                 |
| -------- | ---: | ------------------------------------------------------------------------------------------------ |
| `TS2304` |   14 | Generated type-only globals for comparisons, virtual lists, package managers and timeline charts |
| `TS7006` |    7 | i18n iteration, facet values and search callbacks                                                |
| `TS2339` |    4 | i18n `locales`/`setLocale` and the typed `noodles/[slug]` route                                  |
| `TS2769` |    2 | Component template refs passed to element-consuming composables                                  |
| `TS2322` |    1 | `Brand/Customize.vue` query value assignment                                                     |

The exact-head hosted App E2E result remains required. This preparation does
not filter diagnostics, alter the check's patterns or extend its deadline.
