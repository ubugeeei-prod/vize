# Prepare Nuxt and ReKa fixtures before the App check

**Issue:** [#6830](https://github.com/ubugeeei-prod/vize/issues/6830)

## Decision

The App E2E `check` suite prepares a fixture the way its own project does
before Vize checks it:

- Elk, Nuxt UI and VueFes run their authored Nuxt preparation, so the types
  that modules register through Nuxt's `prepare:types` hook are part of the
  checked program. The preparation uses the Node 22 runtime those projects
  declare, scoped to the child process; the checker keeps its own runtime.
- ReKa resolves its pinned `tsconfig` providers (TypeScript 5.8.3, Vue 3.5.17
  and `@vue/tsconfig` 0.7.0) from `tests/_fixtures/reka-tsconfig-provider`.

## Reason

Without preparation the checked program was missing generated declarations.
VueFes reported eleven extra SFC errors for `import.meta.vfFeatures`, whose
augmentation is generated into `types/feature-flags.d.ts` and
`types/nitro-feature-flags.d.ts` during `nuxt prepare`. Those diagnostics came
from the test setup, not from the type checker.

## Scope

This change carries the preparation helpers, their tests, the three check
snapshots and the workflow step from
[#8615](https://github.com/ubugeeei-prod/vize/pull/8615). It does not carry:

- **npmX.** Its pinned commit depends on a `pkg.pr.new` tarball that now
  returns 404, so it cannot be installed. Preparing it needs the fixture pin
  to move, and that pin is referenced by other frozen lint fixtures. npmX
  keeps its current check until the pin moves in its own change.
- **Canonical corpus input epochs.** They existed only to authenticate the
  changed corpus file count after the npmX pin moved.
- **Test-only native reference receipts** for ReKa and VueFes and the
  matching entries in the changed-file allow-list. They bind hashes and byte
  lengths of provider files rather than checking type checker behaviour.

Golden diagnostics, budgets and the injected-error controls are unchanged.
