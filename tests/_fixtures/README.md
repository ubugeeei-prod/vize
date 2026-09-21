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

`davinci-remarks-baseline.folio` is the TS-32 optimization-remarks baseline (Davinci P3-13): every
remark the S2 transform pipeline emits over the in-repo `.vue` fixtures (`_git` excluded), keyed by
file. `cargo test -p vize_s1_to_s2 --features davinci-differential --test davinci_remarks_corpus`
requires exact equality; re-bless with `UPDATE_REMARKS_BASELINE=1`, which refuses any
`applied → missed` transition not explained in the baseline's `[remarks-corpus.explained]` section.

`davinci-s3-remarks-baseline.folio` is the same gate for S3 extraction (Davinci P3-10): every
`s3.extract-placements` remark the `-O3` S3 optimization pipeline emits over the same sweep.
`cargo test -p vize_s2_to_s3 --test davinci_s3_remarks_corpus` requires exact equality and re-blesses
under the same `UPDATE_REMARKS_BASELINE=1` rules.
