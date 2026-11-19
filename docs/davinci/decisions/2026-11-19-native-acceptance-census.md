# One native-only acceptance census and zero-fallback verdict (#6891, #6853, #6854)

Decision (2026-11-19): every product runner writes the same differential
bundle entry, and one tool turns all entries into the published per-product
native-only rates and the zero-fallback verdict. Before this, each product
published its own Actions summary table in a different place and nothing read
the T1 and T2 rates together.

## Bundle

`tests/differential/bundle.ts` owns one layout:

```
<bundle>/<tier>/<product>/<pack>/{manifest,result,acceptance}.json
```

- `manifest.json` holds the planned `(case, target)` coordinates exactly as
  the product adapter loaded them (adapters may expand packs or attach pinned
  inputs). It binds the committed manifest by path and SHA-256
  (`sourceManifest`) instead of copying it.
- `result.json` is the adapter's `vize.differential.result` report and
  `acceptance.json` its `vize.differential.acceptance` summary.
- T1 packs come from the in-repo fixture runners: formatter `cli`, compiler
  `cli`, `api-sfc`, `api-ssr` and `api-vapor`, linter `api`, LSP `cli` and the
  verified full-workspace typechecker `workspace`. They write to
  `target/differential/bundle` (or `VIZE_DIFFERENTIAL_BUNDLE_DIR`), which the
  full tooling workers already upload with the rest of `target/differential/`.
- T2 is `real-project.ts`, which keeps its existing `T2/<product>/` layout;
  readers name that pack `real-projects`.

A reader re-validates every result envelope against the planned coordinates
and cross-checks the acceptance counts against the rows: non-completed native
states map one-to-one onto the unsupported and legacy-backed buckets, native
credit can never exceed the completed rows, and the buckets must sum to the
planned count. A pack observed twice must be identical. One tier may not mix
source revisions.

## Census and verdict

`tests/differential/native-acceptance.ts` reads any number of bundle roots (or
directories of downloaded artifacts that contain `bundle/` roots) and writes
`native-acceptance.json`, `zero-fallback.json` and `summary.md`:

- per product and policy target: native-only rate in T1 and in T2, equivalent,
  legacy-backed, unsupported and unverified counts;
- per product and dialect: verified and provisional rows and their native-only
  counts;
- the zero-fallback verdict over every product target and every dialect in
  `docs/davinci/plan/deletion-readiness-policy.json`.

A coordinate is fallback-free only when the product adapter credited it as
native-handled and its comparator verified equivalence. Legacy fallbacks are
legacy-backed plus unsupported coordinates; unverified and non-equivalent
coordinates are counted separately. A policy target with no planned rows in a
tier, or a policy dialect with no verified rows, is unmeasured and blocks the
verdict exactly like a fallback. Provisional (source-inspected) dialect labels
never satisfy coverage.

The tool is record-only by default; `--enforce` exits nonzero unless the
verdict holds. `check-deletion-readiness.mjs` stays the deletion audit: it
additionally requires per-product deletion verifiers, exact-SHA Actions runs
and the stability history.

## Where it runs

- T1: the `differential-acceptance` job of Full qualification (daily `Check`
  schedule and full dispatch) downloads the tooling workers' artifacts,
  re-verifies the full-workspace typechecker observation into the bundle
  (`typechecker-bundle.ts`), and uploads `differential-acceptance-t1-<sha>-<attempt>`
  with the merged bundle. It is a small node-only job off the PR and merge
  queue paths. A failed or missing producer leaves its product unmeasured.
- T2: `Real Project Matrix differential` publishes the census after writing
  the per-product T2 results, inside the existing `real-project-differential`
  artifact. A `workflow_dispatch` with `matrix_run_id` republishes an earlier
  full matrix run on main.

No product source or binary changes. The rates are measured, not improved.
