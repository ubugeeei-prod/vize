# Prepare original config providers for App checks

**Issue:** [#6830](https://github.com/ubugeeei-prod/vize/issues/6830)

## Failure

The main App E2E [check:all job](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055/job/116860084891)
failed before checking source in seven cases: the full Naive UI, npmX and
Voicevox checks, plus the injected-error checks for those three apps and
Misskey. Their original Vite configs eagerly import providers including
`@rollup/plugin-babel`, `@nuxt/test-utils/config`, `vite-plugin-electron`
and Misskey's workspace `estree-walker`. Those fixtures had no preparation
for these imports. The full Misskey check already used its prepared worktree,
but its injected-error check ran in the unprepared source checkout.

The same job also had diagnostic snapshot differences in Elk, Misskey and
ReKa. They are separate product investigations; installing config providers
does not justify accepting those differences.

## Decision

Share the existing [pinned lint fixture preparation](2026-10-11-6830-pinned-lint-fixture-coverage.md)
with these check cases. Each check gets an isolated export of the original
pinned source and the same frozen dependency recipe already used by lint.
Voicevox runs its authored Electron installation and copies `.env.example`;
Misskey builds its i18n package and copies its example config. The package
name, version and runtime environment reach every checker invocation.

The npmX installer uses the existing, explicitly migrated dependency graph:
the unavailable Lunaria preview is replaced by its reviewed official release
only in a separate installer export. The original source manifest and lock
remain exact, and the receipt continues to say `originalFrozenClosure: false`.
This does not claim that the missing preview tarball has become available.
Naive UI retains its existing generated lock bound to the pinned source.

Check patterns, tsconfig arguments, diagnostics and timeouts stay unchanged.
The injected-error control still runs clean, broken and repaired source and
requires the repaired result to equal the clean result. Verification after
each test rejects changes to any original blob, frozen lock or generated
config input. Check provider receipts are retained under
`.vize/artifacts/check-fixtures/` by the existing App E2E artifact upload.

## npmX generated type providers

The next [full check run](https://github.com/ubugeeei-prod/vize/actions/runs/38882529893/job/116912548686)
passed every injected-error control and every full app check except npmX's
exact snapshot. Installing npmX's providers alone changed 387 errors to 218,
including false nullable errors because its authored Nuxt type generation
had not run. Those incomplete-provider diagnostics are not accepted as a
new baseline.

The [npmX follow-up](2026-10-16-6830-npmx-authored-check-preparation.md) runs the
original lexicon and sprite tasks and root `nuxt prepare` only for checks.
It attaches the existing installer dependencies to the original workspace
directories and retains the generated input hashes. Lint preparation stays
unchanged. The original check pattern and CLI tsconfig arguments also stay
unchanged; the now-present generated app tsconfig expands the resolved
program from 222 to 374 file records, retaining every original file.

The authored `vue-tsc -b --noEmit` command passes on that prepared graph.
Clean current main `cd861aa001f7ea458f8b0147e5b9c850b9db1c87` and the repair
CLI produce byte-identical whole JSON on that same graph: 374 files, 28 errors
and no warnings, including identical ordered diagnostics and programs.
The source, generated inputs and installed provider graph remain unchanged
before and after both runs. The npmX golden therefore retains the complete
prepared answer using the existing snapshot normalization.

The remaining 28 diagnostics are pre-existing main discrepancies, not proven
project errors: 14 missing generated type-only globals (`TS2304`), seven
implicit-any rows (`TS7006`), four i18n/typed-route property errors (`TS2339`),
two component-template-ref overload errors (`TS2769`) and one query-value
assignment error (`TS2322`). The exact rows stay in the snapshot and remain
product TODOs under #6830. The full diagnostic vectors and exact-head hosted
result remain required evidence; this preparation does not resolve them.

## Validation

- Fifteen focused tests pass for environment forwarding, clean/broken/repaired
  controls, config-load failure propagation, original source custody,
  dependency migration and runtime selection.
- Scoped formatting and lint pass without warnings.
- The native loader's actual module import and Vite config projection run
  against prepared Naive UI, npmX, Voicevox and Misskey providers on Node 24.14.0.
  All original source and lock checks pass after evaluation.
- Full checker diagnostics and the remaining snapshot investigations require
  the exact-head App E2E run. The initial config-provider repair changed no
  golden; npmX's generated-provider follow-up requires the additional
  classification above.
