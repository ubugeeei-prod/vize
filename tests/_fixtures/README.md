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
