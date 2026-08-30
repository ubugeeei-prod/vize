# Third-party fixtures

Projects under `_git/` are read-only upstream test inputs pinned as Git submodules. They are not
covered by Vize's license. Each project's revision, SPDX expression, and preserved license files
are recorded in `vue-ecosystem-fixtures.json`.

Do not patch fixture source to make a Vize test pass. Fix Vize itself, then rerun the same pinned
revision. When adding a fixture, keep its upstream license files in the submodule and declare every
license that applies to the tested source tree.

If an upstream revision publishes no license, record `NONE` with no license files instead of
guessing a license. The entry remains an external, read-only gitlink and does not grant permission
to copy or redistribute its source as part of Vize.

`compat-baseline.json` is the per-PR drop-in compatibility ratchet baseline: the accepted
vize/vue-tsc typecheck divergence over pinned probe workspaces cut from the hydrated vue-parity
fixtures. `tests/tooling/compat-ratchet.test.ts` recomputes the divergence on every PR and only
allows it to hold or improve; regenerate with `UPDATE_COMPAT_BASELINE=1` after intentional
compatibility improvements or pinned toolchain moves.

`vite-plugin-vue-option-parity.json` is the intentional-gap ledger behind the `@vizejs/vite-plugin`
drop-in claim: one entry per documented `@vitejs/plugin-vue` option, `Api` member, and plugin hook,
each recorded as `honored` (with the behavioral probe that proves it),
`intentional-divergence` (with a reason), or `unimplemented` (with issue #3227 and a reason).
`tests/tooling/vite-plugin-vue-option-parity.test.ts` re-enumerates the pinned upstream surface and
fails when an entry is missing, so a new or newly honored option cannot pass unrecorded.

`fixture-compatibility-ledger.json` joins every pinned gitlink to its ecosystem-matrix and App E2E
memberships, then records only evidence-backed Vue generations, API styles, Nuxt macros, and test
oracles. Capability presence, exercised behavior, and runtime verification are independent levels:
finding source text never promotes a project to runtime coverage. Run
`rust-script tools/commands/fixtures/fixture-compatibility-report.rs` for the deterministic coverage report. Any
unknown, unverified, or excluded compatibility dimension must retain a reason and tracking Issue.
